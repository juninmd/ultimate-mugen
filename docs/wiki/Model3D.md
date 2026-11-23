# 3D fighters (prototype)

Status: isolated prototype, not wired into the game. Crate `crates/model3d` (`ume-model3d`); removing the workspace member and the crate directory reverts it completely.

## Principle

The simulation stays 2D. Hitboxes, physics, state machine and rollback never read the model. A 3D fighter is only a way to draw the same `stateno` and `statetime` the 2D fighter would use, so a 3D fighter can fight a 2D one.

## Pipeline

1. `Model::from_glb` parses a `.glb` (own parser over `serde_json`, no new dependency): nodes, skins, meshes, animations (linear and step; cubic spline is rejected).
2. `StateMap` (JSON) maps MUGEN state numbers (ranges) to clip name, loop flag and speed; unknown states use the fallback clip.
3. `Model::pose_for_state(map, stateno, statetime, &mut Posed)` samples the clip at `statetime / 60 * speed` and skins on the CPU. It is a pure function of its inputs, so it is rollback safe. `Posed` is reused between frames.
4. `Model::project` applies an orthographic camera on the fight plane (x right, y up, depth only sorts), flips for `Facing`, shades each face with one directional light and returns back-to-front `Triangle`s in screen pixels.

The triangles are meant to be pushed through `ume_shell::quad::draw_mesh` (per-vertex color, no texture), so no GPU pipeline change is needed and the web build keeps its single batch.

## State map

```json
{ "fallback": { "clip": "Survey" },
  "states": [ { "from": 20, "to": 21, "clip": "Walk", "loop": true, "speed": 1.0 } ] }
```

## Preview

`cargo run -p ume-model3d --example preview -- crates/model3d/tests/data/Fox.glb crates/model3d/tests/data/fox_states.json out.png`

## Limits and next steps

- No textures yet (base color factor or a default tint).
- Fixture `Fox.glb` is CC-BY 4.0 (see `crates/model3d/tests/data/ATTRIBUTION.md`) and is not shipped.
- Next: feature flag `model3d` (off by default) in the app, a `model` field in UMF, light mode falls back to 2D sprites, 3D stage background with parallax.
