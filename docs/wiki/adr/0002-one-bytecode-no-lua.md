# ADR 0002: One bytecode for CNS, ZSS and UMF; no Lua

- Status: Accepted
- Date: 2026-11-03

## Context

Characters arrive as M.U.G.E.N CNS, Ikemen ZSS or the native UMF JSON. Running three interpreters would triple the semantics to keep deterministic.

## Decision

All three formats compile to one bytecode executed by the engine VM. Lua scripts are not executed; the wiki lists what Lua packs do and the data-driven alternatives.

## Consequences

Fighters in any format fight each other with the same rules, and the tick stays allocation-free. Packs that depend on Lua logic need their behavior expressed in data. New controllers and triggers are added once in `crates/engine`.
