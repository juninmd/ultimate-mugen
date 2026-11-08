# Compatibility decisions

This page records the features the engine deliberately does not implement, so character and stage authors get a clear answer instead of a silent failure.

## Module state controllers

The Ikemen module/motif layer is triaged as a group. None of these is implemented yet; the compatibility checker reports them as unsupported controllers.

| Controller | Decision | Reason |
|---|---|---|
| `Storyboard` | Planned | Needs the screenpack/storyboard runtime; tracked with the screenpack work |
| `Dialogue` | Planned | Same runtime as `Storyboard` |
| `MatchRestart` | Planned | Needs the rules surface and a deterministic restart path |
| `LifebarAction` | Planned | Needs the `fight.def` lifebar renderer |
| `ChangeMovelist` | Planned | Needs a per-match move list; the debug move list is read-only today |
| `LoadFile` | Rejected by design | Characters run inside a deterministic match; disk reads from scripts are not allowed |
| `SaveFile` | Rejected by design | Same sandbox rule as `LoadFile` |
| `SaveState` | Rejected by design | No rollback snapshot in the engine; replaying the input stream is the supported path |
| `LoadState` | Rejected by design | Same as `SaveState` |
| `ShaderSet` | Rejected by design | Shaders are part of the engine renderer, not character content |

The `REJECTED_CONTROLLERS` list in `crates/engine/src/diagnostics.rs` is used by the load report: these names are marked `(rejected by design)` in the detail output, while the planned ones are simply unsupported.

## 3D stages

glTF stages are **rejected by design**. The stage loader detects a model request (`[Model]`, `model = *.glb`, `[StageInfo] scene = ...`), warns once per stage and renders the normal 2D background layers. There is no fallback for stages that only contain a 3D model.

## Tracker music

`xm`, `it`, `mod` and `s3m` music is **rejected by design** for now. The audio backend decodes ogg/vorbis, mp3, flac and wav; an unsupported track logs a warning and the fight continues with the built-in music instead of going silent.

## Deliberate physics deviations

| Behaviour | M.U.G.E.N | Ultimate MUGEN | Reason |
|---|---|---|---|
| Airborne hit state with `physics = N` | No gravity: a fighter hit by repeated `yvel = 0` attacks stays in mid-air | The fighter falls, as if its state had ground-or-air gravity | Stops fighters from hovering while a projectile keeps hitting them. Attack states with `physics = N` (hover and flight moves) keep their scripted height. |

| Duplicate `[Begin Action N]` in an AIR file | First definition wins | First definition wins | Later copies used to overwrite the first and dropped its hitboxes. |
| `AnimTime` on an animation ending in a `-1` frame (M.U.G.E.N. characters) | 0 on that frame, negative before | Same | Taunts and one-shot states ending on a `-1` frame never finished. |
| Walk (20) and guard (130/131) exits | Engine hands control back when the direction is released | Same, even if the character's `common1.cns` has no exit | Characters were stuck walking or guarding. |
| Character-owned intro (states 190-199) | Round waits for the `intro` flag | Same; the engine face-off no longer replaces it | The intro was cut short. |
| Round start with a character that defines `Statedef 190` | The character's own intro state runs (its speech bubble, sounds, story variants) | Same; the engine enters it instead of its generic face-off pose, and skips its own speech bubble for M.U.G.E.N. characters | Characters' own intros and bubbles never ran and the generic bubble could talk over them. |
| `AfterImageTime`, `HitDef persistant`, `PlaySnd volume`, `Explod random/ownpal`, `ScreenBound movecamera`, `Projectile projshadow/air.juggle/projscreenbound` | Mostly accepted | Reported by the import audit as unknown parameters and ignored | Not implemented yet; the others in this row have no effect in M.U.G.E.N. either. |
