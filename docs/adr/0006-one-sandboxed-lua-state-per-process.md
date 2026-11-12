# 0006. One sandboxed Lua state per process

- Status: accepted
- Date: 2026-11-12

## Context

A Lua script run by kernelino, even a malicious one, must never reach the host file system or the network, never see another script, and never crash the system (`docs/CONTEXT.md`). Lua 5.4 through `mlua` ([ADR 0005](0005-real-lua-through-vendored-mlua.md)) ships libraries that do exactly that (`io`, `os`, `require`, `debug`) and can load precompiled bytecode.

## Decision

`LuaProcess::spawn` (`src/lua.rs`) creates one `Lua` state per process, and sets it up before the script runs:

- Only the `coroutine`, `table`, `string`, `utf8` and `math` libraries are loaded (`StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH`): no `io`, `os`, `package`/`require` or `debug`.
- `src/sandbox.lua` runs first in every state. It removes `dofile`, `loadfile` and `string.dump`, and replaces `load` with a version forced to text mode (`"t"`).
- `print` is replaced by a function that writes into a per-process buffer. The shell drains it with `LuaProcess::take_output` after every slice.
- The state has a memory limit (`Lua::set_memory_limit` with `Limits::memory`; the shell uses 64 MB in `LUA_LIMITS`, `src/shell.rs`).

## Alternatives considered

- **One state shared by every script.** Rejected: scripts could see and tamper with each other's globals and shared metatables, for example the string metatable.
- **Allowing bytecode** (`load` in binary mode, `string.dump`). Rejected: crafted bytecode can corrupt the Lua VM, and with it the host process.

## Consequences

- Scripts are isolated from each other and from the host. `host_access_is_not_available` and `bytecode_cannot_be_loaded` in `src/lua.rs` check it.
- Scripts have no `io`, `os` or `require` until roadmap milestone M4 maps `io` to the VFS.
- A script that allocates too much fails with `not enough memory`. Lua memory is limited per state but not counted in the VMM yet (a known issue in `docs/CONTEXT.md`).
- A new library or global for scripts must be added deliberately in `src/lua.rs` or `src/sandbox.lua`, and must not open a path to the host.
