# Customizing the engine

## Architecture

| Path | Responsibility |
|---|---|
| `crates/formats` | M.U.G.E.N file readers: INI, SFF, AIR, SND, ACT, PCX, PNG, stages, virtual file system (folders and zips). No platform dependencies |
| `crates/engine` | State machine: expressions (`expr.rs`), triggers (`triggers.rs`), controllers (`controllers.rs`), HitDef (`hitdef.rs`), fight (`matchs.rs`), commands and input, UMF compiler (`native/`) |
| `src/` | Application: window, render, HUD, audio, menus, native stages, visual effects, controls and gamepad |
| `web/` | Page, service worker and the browser JS bridge |
| `assets/chars` | Bundled UMF fighters |

Tick flow: inputs → `Match::tick` → for each fighter, `[Statedef -3/-2/-1]` and the current state → controllers whose triggers are true → physics → collision and `apply_hit` → camera. Simulation runs at 60 ticks per second, is deterministic and independent from rendering.

## Add a state controller

1. In `crates/engine/src/controllers.rs`, inside `run`, add a branch with the lowercase name:

```rust
"victoryquote" => { let v = n(m, "value", -1.0) as i32; m.fighters[i].victory_quote = v; }
```

`victory_quote` is a new field you add to `Fighter`.

2. `n(m, "param", default)` reads a numeric parameter; `n2(m, "param", default, index)` reads the Nth value; `sc.has("param", 0)` tests presence; `sc.raw("param")` returns the text.
3. Add the field in `fighter.rs` if you need new state.
4. Write a test in `crates/engine/tests/combat_logic.rs` calling `ume_engine::controllers::run`.
5. List the name in `CNS-Scripts.md` and `pt-BR/Scripts-CNS.md`. `cargo test -p ume-engine --test wiki_sync` fails for an implemented controller that is not listed and for a listed one that is not implemented. Return `true` from the module's `run` (or fall through to the last branch) so the load diagnostics do not report it as unsupported.

## Add a trigger

In `crates/engine/src/triggers.rs`, add a branch to the name `match`:

```rust
"winko" => b(m.match_over && m.winner_side == f.side as i32 && m.won_by_ko),
```

`f` is the evaluated fighter and `m` the match. `won_by_ko` would be a new field on `Match`. `b(bool)` converts to 0/1. Triggers returning floats must appear in `is_float` in `expr.rs`. List the new name in `CNS-Scripts.md` and `pt-BR/Scripts-CNS.md`; `wiki_sync` checks both directions.

## Add a file format

Put the decoder in `crates/formats/src/<format>.rs`, export it in `lib.rs` and test it with real files in `crates/formats/tests/real_files.rs`. The crate must keep compiling for `wasm32-unknown-unknown`.

## Tests

| Command | Covers |
|---|---|
| `cargo test --workspace --release` | Everything |
| `cargo test -p ume-engine --test kfm` | Reference Kung Fu Man |
| `cargo test -p ume-engine --test combat_logic` | Combat rules |
| `cargo test -p ume-formats` | Decoders |
| `cargo clippy --workspace -- -D warnings` | Lint |
| `node scripts/web-smoke.js web/dist` | Web build in Chromium |
| `node scripts/web-match-smoke.js web/dist` | Full match in Chromium: inputs, rounds, no freeze, results |

## Code rules

- No code comments: names of functions, types and variables must explain intent. Explanations live in this wiki.
- Code, commits, issues and documentation are written in English. Portuguese translations live in `*.pt-BR.md` files and `docs/wiki/pt-BR/`.
- The simulation (`crates/engine`) uses no clock, threads or randomness outside `Match::random`, to keep replays and rollback working.
- No allocation inside `Match::tick` in new code.
- Files stay under 250 lines with one responsibility each, split by axis of change: see [File layout](File-Layout.md) and `node scripts/file-size.mjs`.
- Conventional Commits (`feat:`, `fix:`), validated by `scripts/commit-lint.mjs`.

## Architecture decisions

| Topic | Choice |
|---|---|
| Render | `wgpu` (Vulkan, Metal, DX12, WebGPU, WebGL2), palette in the shader |
| Scripts | CNS and ZSS for fighters, UMF JSON for the native format; no Lua |
| Expressions | Stack bytecode |
| Audio | `kira` + `symphonia` |
| Network | `ggrs` (rollback) + `matchbox` (WebRTC) |

The migration is described in [`docs/ROADMAP.md`](../ROADMAP.md).

## State ownership and control

A fighter runs a state from the definition that gave it: its own `.cns`, the shared `common1.cns`, or another character's definition (`TargetState`, `p2stateno`, a throw). Three rules keep control flowing:

- A state number the character defines always runs from the character's own definition, so returning from an engine state (face-off, awakening) or from a `common1` state hands control back instead of leaving the fighter owned by the shared file.
- States from `common1.cns` are the player's own: the character's `-1` states and the engine's walk/jump/crouch transitions keep running during a get-hit, lie-down or any other shared state — M.U.G.E.N. characters cancel hits from `-1` that way.
- Only a custom state given by the opponent pauses them, because the giver's script owns the fighter until it hands control back.

A fighter whose state owner is not itself and not `common1` is "in a custom state"; the helpers that answer this are the guard for every input-driven transition, so losing it makes the character stand still while still reading as alive (`ctrl`, state 0).

## HUD glass and impact effects

- The in-fight HUD panels are "glass": `ume_shell::quad::glass_panel` registers up to 16 rounded rectangles per frame, and the post-processing composite pass replaces those pixels with a blurred copy of the scene (quarter resolution, separable blur, no per-frame allocations), plus tint, drop shadow and a rim highlight. With post-processing off (`fx` level 0) the HUD falls back to translucent polygons.
- Impact feedback is deliberately small: guard and counter share the same small label and ring, super arts only get a light ring burst and a compact label, and the number of simultaneous floats, rings, lights and particles is capped in `src/vfx.rs`.
