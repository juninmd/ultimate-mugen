# Agents Guide - Ultimate MUGEN

## Overview
- Engine: 2D fighting game engine written in Rust using `macroquad`, `miniquad`, `tiny-skia`.
- Formats: Native UMF (`assets/chars/*.json`) and legacy M.U.G.E.N (`.def`, `.sff`, `.air`, `.cmd`, `.cns`, `.snd`).

## Character Directories
- Built-in UMF: `assets/chars/` (embedded at compile time via `build.rs` and `index.json`).
- MUGEN Chars: `mugen/chars/<char_name>/` (auto-scanned by `scan_folders()` at startup, git-ignored).
- Bundled MUGEN packs: `assets/mugen/elecbyte.zip` and `assets/mugen/community.zip`.

## Testing & Validation
- Engine compatibility: `cargo test --workspace --release`
- DBZ integration test: `cargo test -p ume-engine --test dbz -- --nocapture`
- Shonen Jump integration test: `cargo test -p ume-engine --test shonen_jump -- --nocapture`
- Headless match check: `cargo run --release -p ume-engine --example mugen_check -- <char_dir>...`
- Fetch/verify Jump chars: `powershell -ExecutionPolicy Bypass -File scripts/fetch-jump-chars.ps1 [-VerifyOnly]`
