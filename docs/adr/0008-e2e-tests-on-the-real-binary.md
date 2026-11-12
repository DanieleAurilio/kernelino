# 0008. End-to-end tests on the real binary

- Status: accepted
- Date: 2026-11-12

## Context

Unit tests are inline `#[cfg(test)]` modules in each `src/*.rs` file; `Shell::run` is already tested in-process in `src/shell.rs`. Nothing yet tests the built binary as a user runs it: `main.rs`, the terminal clearing of `utils::clear_terminal`, and real stdin and stdout.

kernelino is a binary crate only (`src/main.rs`, no `src/lib.rs`), so code in `tests/` cannot call `Shell` directly.

## Decision

An end-to-end suite in `tests/e2e.rs` runs the real binary, through the path Cargo gives integration tests (`env!("CARGO_BIN_EXE_kernelino")`). For each scenario `<name>` it:

1. feeds `tests/e2e/<name>.in` on stdin;
2. strips the ANSI escape sequences from stdout (`main.rs` clears the terminal at start-up);
3. compares the result with `tests/e2e/<name>.out`.

A scenario is a pair of plain-text files: a new one needs no Rust. `tests/e2e.rs` and `tests/e2e/` are added by the end-to-end suite ticket of the same plan; this ADR may be merged before it.

## Alternatives considered

- **A `src/lib.rs` split, to test `Shell::run` from `tests/`.** Rejected: it refactors the crate for testing only, and `Shell::run` is already tested in-process in `src/shell.rs`.
- **Assertions written inline in Rust.** Rejected: a harness or a developer could not add a scenario without writing Rust.

## Consequences

- The suite runs in `cargo test` with the unit tests.
- `top` cannot be covered: it enables raw mode and writes to the real stdout, which needs a TTY.
- Scenarios must not hit the 1e9-instruction CPU limit of `lua <path>` (`LUA_LIMITS` in `src/shell.rs`): it takes about 2 s in release and much longer in debug builds.
- A change to the shell's output means updating the `.out` files it changes.
