# Architecture

**English** · [Português (BR)](pt-BR/Arquitetura.md)

How the codebase is split, which direction dependencies may point, and where to extend it. Decisions behind these rules live in the [ADRs](adr/README.md).

## Crate map

| Crate | Responsibility | May depend on |
|---|---|---|
| `ume-formats` | INI, SFF, AIR, SND, ACT, PCX, PNG, FNT, stages, VFS (folders and zips) | nothing in the workspace; builds for `wasm32` |
| `ume-engine` | Fighter VM, expressions, triggers, controllers, HitDef, match rules, UMF compiler | `ume-formats` |
| `ume-netplay` | ggrs rollback, state checksum, replay format | `ume-engine` |
| `ume-render` | Sprite and post-processing pipelines on wgpu | wgpu only |
| `ume-shell` | Adapter, surface, window, input, `quad` draw API, audio out, frame pacing | `ume-render` |
| `ume-market` | GitHub marketplace: listing, bundles, safety limits | `ume-formats` |
| `ume-charforge`, `ume-devtools` | Authoring and audit tooling | formats, engine |
| `ultimate-mugen` (`src/`) | Screens, HUD, stages, VFX, audio mixing, settings, registry | everything above, GPU only through `ume_shell::quad` |

`scripts/crate-boundaries.mjs` fails CI when a runtime dependency breaks this table (for example `ume-render` depending on `winit`, or `ume-engine` depending on `wgpu`).

## Runtime flow

1. `ume-formats` parses character and stage files, or the native UMF JSON, into plain data.
2. `ume-engine` compiles CNS, ZSS and UMF into one bytecode and runs `Match::tick` at a fixed 60 Hz. It never reads a clock, a thread or the GPU.
3. `ume-netplay` clones and checksums `Match` for rollback and replays.
4. `src/` reads `Match` state each frame and draws it through `ume_shell::quad`, which batches into `ume-render`.

Determinism is the reason the simulation is single-threaded and allocation-free per tick. Loading and decoding may use threads through `ume_formats::par::map`.

## Where code goes

| You want to add | Put it in |
|---|---|
| A state controller or trigger | `crates/engine` (new file, one wiring line in `controllers.rs` or `triggers.rs`) and document it in `CNS-Scripts.md` |
| A file format or parser | `crates/formats` |
| A fighter, stage or route | Data only: UMF JSON, `.def`, TOML; no Rust |
| A screen or HUD element | `src/app/<screen>.rs` or the matching feature folder in `src/` |
| A GPU effect | `crates/render`, exposed through `ume_shell::quad` |
| A marketplace rule | `crates/market` |

## Extension points that need no code in the engine

- Native fighters: `assets/chars/*.json` ([UMF character](UMF-Character.md)).
- Stages: M.U.G.E.N `.def` or native stage data ([Stages](Stages.md)), story objects in TOML ([Stage objects](Stage-Objects.md)).
- Arcade routes and unlocks: TOML ([Route files](Route-Files.md)).
- Passives: data in a fighter ([Passives](Passives.md)).

## Keeping it maintainable

- One responsibility per file, 250-line ceiling, grandfathered files may not grow ([File layout](File-Layout.md)).
- A refactor that moves code is its own pull request and changes no behavior.
- Add an ADR when a decision changes a boundary, a dependency or a data format.
