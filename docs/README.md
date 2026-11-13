# kernelino documentation

kernelino is a sandbox OS shell written in Rust. People use it from the CLI to run real Lua 5.4 scripts as processes, confined to RAM: a script (even malicious) must never crash or hang the system, reach the host filesystem or the network. This folder is the source of truth for humans and coding harnesses alike.

## Overview

See [`CONTEXT.md`](CONTEXT.md): what kernelino is, its tech stack, key architectural decisions, roadmap and known issues.

## Architecture decisions

See `adr/`.

## Domain model

See `ddd/`.

Read the ones that govern a module before changing it. A change that contradicts an ADR needs a new ADR that supersedes it.

- [0001. Tree ownership without global state](adr/0001-tree-ownership-without-global-state.md)
- [0002. VFS without parent pointers](adr/0002-vfs-without-parent-pointers.md)
- [0003. File contents live in the VMM](adr/0003-file-contents-live-in-the-vmm.md)
- [0004. Typed errors and panic isolation](adr/0004-typed-errors-and-panic-isolation.md)
- [0005. Real Lua through vendored mlua](adr/0005-real-lua-through-vendored-mlua.md)
- [0006. One sandboxed Lua state per process](adr/0006-one-sandboxed-lua-state-per-process.md)
- [0007. Preemption through a global instruction hook](adr/0007-preemption-through-a-global-instruction-hook.md)
- [0008. End-to-end tests on the real binary](adr/0008-e2e-tests-on-the-real-binary.md)
- [0009. Hand-written, validated knowledge graph](adr/0009-hand-written-validated-knowledge-graph.md)

## Domain model

- [Glossary](ddd/glossary.md) — the terms of the domain, their context and their meaning
- [Context map](ddd/context-map.md) — how the bounded contexts relate in the code today
- [Shell](ddd/shell.md) — commands, the command line grammar, the editor, error output
- [Kernel](ddd/kernel.md) — ownership of the subsystems, file operations that keep tree and bytes consistent
- [File system](ddd/filesystem.md) — nodes, the working directory, path resolution
- [Memory](ddd/memory.md) — pages, frames, the page table, allocation
- [Processes](ddd/processes.md) — the process table, sandboxed Lua processes, time slices and kills

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
