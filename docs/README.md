# kernelino documentation

kernelino is a sandbox OS shell written in Rust. People use it from the CLI to run real Lua 5.4 scripts as processes, confined to RAM: a script (even malicious) must never crash or hang the system, reach the host filesystem or the network. This folder is the source of truth for humans and coding harnesses alike.

## Overview

See [`CONTEXT.md`](CONTEXT.md): what kernelino is, its tech stack, key architectural decisions, roadmap and known issues.

## Architecture decisions

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

See `ddd/`.

## Knowledge graph

See `graph.json`.

## Testing

Unit tests are inline `#[cfg(test)]` modules in each `src/*.rs` file. Run them with:

```sh
cargo test
```
