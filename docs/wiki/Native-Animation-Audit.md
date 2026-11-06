# Native animation coverage

This delivery completes dedicated pose art for the native roster: every animation frame the engine can
name resolves to its own PNG file. The shared numbered canvases (`09_hit1`, `13_fall2`, ...) keep serving
their legacy aliases, and purely procedural effect frames (`boltHit`, beams) stay engine-rendered by
design; both are approved resolutions, not gaps.

## Included in this wave

- Dedicated `portrait.png` images for all sixteen native fighters remain in place for roster tiles and HUD portraits.
- Every string pose referenced by an animation now has a dedicated PNG in `assets/chars/sprites/<id>/`,
  including the guard, crouch, jump, flip, turn, land, win, intro, taunt, `df*` defeat set, and the
  previously idle-reusing low-hit poses. Inline pose objects have their `inline_<hash>` frame rendered once.
- The regeneration tool is `cargo run --release -p ume-engine --example generate_missing_sprites`. It merges
  `crates/engine/src/native/defaults.json` poses with each character's own table, renders missing poses
  through the rig, and writes 400x400 frames with axis (200, 392) and `refHeight` 320 metadata, matching the
  existing convention. It scans `assets/chars`, the `umf/` and `mugen/umf` scan folders and `UME_UMF_DIR`, so
  a newly created UMF character gets its sprite set with one command. Only plain-identifier ids and frame
  names are accepted, and the pass enforces per-character source-size and target-count caps, because the
  scanned UMF JSON is importable user content.
- Verification: `cargo run --release -p ume-engine --example native_sprite_audit -- <out.json> --require-complete`
  must pass for every fighter, and `python scripts/audit_native_sprites.py --out <report.json>` must stay clean
  on geometry and alpha edges. `python scripts/native_sprite_coverage.py` reproduces the resolution rules in
  Python without building the engine.

## sprite-gen QA loop

The [sprite-gen](https://github.com/aldegad/sprite-gen) skill remains the tool of record for art quality QA on
the native loops, with no image-generation credentials required:

- `python scripts/native_motion_descriptor.py --character assets/chars/<id>.json --sprites assets/chars/sprites/<id> --animation walk --out walk.json`
- `sprite-gen inspect-motion --source walk.json --out walk-report.json` measures anchors, duplicate frames,
  bbox drift and edge contact; foot contact needs explicit `--contacts` and `--foot-box` inputs.
- `scripts/preview_native_motion.py` and `scripts/preview_native_scene.py` render review GIFs and MP4s with the
  sprite-gen stages. See [Sprite Atlas Import](Sprite-Atlas-Import.md) for the component-row import path when
  AI-generated art is wanted.

## Known limits

Inventory and audit reports do not prove that the animations play naturally in a match: state triggers,
transition smoothness and hitbox fit must still be validated in Training. Recolors apply at load time through
the palette system, so visual spot checks should cover every palette of a fighter, not just the first one.
