# 0009. Hand-written, validated knowledge graph

- Status: accepted
- Date: 2026-11-12

## Context

`docs/` is the source of truth for humans and coding harnesses alike (`docs/README.md`). A harness that changes a module needs to know which ADRs govern it, which tests cover it and which domain concepts it implements, without reading the whole repository. `AGENTS.md` points to `docs/graph.json` as that machine-readable map.

A map that is not checked goes stale at the first change that forgets it.

## Decision

`docs/graph.json` is written by hand and validated by `tests/docs_graph.rs`, which runs in `cargo test`. The validator checks that:

- every file the graph names exists;
- every file of the mapped folders (`src/`, `tests/*.rs`, `docs/adr/`, `docs/ddd/`) is a node;
- every edge links two existing nodes, with one of the allowed kinds;
- every ADR governs at least one node.

`docs/graph.json` and `tests/docs_graph.rs` are added by the knowledge graph ticket of the same plan; this ADR may be merged before it.

## Alternatives considered

- **Generating the graph from the code or from front-matter.** Rejected: the semantic edges, for example which ADR governs which module, cannot be derived from the code.
- **Writing it by hand without validation.** Rejected: it would be stale at the first pull request.

## Consequences

- Every pull request that adds, removes or renames a mapped file must update `docs/graph.json`, or `cargo test` fails. `AGENTS.md` states this rule.
- `docs/adr/` must contain only ADR files, since every `docs/adr/*.md` is mapped to an ADR node: no index file inside it. The list of ADRs is in `docs/README.md`.
- A new ADR must govern at least one node of the graph.
