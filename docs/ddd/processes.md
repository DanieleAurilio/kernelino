# Processes

## Responsibility

Processes holds the process table and runs Lua scripts as sandboxed processes: a script, even a malicious one, must never crash or hang the system nor reach the host. It decides how long a Lua process runs before control returns to the scheduler, what it may use, and when it is killed. The process table and the Lua sandbox share this context because roadmap M4 merges them into real scheduled processes.

## Code

- [`src/vpm.rs`](../../src/vpm.rs): `Vpm`, `Process` (the process table).
- [`src/lua.rs`](../../src/lua.rs): `LuaProcess`, `Limits`, `Step`, `TIME_SLICE`, `STUCK_LIMIT`.
- [`src/sandbox.lua`](../../src/sandbox.lua): the Lua code run in every new Lua process before its script.

## Concepts

### Process

An entry of the process table (`Process`): a PID, a name and its start time, from which `uptime` is computed. The table (`Vpm::processes`) is created with one Process, `shell`, PID 1, and has no operation to add or remove one: today it holds only the Shell. `top` shows it.

### Lua process

A Lua 5.4 script run as a process (`lua <path>`, `LuaProcess::spawn(name, source, limits)`):
- in its own Lua state, so scripts cannot see or tamper with each other (e.g. the string metatable);
- with only safe libraries: `coroutine`, `table`, `string`, `utf8`, `math`; no host access: no `io`, `os`, `require`, `package`, `debug`; `dofile`, `loadfile` and `string.dump` are removed, and `load` accepts only text chunks (no bytecode: crafted bytecode can corrupt the VM);
- its `print` writes into a per-process output (`take_output` drains it), that the Shell prints after every Time slice;
- its chunk is named after the absolute path of the script, so its errors read `/path.lua:<line>: <message>`.

A syntax error fails `spawn`. A started Lua process ends Finished or Failed.

### Limits

The resources a Lua process may use (`Limits`): `memory`, the memory limit of its Lua state in bytes, and `instructions`, its total instruction budget (the CPU limit). Exceeding either fails the process; the Shell keeps running. The Shell's values (`LUA_LIMITS`): 64 MB and 1 000 000 000 instructions, about 2 s in a release build.

### Time slice

The number of instructions a Lua process runs before control returns to the scheduler: `TIME_SLICE`, 10 000 instructions. Instructions are counted by a hook every 1 000 instructions (`HOOK_EVERY`), so a slice ends within 1 000 instructions of its limit. `LuaProcess::step` runs one Time slice.

### Preemption

The forced return of control to the scheduler at the end of a Time slice: the instruction hook makes the Lua process yield. The hook is global, so it also runs in the coroutines the script creates. `sandbox.lua` makes Preemption invisible to the script, even inside its own coroutines: when a coroutine of the script is preempted, the wrapped `coroutine.resume` passes the yield up to the scheduler, then resumes the coroutine as if nothing happened, so the script only ever sees its own `coroutine.yield` values. Where the script cannot yield (e.g. inside a `table.sort` comparator), the Preemption is refused and tried again at the next hook.

### Kill

The termination of a Lua process by the system, with a Failed Step:
- on CPU limit: more instructions than `Limits::instructions` (`process killed: CPU limit exceeded`);
- when stuck: more than `STUCK_LIMIT` (1 000 000) instructions past its Time slice where it cannot be preempted (`process killed: too long without yielding to the scheduler`).

A Kill cannot be caught: once killed, every hook raises the error again, and the wrapped `pcall`, `xpcall` and `coroutine.resume` re-raise it instead of returning it.

### Step

The outcome of one Time slice (`Step`):
- `Running`: the Time slice ended and the process can run again;
- `Finished`: the script returned;
- `Failed(message)`: a Lua error not caught by the script, the memory limit, or a Kill. The message has no stack traceback.

### Scheduler

Today the scheduler is the Shell's `lua` command: it runs one Lua process in the foreground, calling `step` until the Step is not `Running`.

## Errors

This context has no error type: `LuaProcess::spawn` returns the message of the error as a `String`, and a failed process ends with `Step::Failed(String)`.

| Message | Meaning |
|---|---|
| `<path>:<line>: <message>` | a syntax error at spawn, or a runtime error not caught by the script |
| `not enough memory` | the Lua state reached `Limits::memory` |
| `process killed: CPU limit exceeded` | Kill: `Limits::instructions` exceeded |
| `process killed: too long without yielding to the scheduler` | Kill: stuck for `STUCK_LIMIT` instructions past the Time slice |

## Not in this context

- Lua processes are not in the process table yet: `top` shows only the Shell (known gap of [`docs/CONTEXT.md`](../CONTEXT.md), roadmap M4).
- No background processes, no `ps` nor `kill` command, no per-process memory quota in [Memory](memory.md) (roadmap M4); a foreground Lua process stops only at its end, on failure or on a Kill (no interrupt key).
- Lua processes cannot reach the [File system](filesystem.md) (Lua `io` mapped to it is roadmap M4), the host, nor the network.
