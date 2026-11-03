# Creating stages

There are two paths: a M.U.G.E.N stage (`.def` + `.sff` files, no programming) or a native stage (Rust code, with procedural layers and reactive effects).

## M.U.G.E.N stage

Put `my_stage.def` and `my_stage.sff` in `mugen/stages/`.

```ini
[Info]
name = "My Stage"
displayname = "My Stage"

[Camera]
startx = 0
starty = 0
boundleft = -150
boundright = 150
boundhigh = -25
boundlow = 0
verticalfollow = .2
tension = 50

[PlayerInfo]
p1startx = -70
p2startx = 70

[StageInfo]
zoffset = 210
localcoord = 320, 240

[BGdef]
spr = my_stage.sff

[BG Sky]
type = normal
spriteno = 0, 0
layerno = 0
start = 0, 0
delta = .2, .2
tile = 1, 0
velocity = .1, 0

[BG Floor]
type = normal
spriteno = 1, 0
start = 0, 160
delta = 1, 1
mask = 1

[BG Flag]
type = anim
actionno = 10
start = 80, 60
delta = .8, .8

[Begin Action 10]
2,0, 0,0, 6
2,1, 0,0, 6
```

| Section | Supported keys |
|---|---|
| `[Info]` | `name`, `displayname` |
| `[Camera]` | `startx`, `boundleft`, `boundright`, `boundhigh`, `boundlow`, `verticalfollow`, `tension` |
| `[PlayerInfo]` | `p1startx`, `p2startx` |
| `[StageInfo]` | `zoffset`, `localcoord` |
| `[BGdef]` | `spr` |
| `[BG ...]` | `type` (`normal`, `anim`), `spriteno`, `actionno`, `layerno` (0 behind, 1 in front), `start`, `delta`, `tile`, `tilespacing`, `velocity`, `trans`, `mask` |

Planned: `[BGCtrlDef]`/`[BGCtrl]`, `window`, `[Music]`, configurable shadows and reflections, `type = parallax`.

## Native stage

Native stages live in `src/stages.rs` and their animated effects in `src/stagefx.rs`.

1. Write a `fn mine() -> Stage` that builds layers with `lay(...)` and the primitives from `ume_engine::raster` (`rect`, `lin`, gradients) and `texgen` (`blobs`, `grain`, `haze`). Use `dojo()` as a model.
2. Register it in `builtin()`:

```rust
StageEntry { id: "mine".into(), name: "My Stage".into(), thumb: [[10, 10, 30], [40, 40, 90], [200, 120, 60]], build: Box::new(mine) },
```

3. For reactive effects (dust, crowds, water), add a `StageKind` variant and handle it in `stagefx.rs`.
4. Capture screenshots to review: `UME_SHOTS_ONLY=stages UME_STAGE=mine cargo run --release -- --shots out/`.

Each `Layer` has `delta` (parallax), `y`, `front` (drawn in front of the fighters) and `dynamic` (no bitmap, drawn by `stagefx`).
