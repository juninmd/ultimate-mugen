# Move Generator

`crates/charforge/` (`movekit` module) turns a TOML description of moves into MUGEN text: animations with automatic Clsn boxes, statedefs, command entries and a `[Statedef -1]` block. Edit the spec and regenerate instead of hand-editing the output.

```
cargo run --release -p ume-charforge -- moves moves.toml out/
cargo test --release -p ume-charforge
```

Writes `moves.air`, `moves.cns`, `moves.cmd` and `statedef-1.cns`. A full example is in `crates/charforge/examples/moves.toml`; it expects sprites sliced by [Sprite toolkit](Sprite-Toolkit.md) at `sprites/manifest.json`.

## Spec

| Key | Meaning |
|---|---|
| `[defaults]` | `sprite_manifests`, `clsn1_height` (standing height, grows the attack box), `hitsound`, `guardsound`, `super_sound`, `voice_channel` |
| `kind` | `normal`, `crouch`, `air`, `special`, `air_special`, `super`, `projectile`, `throw` (sets damage, pause, power and attribute defaults) |
| `state`, `anim` | State number and animation number (default: the state) |
| `command`, `command_name`, `command_time` | MUGEN input notation; default name `spec_<state>` |
| `frames` | `[group, item, ticks, "hit"?, {x, y, flip, trans, clsn1, clsn2}?]`; `"hit"` adds an automatic Clsn1 |
| `hit` | HitDef list; `elem` or `elems` choose the animation elements (default: first frame with Clsn1); `extra` adds raw parameters |
| `sounds`, `vel`, `vfx`, `helpers`, `cancel`, `extra`, `voice` | PlaySnd, velocity, Explod, Helper, cancel and raw controllers |
| `power`, `superpause` | Power cost and SuperPause for supers |
| `phases` | Extra chained states with `name`, `next`, `like`; the move is the first state |

Generated entries in `[Statedef -1]` are ordered super, special, throw, normal and gated by `roundstate = 2`, state type and power.

## Validating

`cargo run --release -p ume-charforge -- check <char_dir> --strict` loads the character with the engine loader and prints the same diagnostics as `mugen_check`; `--strict` fails on unsupported controllers, triggers or parameters.

## Next candidates from CharFactory

A tick-by-tick move simulator for frame data, a combo tuner, palette range tools and a contact-sheet renderer. Not ported yet.

## Acknowledgments

Approach inspired by [CharFactory](https://github.com/eduardovbe/CharFactory) (MIT, Copyright (c) 2026 eduardovbe). Fresh implementation; see `REFERENCES.md`.
