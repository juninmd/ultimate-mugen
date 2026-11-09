# Dev tools

**English** · [Português (BR)](pt-BR/Ferramentas-de-Desenvolvimento.md)

Repository build and audit tools are Rust binaries, not scripts. `crates/devtools` builds `ume-tools`:

| Command | Replaces | What it does |
|---|---|---|
| `ume-tools pack-sprites [out] [--sprites dir]` | `scripts/pack_sprites.py` | Packs `assets/chars/sprites/<id>/` into `<out>/assets/chars/sprites/<id>.pack` for the web build (768 px cap, 256 colours, `meta.json` anchors rescaled, parallel). Run by `scripts/build-web.sh` |
| `ume-tools stage-thumbs [stages]` | `scripts/generate-stage-thumbnails.py` | Writes 240x135 Lanczos thumbnails to `<stages>/thumbs` |
| `ume-tools audit-moves [dir] [--json]` | `scripts/audit_native_moves.py` | Static roster audit: inputs, cancels, throws, hit boxes |
| `ume-tools audit-sprites --out report.json` | `scripts/audit_native_sprites.py` | Exact-file PNG inventory with alpha bounds |

Run them with `cargo run --release -p ume-devtools --bin ume-tools -- <command>`. Pose-resolution coverage is `native_sprite_audit --require-complete` (the removed `native_sprite_coverage.py` only mirrored it). Character creation tools live in `crates/charforge` ([Sprite toolkit](Sprite-Toolkit.md), [Move generator](Move-Generator.md)).

## Still Python

These scripts have not been ported: `scripts/gen_story_npcs.py` and `scripts/umf_edit.py` (need key-order-preserving JSON output), `scripts/native_motion_descriptor.py`, `scripts/preview_native_motion.py`, `scripts/preview_native_scene.py`, `scripts/ability_reel.py`, `scripts/build_motion_showcase.py` (captioned GIF/MP4 renderers over ffmpeg) and `tools/story_segments.py` (procedural stage art).
