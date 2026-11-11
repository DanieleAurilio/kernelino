# kernelino — start here

Read `docs/README.md` first: it is the index of the whole documentation.

- `docs/CONTEXT.md` — what kernelino is, tech stack, roadmap, known issues
- `docs/adr/` — architecture decisions: read the ones that govern a module before changing it
- `docs/ddd/` — domain model: bounded contexts, glossary, context map
- `docs/graph.json` — machine-readable map linking modules, ADRs, contexts and tests

## Commands
- `cargo fmt --check`
- `cargo clippy --all-targets`
- `cargo test`

## Rules
- Use the terms of `docs/ddd/glossary.md` in code, docs and commits.
- When you add, remove or rename a file in `src/`, `tests/*.rs`, `docs/adr/` or `docs/ddd/`, update `docs/graph.json` (`cargo test` fails otherwise).
- A change that contradicts an ADR needs a new ADR that supersedes it.
