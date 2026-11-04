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
