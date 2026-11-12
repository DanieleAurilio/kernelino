# 0004. Typed errors and panic isolation

- Status: accepted
- Date: 2026-11-12

## Context

kernelino must never crash because of what a user types or what a script does. Most failures are expected: a missing path, a file where a directory is expected, memory running out.

The 2026-04-02 code review found `panic!` used for recoverable errors throughout the previous parser: "Every unexpected token results in `panic!`. A production parser should return `Result<Stmt, ParseError>`."

## Decision

- Every subsystem returns typed errors: `VfsError` (`src/vfs.rs`), `VmmError` (`src/vmm.rs`) and `KernelError` (`src/kernel.rs`, which wraps the other two through `From`). Each implements `Display` and `std::error::Error`.
- `Shell::run` (`src/shell.rs`) prints a failed command as `<command>: <error>` and reads the next line.
- Every command runs under `panic::catch_unwind`. If a command panics anyway, the shell prints `internal error: the command was aborted` and goes on.
- A line that is not a valid command is reported by `Command::parse` as `Unknown command: <name>` or `usage: <usage>`, without running anything.

## Alternatives considered

- **`panic!` for recoverable errors** (the previous design). Rejected: a typo or a missing file would take the whole shell down. The 2026-04-02 code review flagged it.

## Consequences

- No input reachable from the shell can take it down: expected failures are reported as errors, and an unexpected panic only aborts the command.
- New failure modes are added as variants of the subsystem's error enum, with a `Display` message, not as panics.
- `catch_unwind` is a safety net, not a way to report errors: code must still return `Err` for anything it can foresee.
