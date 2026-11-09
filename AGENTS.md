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

## Verification
- Every change gets a browser smoke test: `scripts/build-web.sh`, serve `web/dist`, drive it in headless Chromium with Playwright (hold keys at least 100 ms), and look at screenshots.
- Screenshot tours: `UME_SHOTS_ONLY=pause|hud|menu xvfb-run -a target/release/ultimate-mugen --shots <dir> --res 1280x720 --mute`.

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

## CI and Release
- Jobs in `.github/workflows/ci.yml`: `release-scripts` (`node --test scripts/*.test.mjs`, `node scripts/file-size.mjs`), `test` (`cargo test --workspace --release` and `mugen_check --strict` over `crates/engine/tests/data/chars`), `wasm` (`cargo build --release --target wasm32-unknown-unknown`), `web-smoke` (`scripts/web-smoke.js` and `scripts/web-story-smoke.js` in Playwright 1.56.1), `android` (`cargo apk`, throwaway keystore), `ios` (`aarch64-apple-ios` build) and `desktop` (3 OSes). `commit-lint.yml` runs `scripts/commit-lint.mjs`.
- PR title (squash flow) or every commit (merge flow) must be `type(scope)?(!)?: description`, type in `feat|fix|perf|refactor|docs|test|build|ci|chore|style|revert`. A title without a type fails `commit-lint`.
- Releases are automatic on every push to `master` (`release.yml`, `scripts/release-plan.mjs`): `feat` is minor, `fix`/`perf` is patch, `!` or `BREAKING CHANGE:` is major; `docs`/`chore`/`ci`/`test`/`refactor`/`style`/`build` alone cut no release. The git tag is the version source of truth; the build is stamped through `UME_RELEASE_VERSION`, built on Linux, Windows and macOS, verified as a draft and then published. The desktop build self-updates from `latest.json` (`src/update/`, `docs/wiki/Auto-Update.md`).
- Web deploys to https://juninmd.github.io/ultimate-mugen/ (`pages.yml`) on every `master` merge.
- Merge policy: Antonio wants PRs merged once CI is green (repo auto-merge is off, so merge manually). Base branch is `master`.

## Pitfalls
- Never run `cargo fmt --all`: master is not rustfmt-clean and it rewrites hundreds of unrelated files. Format only new files with `rustfmt --edition 2021 <file>`.
- The 250-line ceiling and the baseline: files like `render.rs`, `app.rs`, `session.rs`, `i18n.rs` are grandfathered and may not grow by a single line. Put new code in a new file and wire it with one line; remove a baseline entry by hand once a file drops under 250 instead of running `--update` (it rewrites unrelated entries).
- Cloud container build deps: `libudev-dev libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev pkg-config`.
- Importing a character must go through `Registry::import_zip` or `add_vfs_lazy` (identity is `CharEntry.key`/`StageEntry.key` from `ume_formats::vfs::{char_key, stage_key}`); never push entries directly, or duplicates return.
- Flaky unrelated test: `preview_is_light_and_art_res_scales` (global art_res race); rerun it.
- The user recreates character and stage art elsewhere: do not touch native character sprites or stages unless explicitly asked (non-art data and new story stage extensions are fine).
- Never write that the engine will "beat" or "surpass" Ikemen GO; only credit it (and Elecbyte) as inspiration.
- Never bypass logins or Cloudflare challenges to fetch chars.

## Localization
- `src/i18n.rs` plus `src/i18n/`: keyed catalogs `catalog.rs` (`EN`, `PT`) and `es_catalog.rs`/`es_phrases.rs` (`ES`), `Lang::{Pt,En,Es}`. Lookup is current locale, then English, then the key (a missing key is logged once). The system language is detected in `i18n/system.rs`. Every new UI string needs en, pt and es entries; `src/i18n/tests.rs` guards the catalogs.

## Native Fighter Data
- Passives are data in `passives` of a UMF fighter: `PassiveSpec` and its parts in `crates/engine/src/native/passive_spec.rs`, runtime rules in `crates/engine/src/passives.rs` and `passive_time.rs`. Fields: `meterRegen`, `lifeRegen` (compiled to `[Statedef -2]` by `compile_extras.rs::passives_cns`), `rage` (`maxMultiplier`, `exponent`, `startRatio`, `fullRatio`), `stacks` (`damageGain`, `defenceGain`, `speedGain`, `maxStacks`, `decayTicks`, `resetOnRound`, `specials`, `supers`) and `enrageTime` (`maxMultiplier`, `startSeconds`, `rampSeconds`, `exponent`; damage grows with fight time).
- Hard ceilings in `passives.rs`: `TOTAL_ATTACK_CAP` 8x total damage, `RAGE_MULTIPLIER_CAP` 6x per rule, `STACK_DEFENCE_CAP` 3x, `STACK_SPEED_CAP` 1.5x, `STACK_LIMIT` 30 stacks. Data cannot exceed them; tests are `tests/passives.rs` and `tests/passive_time.rs`.
- Web MUGEN import is sliced: `ume_formats::vfs::zip_job::ZipJob::step(budget)` inflates a bounded amount per frame so the browser paints real progress and the user can cancel (`src/app/web_import.rs`, `src/webimport.rs`). Keep any new import work incremental on `wasm32`.
- Atlas side is 4096 on desktop/web and 2048 on Android/iOS, clamped by the device limit (`crates/shell/src/platform.rs`).

## Evidence Conventions
- Screenshots and reports for a PR go to `/mnt/project-files/<topic>/` (for example `hud-glass/`, `menu/`, `mobile/`, `import-fix/`, `mugen-gogeta/`, `mugen-goku/`), with before/after images named `antes-depois`, and are attached to the thread reply and the PR.
- MUGEN compatibility QA uses real characters: frames per command in `frames/`, a `commands.md` and a `REPORT.md`/`engine/*.txt` audit per character.

## Character Directories
- Built-in UMF: `assets/chars/` (embedded at compile time via `build.rs` and `index.json`).
- MUGEN Chars: `mugen/chars/<char_name>/` (auto-scanned by `scan_folders()` at startup, git-ignored).
- Bundled MUGEN packs: `assets/mugen/elecbyte.zip`.
- GitHub marketplace: logic in `crates/market` (listing, safety limits, bundles, downloads), app glue in `src/market.rs`, screen in `src/app/market.rs`, web storage in `web/market.js`. Docs: `docs/wiki/Marketplace.md`.

## Repository Map
- `docs/wiki/` (EN) and `docs/wiki/pt-BR/` (PT): the wiki, indexed by `Home.md`. `docs/assets/` holds README media; `docs/evidence/` and `docs/research/` hold audit output.
- `.agents/skills/`: agent skills (`mugen-character-creator`, `ume-character-creator`, `ume-stage-creator`). One copy only; do not duplicate them elsewhere.
- `scripts/`: build, audit and capture tools; `tools/`: asset generators.

## External Resource Sources
- https://qxmugen.com/ (chars, stages, tools; downloads need login).
- https://mugenarchive.com/ (chars, stages; Cloudflare challenge, download manually).
- Agents never bypass login/challenges: user downloads into `mugen/incoming/`.

## Testing & Validation
- **Regra de ouro**: Baixar e descompactar todos os personagens primeiro; executar suites de teste (`cargo test`, `mugen_check`) **apenas uma vez ao final** da instalação completa.
- **Browser smoke test (always)**: every change that can affect gameplay, rendering, loading, input or the web build must be checked in a real browser before the PR is opened, not only with `cargo test`. Build the wasm bundle and drive it in headless Chromium: `rustup target add wasm32-unknown-unknown`, `cargo install wasm-bindgen-cli --locked --version <version in Cargo.lock>`, `./scripts/build-web.sh`, then `NODE_PATH=$(npm root -g) WEB_SMOKE_SHOTS=<dir> node scripts/web-smoke.js web/dist` (title, menu, select, stage and fight must finish with `WEB SMOKE OK`; look at the screenshots, a black or frozen frame is a failure). For a M.U.G.E.N. character, also import its zip through `#ume-file`, pick it on the select screen, fight and attach the screenshots to the PR. CI runs the same script as `web-smoke`.
- Capture README media with `CAPTURE_OUT=<dir> node scripts/web-capture.js <plan.json>` (see `docs/wiki/Media-Capture.md`).
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
- Native fighter boxes rule: hit boxes and hurt boxes only exist where the rendered sprite has opaque pixels. Every box edge must touch visible pixels, and each strike or body pose must be split into at least 3 smaller boxes that follow the limb or body shape (`crates/engine/src/native/pixel_fit.rs` derives them from the sprite alpha, offset, axis and scale). Never hand-widen a box into empty space and never edit sprite images to fix a box; enforced by `cargo test --release -p ume-engine --test native_box_sprite_fit -- --nocapture`. Review with `UME_BOXES=1` or `cargo run --release -p ume-engine --example box_overlay -- <fighter> <out.png>`.
- Hit box audit (every strike frame of every UMF fighter: height, reach, start, width against the M.U.G.E.N-measured proportions): `cargo test --release -p ume-engine --test native_hitbox_audit -- --nocapture`; per-fighter normal reach and damage: `--test native_normal_reach`
- Whole-roster match invariants (every ordered pairing, two seeds: no fighter off camera, below the floor, hovering, out of control, in an endless attack, non-finite or out of life/power range): `cargo test --release -p ume-engine --test native_match_invariants -- --nocapture`
- Real-input move audit (every button command of every fighter is driven as joystick and button presses, standing, crouching and airborne, in every form; every ability state must be reachable): `cargo test --release -p ume-engine --test native_input_moves -- --nocapture`
- Air abilities (every special and super has an air version that must connect, land and give control back; the real-input audit reaches them too; air clips: `UME_CLIP_AIR=1` with `UME_TOUR=clips`): `cargo test --release -p ume-engine --test native_air_abilities -- --nocapture`
- Energy charge (every fighter gains meter while `R` is held, stops on release or a full meter, a hit breaks it, forms still transform): `cargo test --release -p ume-engine --test native_energy_charge`
- Ability reels (a GIF per fighter: every special, super, throw, movement ability, the energy charge and the transformation, plus damage and power printed per clip; `UME_STAGE=plains UME_AMBIENCE=0` keeps the GIFs small): `UME_TOUR=clips UME_CHARS=akio,jin ultimate-mugen --shots <dir> --res 1280x720 --mute` then `python scripts/ability_reel.py <dir> <out> 336 36 12 --no-normals`
- Story mode run (plays all 5 stages with a CPU-driven hero, saves every intro, wave, boss and result screen; thugs are `assets/chars/npc_*.json`, regenerate with `python scripts/gen_story_npcs.py`): `UME_TOUR=storymode UME_AMBIENCE=0 ultimate-mugen --shots <dir> --res 1280x720 --mute`; thug traits and palettes: `cargo test --release -p ume-engine --test story_npcs`
- Stuck-state audit (no UMF fighter may idle 150 ticks inside one hit state across every CPU-vs-CPU pairing): `cargo test --release -p ume-engine --test native_no_stuck_states`
- Draw hit and hurt boxes on the move showcase: `UME_BOXES=1 UME_AFTER_HIT=0` with `UME_TOUR=moves`
- Move showcase screenshots (every special, super and throw against a dummy): `UME_TOUR=moves UME_CHARS=akio,bruno UME_RIVAL=kaito ultimate-mugen --shots <dir> --res 1280x720 --mute`
- Live CPU-vs-CPU fight capture in the real game (frames every `UME_DUEL_STRIDE` ticks, `UME_BOXES=1` draws hit/hurt boxes, `UME_DUEL_SKIP`, `UME_DUEL_FRAMES`, `UME_STAGE`): `UME_TOUR=duel UME_CHARS=akio,jin ultimate-mugen --shots <dir> --res 1280x720 --mute`
- Living stage layers on/off for A/B captures: `UME_AMBIENCE=0` disables them (`UME_SHOTS_ONLY=stages` captures all 21 stages)
- Zero-allocation tick gate: `cargo test -p ume-engine --release --test tick_alloc`
- M.U.G.E.N character load benchmark: `cargo run --release -p ume-engine --example mugen_load_bench [zip] [runs]`
- UMF compile benchmark: `cargo run --release -p ume-engine --example load_bench`
- Native roster tick benchmark (every UMF pair, only `Match::tick` timed): `cargo run --release -p ume-engine --example native_tick_bench -- assets/chars`
- Simulation tick benchmark: `cargo run --release -p ume-engine --example tick_bench -- [zip_or_char_dir] [seconds] [fighters_per_side]`
- Format fuzz & engine stress resilience: `cargo test --release -p ume-formats --test fuzz_resilience -p ume-engine --test stress_resilience`
- MUGEN visual probe: `UME_TOUR=mugen UME_CHARS=<id-suffix>,<id-suffix> ultimate-mugen --shots <dir> --res 1280x720 --mute`
- Pixel-art state probe (idle, transformation, hit, knockout, defeat of a native fighter that has `assets/chars/sprites/<id>/`): `UME_TOUR=pixel UME_CHARS=<fighter>,<opponent> ultimate-mugen --shots <dir> --res 1280x720 --mute`
