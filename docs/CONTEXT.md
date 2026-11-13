# Kernelino — Project Context

## What is it?
A sandbox OS shell written in Rust. People use it from the CLI to run **real Lua 5.4 scripts as processes**, confined to RAM: a script (even malicious) must never crash or hang the system, reach the host filesystem or the network.

- A REPL shell (`shell.rs`)
- Kernel that owns and coordinates the subsystems (`kernel.rs`)
- Virtual file system (`vfs.rs`) — in-memory tree, navigated by path
- Virtual memory manager (`vmm.rs`) — page table + lazily allocated frames
- Virtual process manager (`vpm.rs`) — process table (today only the shell)
- A minimal line editor (`editor.rs`)
- Sandboxed Lua 5.4 processes (`lua.rs` + `sandbox.lua`)

## Tech stack
- Rust 2021 edition, synchronous (no async runtime)
- crossterm (terminal)
- mlua 0.12 with vendored Lua 5.4 (C, compiled by the build)

## Key architectural decisions

- Ownership: `Shell` → `Kernel` → { `Vfs`, `Vmm`, `Vpm` }. No global state.
- VFS has no parent pointers: everything is resolved from absolute paths (`Vfs::resolve`).
- File contents live in the VMM; a `File` only stores its pages and size. `Kernel` keeps them consistent (frees pages on rewrite/remove).
- Errors are typed (`VfsError`, `VmmError`, `KernelError`); the shell prints them and continues.
- Every command runs under `catch_unwind`: a panic aborts the command, not the shell.
- Lua sandbox (`lua.rs`):
  - one Lua state per process (scripts cannot tamper with each other, e.g. the string metatable);
  - only coroutine/table/string/utf8/math libs; `dofile`, `loadfile`, `string.dump` removed; `load` is text-only (crafted bytecode can corrupt the VM);
  - `print` writes into a per-process buffer the shell drains after every slice;
  - memory limit per state; a **global** instruction hook (inherited by script coroutines) yields every `TIME_SLICE` instructions and kills on CPU limit or when the script runs `STUCK_LIMIT` past its slice in a non-yieldable section (e.g. `table.sort` comparator);
  - `sandbox.lua` wraps `coroutine.resume/yield/wrap` so a preemption inside a script coroutine is passed up to the scheduler and stays invisible to the script, and wraps `pcall/xpcall/coroutine.resume` so a kill cannot be caught.
- Shell `lua <path>` runs a process in the foreground, slice by slice (64 MB, 1e9 instructions ≈ 2 s in release).


## Roadmap
- M0 ✅ hygiene: tests, CI, unused deps removed, no panic/hang reachable from the shell, help aligned
- M1 ✅ VFS without `parent`, VMM frame tracking fixed
- M2 ✅ spike: Lua 5.4 count hooks can yield via mlua (`VmState::Yield`), but only where the script is yieldable
- M3 ✅ mlua integration: sandboxed `lua <path>`, hand-written lexer/parser/ast removed
- M4 real processes: scheduler, fuel, per-process memory quota, `run &`, `ps`, `kill`; Lua `io` mapped to the VFS
- M5 users and permissions in the VFS (processes must not read each other's files)
- No network access for scripts (for now)

## Entry point
`main.rs` → `Shell::new().run(stdin, stdout)`

## Known issues
- `top` shows only the shell process until M4; Lua processes are not in the process table yet.
- A foreground script can only be stopped by its CPU limit (no interrupt key yet).
- Lua memory is limited per state but not accounted in the VMM yet.
