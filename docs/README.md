# kernelino documentation

kernelino is a sandbox OS shell written in Rust. People use it from the CLI to run real Lua 5.4 scripts as processes, confined to RAM: a script (even malicious) must never crash or hang the system, reach the host filesystem or the network. This folder is the source of truth for humans and coding harnesses alike.

## Overview

See [`CONTEXT.md`](CONTEXT.md): what kernelino is, its tech stack, key architectural decisions, roadmap and known issues.

## Architecture decisions

See `adr/`.

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
