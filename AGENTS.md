# Agents Guide - Ultimate MUGEN

## Overview
- Engine: 2D fighting game engine written in Rust using `macroquad`, `miniquad`, `tiny-skia`.
- Formats: Native UMF (`assets/chars/*.json`) and legacy M.U.G.E.N (`.def`, `.sff`, `.air`, `.cmd`, `.cns`, `.snd`).

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
- Simulation tick benchmark: `cargo run --release -p ume-engine --example tick_bench -- [zip_or_char_dir] [seconds] [fighters_per_side]`
- Format fuzz & engine stress resilience: `cargo test --release -p ume-formats --test fuzz_resilience -p ume-engine --test stress_resilience`
- MUGEN visual probe: `UME_TOUR=mugen UME_CHARS=<id-suffix>,<id-suffix> ultimate-mugen --shots <dir> --res 1280x720 --mute`
- Pixel-art state probe (idle, transformation, hit, knockout, defeat of a native fighter that has `assets/chars/sprites/<id>/`): `UME_TOUR=pixel UME_CHARS=<fighter>,<opponent> ultimate-mugen --shots <dir> --res 1280x720 --mute`
