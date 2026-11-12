# kernelino documentation

kernelino is a sandbox OS shell written in Rust. People use it from the CLI to run real Lua 5.4 scripts as processes, confined to RAM: a script (even malicious) must never crash or hang the system, reach the host filesystem or the network. This folder is the source of truth for humans and coding harnesses alike.

## Overview

See [`CONTEXT.md`](CONTEXT.md): what kernelino is, its tech stack, key architectural decisions, roadmap and known issues.

## Architecture decisions

See `adr/`.

## Domain model

See `ddd/`.

## Knowledge graph

See `graph.json`.

## Testing

Unit tests are inline `#[cfg(test)]` modules in each `src/*.rs` file. Run them with:

```sh
cargo test
```

### End-to-end suite

`tests/e2e.rs` spawns the compiled `kernelino` binary for every scenario of `tests/e2e/`, writes the scenario to its stdin, closes it, and compares the whole stdout with a golden file. A scenario is a pair of plain text files:

- `tests/e2e/<name>.in` — exactly what is typed on stdin: one command per line, editor lines included.
- `tests/e2e/<name>.out` — the expected stdout, with ANSI escape sequences (such as the screen clear at startup) stripped. Since stdin is not echoed, it holds the prompts `kernelino> ` and `> ` followed directly by the output.

The test fails if the binary exits with an error or writes to stderr, and lists every scenario whose output differs. `cargo test` runs it; `cargo test --test e2e` runs it alone.

To add a scenario:

1. Write `tests/e2e/<name>.in`.
2. Generate its `.out` with `KERNELINO_BLESS=1 cargo test --test e2e`: it writes the actual output of every scenario instead of comparing.
3. Review the `.out` by hand against the behaviour of `src/shell.rs` before committing it.

`top` is not covered: it enables raw mode and needs a TTY. Scenarios must not hit the Lua CPU limit, which is too slow to reach in debug builds; process kills are covered by the unit tests of `src/lua.rs`.
