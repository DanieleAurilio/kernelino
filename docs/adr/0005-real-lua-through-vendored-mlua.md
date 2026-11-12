# 0005. Real Lua through vendored mlua

- Status: accepted
- Date: 2026-11-12

## Context

kernelino runs Lua scripts as processes. The previous design ran them with a hand-written interpreter for a subset of Lua: a lexer (`lexer.rs`), a recursive descent parser (`parser.rs`) and AST types (`ast.rs`).

The 2026-04-02 code review found several correctness bugs in it: `is_peek_char_digit` returned `true` when there was no next character, `Ast::build` parsed the program and then threw the result away, and the lexer had no keywords for `and`, `or` and `not`. It also found O(n²) lexing and parsing, because the lexer read characters with `chars().nth(position)` and the parser read tokens with `iter().nth(pos)`, both linear per call.

## Decision

Scripts run on Lua 5.4 through the `mlua` crate, version 0.12, with the `lua54` and `vendored` features (`Cargo.toml`). The `vendored` feature compiles the Lua C sources into kernelino, so no system Lua is needed. The hand-written `lexer.rs`, `parser.rs` and `ast.rs` were removed (roadmap milestone M3 in `docs/CONTEXT.md`). `src/lua.rs` wraps `mlua` into `LuaProcess`.

## Alternatives considered

- **Keep growing the hand-written Lua-subset interpreter.** Rejected: the 2026-04-02 code review found correctness bugs and O(n²) lexing and parsing in it, and full Lua semantics (metatables, coroutines, closures, the standard libraries) would take a long time to reach and to keep correct.

## Consequences

- Scripts get full Lua 5.4 semantics, including integer division, `<const>` locals, `goto` and coroutines.
- A C toolchain is needed at build time to compile the vendored Lua.
- Because the Lua VM is real C code, what scripts can reach must be restricted explicitly: see [ADR 0006](0006-one-sandboxed-lua-state-per-process.md) and [ADR 0007](0007-preemption-through-a-global-instruction-hook.md).
