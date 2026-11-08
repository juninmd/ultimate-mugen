# Agents Guide - Ultimate MUGEN

## Overview
- 2D fighting game engine in Rust with no game framework (no macroquad, miniquad or Bevy): the sprite pipeline is purpose-built for M.U.G.E.N palette-indexed sprites.
- Rendering: own `wgpu` 30 renderer (`crates/render`) and own `winit` 0.30 shell (`crates/shell`). Palette lives in the fragment shader.
- Simulation: `crates/engine`, deterministic at 60 Hz, no clock, threads, randomness (outside `Match::random`) or GPU dependency.
- Formats: Native UMF (`assets/chars/*.json`) and legacy M.U.G.E.N (`.def`, `.sff`, `.air`, `.cmd`, `.cns`, `.snd`).
- Targets: Windows, Linux, macOS, `wasm32` (WebGL2 fallback) and Android; UI is immediate-mode on top of `ume_shell::quad`.

## Workspace
| Path | Responsibility | Constraint |
|---|---|---|
| `crates/formats` | INI, SFF, AIR, SND, ACT, PCX, PNG, FNT, stages, VFS (folders and zips) | No platform deps, must build for `wasm32` |
| `crates/engine` | Fighter VM, expressions, triggers, controllers, HitDef, match rules, UMF compiler | Deterministic, allocation-free tick, no window/GPU |
| `crates/render` | Sprite and post-processing pipelines (wgpu 30) | No winit, no app state |
| `crates/shell` | Adapter/device, surface, window, input, `quad` draw API, audio out, pacer, watchdog | Hosts `Game`; no game rules |
| `crates/netplay` | ggrs rollback, state checksum, replay format | Pure logic over `Match` |
| `crates/market` | GitHub marketplace: listing, bundles, safety limits | No UI |
| `src/` | Screens, HUD, stages, VFX, audio, settings, registry | Uses `ume_shell::quad`, never `wgpu` directly |

## Rendering and GPU
- One command encoder and one submit per frame. Sprites are batched into a single vertex/index buffer (grown by `next_power_of_two`) and drawn in one render pass.
- Two atlases (8-bit index and RGBA) plus a palette texture (`256 x rows`), each with independent reset, so filling one never forces the other to re-upload.
- Indexed sprites upload 1 byte per pixel; `PalFx` (mul/add/color/invert/hue) and palette swaps happen per vertex/in the shader, so `RemapPal` never bakes a new RGBA image.
- Vertices are 40 bytes (positions and UVs as `f32`, PalFx multipliers packed as `unorm16`); indices switch to 16-bit while the frame stays under 65536 vertices, so a sprite quad costs 172 bytes instead of 248.
- Atlas side is 4096 on desktop and 2048 on Android/iOS (80 MB versus 20 MB of VRAM); `UME_ATLAS=<side>` overrides it and the device limit always wins.
- Draw calls are split by blend state (alpha, add, sub) and scissor; pipeline switches are the main CPU cost of the batch loop.
- Post-processing (bright, separable blur, copy, composite) runs only when effects are enabled and reuses its targets; HUD glass reuses a quarter-resolution blurred copy.
- Adapter selection prefers the high-performance GPU and honours `WGPU_POWER_PREF`; vsync and frame cap are runtime toggles (`PresentMode::AutoVsync`/`AutoNoVsync`).

## Concurrency
- The simulation is single-threaded on purpose: `Match` stays `Clone` with a stable checksum so ggrs rollback and replays keep working. Never parallelize `Match::tick`.
- Loading and decoding fan out through `ume_formats::par::map` (`std::thread::scope`, work-stealing chunks, `min(cores, 8)` workers); UMF frame raster uses the same rule in `native/compile.rs`.
- Long jobs run on named threads: `char-load`, `char-preview`, `char-portrait`, `sfx-decode`, `music-synth`, `zip-import`, marketplace fetch.
- The render batch is built on the main thread; GPU uploads are budgeted by `Gfx::prefetch_step` (4 MiB per frame, up to 50% atlas).
- Audio: `symphonia` decodes, `quad-snd` plays; decode runs off the main thread.

## Performance rules
- No allocation inside `Match::tick`; `cargo test -p ume-engine --release --test tick_alloc` must stay green.
- No clocks, threads or `rand` in `crates/engine` outside `Match::random`.
- New draw paths must go through the atlas budget and must not upload the same pixels every frame (`Gfx` caches by key and palette).
- Thread helpers keep the 8-worker cap and the `wasm32` single-thread path.
- Prefer measuring over guessing: a change that cannot be shown in `tick_bench`, `mugen_load_bench`, the watchdog summary or the F3 overlay is not an optimization.
- Enable probes with `UME_TRACE=1` (desktop), `?perf=1` (web), `UME_WATCHDOG=1` and `UME_HITCH_MS=<ms>` (frame hitch log).

## Code Style
- No comments in code: names must explain intent. Explanations go to the wiki in `docs/wiki/`.
- English is the default language for code, commits, issues and docs. Portuguese lives in `*.pt-BR.md` files and `docs/wiki/pt-BR/`; keep both in sync.
- Update `docs/wiki/` whenever a format, controller, trigger, flag or folder layout changes.

## File Size and Layout
- Hard ceiling: **250 lines per source file**, checked by `node scripts/file-size.mjs` (Rust, JS, TS, CSS, HTML, WGSL). Content and docs (UMF JSON, `.cns`, `.air`, `.def`, atlas, `.md`) are out.
- One file, one responsibility: the file name states what it does, the folder names the feature, and `mod.rs` is a facade with declarations and re-exports only. Rules and examples: `docs/wiki/File-Layout.md`.
- Split by axis of change, not by size: a feature becomes `feature.rs` (shared types and entry points) plus `feature/<concern>.rs` (body, collision, palette, screen...), so two PRs working on the same feature touch different files.
- Existing files over the ceiling are grandfathered in `scripts/file-size-baseline.json`. They may not grow; split them and run `--update` in the same PR. Never raise a number in the baseline.
- Parallel PRs: put new logic in a new file and wire it with a single line in the shared file (`mod.rs`, `controllers.rs`, `triggers.rs`, `index.json`). No reformatting or renames of shared files inside a feature PR.
- No `utils.rs`, `helpers.rs`, `misc.rs`, `common.rs` or `manager.rs`: name the concept instead.

## Character Directories
- Built-in UMF: `assets/chars/` (embedded at compile time via `build.rs` and `index.json`).
- MUGEN Chars: `mugen/chars/<char_name>/` (auto-scanned by `scan_folders()` at startup, git-ignored).
- Bundled MUGEN packs: `assets/mugen/elecbyte.zip`.
- GitHub marketplace: logic in `crates/market` (listing, safety limits, bundles, downloads), app glue in `src/market.rs`, screen in `src/app/market.rs`, web storage in `web/market.js`. Docs: `docs/wiki/Marketplace.md`.

## External Resource Sources
- https://qxmugen.com/ (chars, stages, tools; downloads need login).
- https://mugenarchive.com/ (chars, stages; Cloudflare challenge, download manually).
- Agents never bypass login/challenges: user downloads into `mugen/incoming/`.

## Testing & Validation
- **Regra de ouro**: Baixar e descompactar todos os personagens primeiro; executar suites de teste (`cargo test`, `mugen_check`) **apenas uma vez ao final** da instalação completa.
- File size gate: `node scripts/file-size.mjs` (add `--update` only when a split removed a baseline entry).
- Engine compatibility: `cargo test --workspace --release`
- DBZ integration test: `cargo test -p ume-engine --test dbz -- --nocapture`
- Shonen Jump integration test: `cargo test -p ume-engine --test shonen_jump -- --nocapture`
- Headless match check: `cargo run --release -p ume-engine --example mugen_check -- <char_dir>...`
- Fetch/verify Jump chars: `powershell -ExecutionPolicy Bypass -File scripts/fetch-jump-chars.ps1 [-VerifyOnly]`
- Hover scan (fighters stuck off the floor): `cargo run --release -p ume-engine --example hover_check -- <char_dir>` (`TRACE=1` for transitions)
- Missing/blank sprite audit: `cargo run --release -p ume-engine --example sprite_audit -- <char_dir>`
- Strict native sprite coverage: `cargo run --release -p ume-engine --example native_sprite_audit -- <out.json> [--require-complete]`
- Native pose resolution mirror (no Rust build): `python scripts/native_sprite_coverage.py [--out report.json] [--targets]`
- Render dedicated PNGs for every native pose (inline poses, df* defeats, idle-reuse fixes; scans assets/chars + umf dirs): `cargo run --release -p ume-engine --example generate_missing_sprites [--dry-run]`
- Native PNG inventory (dimensions, alpha edges, geometry metadata): `python scripts/audit_native_sprites.py --out <report.json>`
- Sprite payload per fighter (sizes the atlas warm-up budget): `cargo run --release -p ume-engine --example sprite_mem -- <char_dir>...`
- Native roster lab (every attack state of every UMF fighter must connect; prints damage and reach, `UME_LAB_ONLY=id,id`, `UME_CHARS_DIR=<dir>`): `cargo test --release -p ume-engine --test native_roster_lab -- --nocapture`
- Declared cancel routes chain end to end: `cargo test --release -p ume-engine --test native_combo_routes -- --nocapture`; throws and armor: `--test native_throw_and_armor`
- Static roster audit (inputs, cancels, throws, hit boxes): `python scripts/audit_native_moves.py [dir]`
- CPU-vs-CPU balance duel (round-robin of every UMF fighter, win rate, damage, hits, knockout rate; `UME_DUEL_SKIP=oboro` leaves the boss out, `UME_DUEL_LEVEL`, `UME_DUEL_SEED_OFFSET`): `cargo run --release -p ume-engine --example native_duel -- assets/chars`
- Hit box audit (every strike frame of every UMF fighter: height, reach, start, width against the M.U.G.E.N-measured proportions): `cargo test --release -p ume-engine --test native_hitbox_audit -- --nocapture`; per-fighter normal reach and damage: `--test native_normal_reach`
- Whole-roster match invariants (every ordered pairing, two seeds: no fighter off camera, below the floor, hovering, out of control, in an endless attack, non-finite or out of life/power range): `cargo test --release -p ume-engine --test native_match_invariants -- --nocapture`
- Real-input move audit (every button command of every fighter is driven as joystick and button presses, standing, crouching and airborne, in every form; every ability state must be reachable): `cargo test --release -p ume-engine --test native_input_moves -- --nocapture`
- Stuck-state audit (no UMF fighter may idle 150 ticks inside one hit state across every CPU-vs-CPU pairing): `cargo test --release -p ume-engine --test native_no_stuck_states`
- Draw hit and hurt boxes on the move showcase: `UME_BOXES=1 UME_AFTER_HIT=0` with `UME_TOUR=moves`
- Move showcase screenshots (every special, super and throw against a dummy): `UME_TOUR=moves UME_CHARS=akio,bruno UME_RIVAL=kaito ultimate-mugen --shots <dir> --res 1280x720 --mute`
- Living stage layers on/off for A/B captures: `UME_AMBIENCE=0` disables them (`UME_SHOTS_ONLY=stages` captures all 21 stages)
- Zero-allocation tick gate: `cargo test -p ume-engine --release --test tick_alloc`
- M.U.G.E.N character load benchmark: `cargo run --release -p ume-engine --example mugen_load_bench [zip] [runs]`
- UMF compile benchmark: `cargo run --release -p ume-engine --example load_bench`
- Simulation tick benchmark: `cargo run --release -p ume-engine --example tick_bench -- [zip_or_char_dir] [seconds] [fighters_per_side]`
- Format fuzz & engine stress resilience: `cargo test --release -p ume-formats --test fuzz_resilience -p ume-engine --test stress_resilience`
- MUGEN visual probe: `UME_TOUR=mugen UME_CHARS=<id-suffix>,<id-suffix> ultimate-mugen --shots <dir> --res 1280x720 --mute`
- Pixel-art state probe (idle, transformation, hit, knockout, defeat of a native fighter that has `assets/chars/sprites/<id>/`): `UME_TOUR=pixel UME_CHARS=<fighter>,<opponent> ultimate-mugen --shots <dir> --res 1280x720 --mute`
