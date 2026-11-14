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
- Dedicated PNGs must be generated in arcade pixel art via `generate_image` using a validated canonical anchor of the fighter (the deprecated vector-rig `generate_missing_sprites` tool has been removed). Write 400×400 RGBA frames with axis (200, 392) and `refHeight` 320 metadata, matching the existing convention.
- Verification: `cargo run --release -p ume-engine --example native_sprite_audit -- <out.json> --require-complete`
  must pass for every fighter, and `cargo run --release -p ume-devtools --bin ume-tools -- audit-sprites --out <report.json>` must stay clean
  on geometry and alpha edges. `cargo run --release -p ume-devtools --bin ume-tools -- audit-moves` is the static roster audit (inputs, cancels, throws, hit boxes).

## sprite-gen QA loop

The [sprite-gen](https://github.com/aldegad/sprite-gen) skill remains the tool of record for art quality QA on
the native loops, with no image-generation credentials required:

- `ume-tools motion-descriptor --character assets/chars/<id>.json --sprites assets/chars/sprites/<id> --animation walk --out walk.json`
- `sprite-gen inspect-motion --source walk.json --out walk-report.json` measures anchors, duplicate frames,
  bbox drift and edge contact; foot contact needs explicit `--contacts` and `--foot-box` inputs.
- `ume-tools preview-motion` and `ume-tools preview-scene` render review GIFs and MP4s with the
  sprite-gen stages. See [Sprite Atlas Import](Sprite-Atlas-Import.md) for the component-row import path when
  AI-generated art is wanted.

## Known limits

Inventory and audit reports do not prove that the animations play naturally in a match: state triggers,
transition smoothness and hitbox fit must still be validated in Training. Recolors apply at load time through
the palette system, so visual spot checks should cover every palette of a fighter, not just the first one.
