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

Only `tools/story_segments.py` remains: it draws the story stage strips with Python's random generator and Pillow rasterizers, and the committed stage art must stay byte-identical, so it is not ported. Everything else is `ume-tools`: `gen-story-npcs` (byte-identical to the old script, but it overwrites drifted committed NPC files), `motion-descriptor`, `preview-motion`, `preview-scene`, `ability-reel` (ffmpeg; captions use a built-in bitmap font) and the `umf_edit` library (key-order and line-ending preserving JSON edits). The one-off motion showcase image script was dropped; its output stays committed.
