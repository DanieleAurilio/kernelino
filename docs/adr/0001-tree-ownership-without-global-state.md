# 0001. Tree ownership without global state

- Status: accepted
- Date: 2026-11-12

## Context

kernelino simulates an operating system made of several subsystems: a virtual file system (`src/vfs.rs`), a virtual memory manager (`src/vmm.rs`) and a virtual process manager (`src/vpm.rs`). Some operations need more than one of them at once (a file's bytes live in the VMM, see [ADR 0003](0003-file-contents-live-in-the-vmm.md)).

The previous design kept the VFS in a global, `lazy_static! { static ref VFS: RwLock<Vfs> }`, declared in the shell. The 2026-04-02 code review flagged it: global mutable state "makes testing and composition hard", and it suggested dependency injection instead.

## Decision

Subsystems are owned in a tree, with no global state:

- `Shell` (`src/shell.rs`) owns a `Kernel`.
- `Kernel` (`src/kernel.rs`) owns a `Vfs`, a `Vmm` and a `Vpm`, and coordinates them.
- `main.rs` builds the root and runs it: `Shell::new().run(stdin, stdout)`.

`Shell::run` takes its input as `impl BufRead` and its output as `impl Write`, so the caller decides where the shell reads and writes.

## Alternatives considered

- **Global `lazy_static! { static ref VFS: RwLock<Vfs> }`** (the previous design). Rejected: every function could reach and change the file system, tests would share one instance, and two kernels could not exist in one program. The 2026-04-02 code review flagged it as making testing and composition hard.

## Consequences

- Every unit test builds its own `Vfs`, `Vmm`, `Kernel` or `Shell` (for example `Kernel::new(PAGE_SIZE * 16)` in `src/kernel.rs`, `Shell::new().run(&mut script.as_bytes(), &mut out)` in `src/shell.rs`), so tests are isolated and can run in parallel.
- Borrowing is checked by the compiler: an operation that needs two subsystems goes through `Kernel`, which holds both.
- No locks are needed: kernelino is synchronous and single-threaded.
- A new subsystem is added as a field of `Kernel`, never as a global.
