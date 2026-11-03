# Agents Guide - Ultimate MUGEN

## Overview
- Engine: 2D fighting game engine written in Rust using `macroquad`, `miniquad`, `tiny-skia`.
- Formats: Native UMF (`assets/chars/*.json`) and legacy M.U.G.E.N (`.def`, `.sff`, `.air`, `.cmd`, `.cns`, `.snd`).

## Character Directories
- Built-in UMF: `assets/chars/` (embedded at compile time via `build.rs` and `index.json`).
- MUGEN Chars: `mugen/chars/<char_name>/` (auto-scanned by `scan_folders()` at startup, git-ignored).
- Bundled MUGEN packs: `assets/mugen/elecbyte.zip`.

## Testing & Validation
- **Regra de ouro**: Baixar e descompactar todos os personagens primeiro; executar suites de teste (`cargo test`, `mugen_check`) **apenas uma vez ao final** da instalação completa.
- Engine compatibility: `cargo test --workspace --release`
- DBZ integration test: `cargo test -p ume-engine --test dbz -- --nocapture`
- Shonen Jump integration test: `cargo test -p ume-engine --test shonen_jump -- --nocapture`
- Headless match check: `cargo run --release -p ume-engine --example mugen_check -- <char_dir>...`
- Fetch/verify Jump chars: `powershell -ExecutionPolicy Bypass -File scripts/fetch-jump-chars.ps1 [-VerifyOnly]`
- Hover scan (fighters stuck off the floor): `cargo run --release -p ume-engine --example hover_check -- <char_dir>` (`TRACE=1` for transitions)
- Missing/blank sprite audit: `cargo run --release -p ume-engine --example sprite_audit -- <char_dir>`
- MUGEN visual probe: `UME_TOUR=mugen UME_CHARS=<id-suffix>,<id-suffix> ultimate-mugen --shots <dir> --res 1280x720 --mute`
