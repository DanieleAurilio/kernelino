# 0007. Preemption through a global instruction hook

- Status: accepted
- Date: 2026-11-12

## Context

A script must never hang kernelino: the scheduler has to get control back, and a script that runs forever has to be stopped. Scripts are plain Lua and cannot be trusted to yield on their own.

The spike of roadmap milestone M2 (`docs/CONTEXT.md`) showed that a Lua 5.4 count hook can yield through `mlua` (`VmState::Yield`), but only where the script is yieldable. Inside a C call such as a `table.sort` comparator the yield is refused.

## Decision

`LuaProcess` (`src/lua.rs`) installs a **global** hook with `Lua::set_global_hook`, called every `HOOK_EVERY` (1,000) instructions. It counts the instructions used by the process and in the current slice, then:

- kills the process when it has used more than `Limits::instructions` in total (`process killed: CPU limit exceeded`);
- kills it when it has run more than `STUCK_LIMIT` (1,000,000) instructions past its slice without yielding, which only happens in a non-yieldable section (`process killed: too long without yielding to the scheduler`);
- otherwise asks to yield (`VmState::Yield`) once the slice reaches `TIME_SLICE` (10,000) instructions.

`LuaProcess::step` runs the process for one slice.

The hook is global so that coroutines created by the script inherit it. A yield in a script coroutine would return to the script's own `coroutine.resume`, not to the scheduler, so `src/sandbox.lua` wraps `coroutine.resume`, `coroutine.yield` and `coroutine.wrap`: a preemption inside a script coroutine is passed up to the scheduler and the coroutine is resumed as if nothing happened. It also wraps `pcall`, `xpcall` and `coroutine.resume` so that, once the process is killed, the kill is raised again and cannot be caught.

## Alternatives considered

- **A per-thread hook.** Rejected: coroutines created by the script would not have it, and could loop forever without being preempted.
- **OS threads with timeouts.** Rejected: slicing would not be deterministic, and kernelino is synchronous, with no async runtime and no threads ([ADR 0001](0001-tree-ownership-without-global-state.md)).

## Consequences

- The shell runs `lua <path>` slice by slice, draining the output after each slice (`Command::Lua` in `src/shell.rs`), with 1e9 instructions as its CPU limit (about 2 s in release).
- Preemption is invisible to scripts, including inside nested coroutines; `preemption_is_invisible_to_coroutines` and `nested_coroutines_keep_their_values` in `src/lua.rs` check it.
- Yields only work where the script is yieldable, hence `STUCK_LIMIT`: a non-yieldable section can run longer than a slice, but not forever.
- Because a process runs one slice per `LuaProcess::step`, the scheduler of roadmap milestone M4 can interleave several processes.
