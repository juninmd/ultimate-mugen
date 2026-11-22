# ADR 0010: Exact M.U.G.E.N. migration to UMF

- Status: Accepted
- Date: 2026-11-22

## Context

The Python converter (`tools/mugen2umf`, PR #964) rewrote M.U.G.E.N. states as native UMF moves. Each port lost behavior the heuristics did not model: Black Widow hyper cinematics, her throw and Widow's Strike jump, Goku SSJ4 palettes 2 to 5, throws and the charge ramp. Every fix was another heuristic.

## Decision

- A migrated fighter carries the original data inside UMF: a `mugen` block with the CNS, ST, common states and CMD text verbatim, AIR as JSON keyed by action, `[Info]`, and an exact sprite pack (`art.type: "mugen"`: one indexed PNG per SFF sprite, SFF palettes, `pal1..12.act`, sounds as WAV, `meta.json` in SFF order).
- The engine loader is split into `read_mugen_sources` and `assemble_mugen_char(MugenSources)`; `ume_native::exact` rebuilds `MugenSources` from the bundle, so a migrated fighter runs through the same code path as the original.
- UMF-only data (profile, quotes, rivals, move names, extra routes and scripts) is layered on top; native `moves` on top of exact fighters follow in a separate change.
- The converter, parity check and MCP server are one Rust crate (`tools/mugen2umf`). Parity is a structural `CharDef` diff plus every `Statedef -1` route played from its real input for 150 ticks on both sides, compared tick by tick.

## Consequences

- Black Widow, Goku SSJ4 (two versions), Gogeta SSJ4, Super Majin Vegeta, Goku SSJ GT, Aizen (SFF v2) and Goku UI are identical on every data section and every route.
- Engine gaps now affect the original and the migrated fighter the same way and are fixed in the engine, not in the converter.
- Bundles are larger than hand-tuned UMF moves (one PNG per sprite); the LZ4 container of [ADR 0009](0009-lz4-sprite-container.md) can pack them later.
