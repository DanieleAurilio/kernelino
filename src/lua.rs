/**
 * Sandboxed Lua 5.4 processes
 *
 * Every process gets its own Lua state, so scripts cannot see or tamper with
 * each other. Only safe libraries are loaded (see sandbox.lua), and a global
 * instruction hook preempts the script at the end of every time slice, so the
 * scheduler always gets control back.
 */
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use mlua::{
    thread::ThreadStatus, HookTriggers, Lua, LuaOptions, MultiValue, StdLib, Thread, VmState,
};

const SANDBOX: &str = include_str!("sandbox.lua");
const HOOK_EVERY: u64 = 1_000;
pub const TIME_SLICE: u64 = 10_000;
/// How far past its slice a process may run when it cannot be preempted,
/// e.g. inside a `table.sort` comparator.
const STUCK_LIMIT: u64 = 1_000_000;

const CPU_LIMIT_EXCEEDED: &str = "process killed: CPU limit exceeded";
const STUCK: &str = "process killed: too long without yielding to the scheduler";

pub struct Limits {
    pub memory: usize,
    pub instructions: u64,
}

#[derive(Default)]
struct Cpu {
    used: Cell<u64>,
    slice: Cell<u64>,
    /// Set when a coroutine of the script, not the process itself, got preempted.
    preempted: Cell<bool>,
    killed: Cell<Option<&'static str>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    Running,
    Finished,
    Failed(String),
}

pub struct LuaProcess {
    thread: Thread,
    cpu: Rc<Cpu>,
    output: Rc<RefCell<Vec<u8>>>,
    _lua: Lua,
}

impl LuaProcess {
    pub fn spawn(name: &str, source: &str, limits: &Limits) -> Result<Self, String> {
        Self::try_spawn(name, source, limits).map_err(|err| message(&err))
    }

    fn try_spawn(name: &str, source: &str, limits: &Limits) -> mlua::Result<Self> {
        let lua = Lua::new_with(
            StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH,
            LuaOptions::default(),
        )?;
        let cpu = Rc::new(Cpu::default());
        let output = Rc::new(RefCell::new(Vec::new()));
        let main: Rc<OnceCell<Thread>> = Rc::default();

        let write = {
            let output = Rc::clone(&output);
            lua.create_function(move |_, text: mlua::LuaString| {
                output.borrow_mut().extend_from_slice(&text.as_bytes());
                Ok(())
            })?
        };
        let take_preempted = {
            let cpu = Rc::clone(&cpu);
            lua.create_function(move |_, ()| Ok(cpu.preempted.replace(false)))?
        };
        let preempt = {
            let (cpu, main) = (Rc::clone(&cpu), Rc::clone(&main));
            lua.create_function(move |lua, ()| {
                if !is_main(lua, &main) {
                    cpu.preempted.set(true);
                }
                Ok(())
            })?
        };
        let killed = {
            let cpu = Rc::clone(&cpu);
            lua.create_function(move |_, ()| Ok(cpu.killed.get()))?
        };
        lua.load(SANDBOX).set_name("=sandbox").call::<()>((
            write,
            take_preempted,
            preempt,
            killed,
        ))?;

        // Global, so that coroutines created by the script inherit it.
        {
            let (cpu, main) = (Rc::clone(&cpu), Rc::clone(&main));
            let max_instructions = limits.instructions;
            lua.set_global_hook(
                HookTriggers::new().every_nth_instruction(HOOK_EVERY as u32),
                move |lua, _| hook(lua, &cpu, &main, max_instructions),
            )?;
        }
        lua.set_memory_limit(limits.memory)?;

        let script = lua
            .load(source)
            .set_name(format!("@{name}"))
            .into_function()?;
        let thread = lua.create_thread(script)?;
        let _ = main.set(thread.clone());

        Ok(Self {
            thread,
            cpu,
            output,
            _lua: lua,
        })
    }

    /// Runs the process for one time slice.
    pub fn step(&mut self) -> Step {
        self.cpu.slice.set(0);
        match self.thread.resume::<MultiValue>(()) {
            Err(err) => Step::Failed(message(&err)),
            Ok(_) if self.thread.status() == ThreadStatus::Resumable => Step::Running,
            Ok(_) => Step::Finished,
        }
    }

    pub fn take_output(&self) -> Vec<u8> {
        std::mem::take(&mut self.output.borrow_mut())
    }
}

fn is_main(lua: &Lua, main: &OnceCell<Thread>) -> bool {
    main.get() == Some(&lua.current_thread())
}

fn hook(
    lua: &Lua,
    cpu: &Cpu,
    main: &OnceCell<Thread>,
    max_instructions: u64,
) -> mlua::Result<VmState> {
    cpu.used.set(cpu.used.get() + HOOK_EVERY);
    cpu.slice.set(cpu.slice.get() + HOOK_EVERY);

    if cpu.used.get() > max_instructions {
        cpu.killed.set(Some(CPU_LIMIT_EXCEEDED));
    } else if cpu.slice.get() > TIME_SLICE + STUCK_LIMIT {
        cpu.killed.set(Some(STUCK));
    }
    if let Some(reason) = cpu.killed.get() {
        return Err(mlua::Error::runtime(reason));
    }

    if cpu.slice.get() < TIME_SLICE {
        return Ok(VmState::Continue);
    }
    if !is_main(lua, main) {
        cpu.preempted.set(true);
    }
    // Ignored by mlua where the script cannot yield: the next call tries again.
    Ok(VmState::Yield)
}

/// The error message without Rust-side decorations and stack traceback.
fn message(err: &mlua::Error) -> String {
    let text = match err {
        mlua::Error::RuntimeError(text) | mlua::Error::MemoryError(text) => text.clone(),
        mlua::Error::SyntaxError { message, .. } => message.clone(),
        mlua::Error::CallbackError { cause, .. } => return message(cause),
        err => err.to_string(),
    };
    text.split("\nstack traceback:")
        .next()
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: Limits = Limits {
        memory: 8 * 1024 * 1024,
        instructions: 20_000_000,
    };

    /// Runs to the end, like the scheduler would. Returns output, outcome and slices used.
    fn run(source: &str) -> (String, Step, usize) {
        let mut process = LuaProcess::spawn("test.lua", source, &LIMITS).unwrap();
        let mut output = Vec::new();
        let mut slices = 0;
        loop {
            let step = process.step();
            slices += 1;
            output.extend(process.take_output());
            if step != Step::Running {
                return (String::from_utf8(output).unwrap(), step, slices);
            }
        }
    }

    fn outcome(source: &str) -> Step {
        run(source).1
    }

    #[test]
    fn prints_like_lua() {
        let (output, step, _) = run("print('hello', 1, 2.5, nil, true) print()");
        assert_eq!(step, Step::Finished);
        assert_eq!(output, "hello\t1\t2.5\tnil\ttrue\n\n");
    }

    #[test]
    fn is_lua_5_4() {
        let (output, _, _) = run(
            "local x <const> = 7 // 2 goto skip do return end ::skip:: print(x, math.type(x), _VERSION)",
        );
        assert_eq!(output, "3\tinteger\tLua 5.4\n");
    }

    #[test]
    fn long_loops_are_preempted() {
        let (_, step, slices) = run("local n = 0 while n < 100000 do n = n + 1 end");
        assert_eq!(step, Step::Finished);
        assert!(slices > 10);
    }

    #[test]
    fn preemption_is_invisible_to_coroutines() {
        let (output, step, slices) = run("local gen = coroutine.wrap(function()
                local n = 0
                for i = 1, 300000 do n = n + 1 end
                coroutine.yield(n)
                coroutine.yield('second')
            end)
            print(gen(), gen())");
        assert_eq!(step, Step::Finished);
        assert_eq!(output, "300000\tsecond\n");
        assert!(slices > 10);
    }

    #[test]
    fn nested_coroutines_keep_their_values() {
        let (output, _, _) = run("local outer = coroutine.wrap(function()
                local inner = coroutine.wrap(function()
                    local s = 0
                    for i = 1, 200000 do s = s + i end
                    coroutine.yield(s)
                end)
                coroutine.yield(inner())
            end)
            print(outer())");
        assert_eq!(output, "20000100000\n");
    }

    #[test]
    fn refused_preemption_is_not_mistaken_for_a_yield() {
        // The comparator burns a slice where yielding is impossible, then the coroutine yields for real.
        let (output, _, _) = run("local co = coroutine.wrap(function()
                table.sort({3, 2, 1}, function(a, b)
                    for i = 1, 20000 do end
                    return a < b
                end)
                coroutine.yield('real')
            end)
            print(co())");
        assert_eq!(output, "real\n");
    }

    #[test]
    fn infinite_loop_is_killed() {
        assert_eq!(
            outcome("while true do end"),
            Step::Failed(CPU_LIMIT_EXCEEDED.into())
        );
    }

    #[test]
    fn infinite_loop_in_coroutine_is_preempted_and_killed() {
        let (_, step, slices) = run("coroutine.wrap(function() while true do end end)()");
        assert_eq!(step, Step::Failed(CPU_LIMIT_EXCEEDED.into()));
        assert!(slices > 10);
    }

    #[test]
    fn unpreemptable_loop_is_killed() {
        assert_eq!(
            outcome("table.sort({3, 2, 1}, function() while true do end end)"),
            Step::Failed(STUCK.into())
        );
    }

    #[test]
    fn kill_cannot_be_caught() {
        for source in [
            "table.sort({3, 2, 1}, function() while true do pcall(function() while true do end end) end end)",
            "table.sort({3, 2, 1}, function() while true do xpcall(function() while true do end end, print) end end)",
            "table.sort({3, 2, 1}, function() while true do coroutine.resume(coroutine.create(function() while true do end end)) end end)",
        ] {
            assert_eq!(outcome(source), Step::Failed(STUCK.into()), "{source}");
        }
    }

    #[test]
    fn memory_is_limited() {
        assert_eq!(
            outcome("local t = {} for i = 1, 1e9 do t[i] = string.rep('x', 1000) end"),
            Step::Failed("not enough memory".into())
        );
    }

    #[test]
    fn host_access_is_not_available() {
        let (output, _, _) =
            run("print(io, os, require, package, debug, dofile, loadfile, string.dump)");
        assert_eq!(output, "nil\tnil\tnil\tnil\tnil\tnil\tnil\tnil\n");
    }

    #[test]
    fn bytecode_cannot_be_loaded() {
        let (output, _, _) = run("print(load('\\27Lua'))");
        assert!(output.starts_with("nil\t"), "{output}");
        assert!(output.contains("binary"), "{output}");
    }

    #[test]
    fn errors_keep_lua_semantics() {
        let (output, step, _) = run("print(pcall(error, 'boom')) error('fatal')");
        assert_eq!(output, "false\tboom\n");
        assert_eq!(step, Step::Failed("test.lua:1: fatal".into()));
    }

    #[test]
    fn syntax_errors_are_reported_at_spawn() {
        let err = LuaProcess::spawn("bad.lua", "x = = 1", &LIMITS)
            .err()
            .unwrap();
        assert!(err.starts_with("bad.lua:1:"), "{err}");
    }
}
