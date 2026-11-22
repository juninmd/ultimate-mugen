# Architecture decision records

An ADR records one decision that changes a crate boundary, a dependency or a data format, so later contributors know why it was made and when it can be revisited.

## How to add one

1. Copy `0000-template.md` to `NNNN-short-title.md` with the next free number.
2. Fill the sections; keep it under one page.
3. Open it in the same pull request as the change, with status `Proposed`; set `Accepted` when it merges.
4. Never edit an accepted ADR to change its decision: add a new ADR that supersedes it and update the old status line.

## Index

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-wgpu-winit-shell.md) | Own wgpu renderer and winit shell, no game framework | Accepted |
| [0002](0002-one-bytecode-no-lua.md) | One bytecode for CNS, ZSS and UMF; no Lua | Accepted |
| [0003](0003-deterministic-single-thread-simulation.md) | Deterministic single-thread simulation for rollback | Accepted |
| [0004](0004-wiki-mdbook-on-pages.md) | Wiki rendered with mdBook under `/wiki/` on GitHub Pages | Accepted |
| [0005](0005-crate-boundaries-enforced-in-ci.md) | Crate dependency boundaries enforced in CI | Accepted |
| [0006](0006-content-is-data-first.md) | Content is data first, with a documented extension table | Accepted |
| [0007](0007-umf2-definitive-fighter-format.md) | UMF 2: JSON kept, compact frames, canonical layout, quality floor | Accepted |
| [0008](0008-umf-schema-and-storage-review.md) | UMF JSON Schema checked in CI; JSON + PNG kept after measuring alternatives | Accepted / Proposed |
