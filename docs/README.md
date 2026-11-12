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
