# 3D fighters

Status: shipped behind the Cargo feature `model3d`, on by default. Rollback: remove `model3d` from `default` in the root `Cargo.toml`; the game then draws the 2D donor sprites and hides the 3D category.

## Principle

The simulation stays 2D. Hitboxes, physics, state machine and rollback never read the model. A 3D fighter is a UMF fighter (`assets/chars3d/<id>.json`, usually `extends` a native kit) whose drawing is replaced by a GLB model, so a 3D fighter fights a 2D one with the same rules. Light mode (`lite::scale() < 1`) falls back to the 2D sprites.

## Pipeline

1. `Model::from_glb` parses a `.glb` (own parser over `serde_json`, PNG textures through `png`): nodes, skins, meshes, materials, animations.
2. `StateMap` (JSON) maps native anim names (`anims`) or MUGEN state ranges (`states`) to a clip, loop flag and speed.
3. The pose is a pure function of `anim_no`/`anim_elapsed` (or `stateno`/`statetime`), skinned on the CPU, so it is rollback safe.
4. `Model::project` uses an orthographic camera on the fight plane, flat Lambert shading and back-to-front triangles, drawn through `ume_shell::quad::draw_mesh` batched by texture. No GPU pipeline change.

## In the game

- `src/model3d.rs` is the facade (`draw`, `draw_fit`, `is_3d`, `portrait`); `src/model3d/specs.rs` lists the models.
- Fight: one line in `render.rs`. Select, versus and results panels: `draw_fit` inside `draw_char_fit`. Tiles and loading use `portrait`.
- The select screen has a `3D` category.
- The awakened form swaps to the second model (Ichigo: Hollow Total).

## Asset tools

`tools/retarget` (Python, stdlib only):
- `retarget.py <clips dir> <source.glb> <out.glb> <rig>` bakes the clips listed in `clips.json` onto a rig from `rigs.json`.
- `autorig.py <mesh.glb> <clips dir> <out.glb>` fits a skeleton and skin weights to an unrigged mesh.
- Portrait: `cargo run -p ume-model3d --example portrait -- <model.glb> <states.json> <out.png>`.
- Voices: `tools/voices/generate.py <id>` adds the fighter to `voices.bank` and keeps the other banks.

## Ichigo moves

Jumps: `D, U` or `U, U` super jump (also in the air), two air jumps, three air dashes. Specials: Getsuga Tenshou, Shunpo, Shunpo Rush, Hollow Cero, Getsuga Juujishou, Kuro Getsuga, Hollow Rampage; Hollow Total adds Cero Oscuras.

## Credits

See `assets/model3d/ATTRIBUTION.md`.
