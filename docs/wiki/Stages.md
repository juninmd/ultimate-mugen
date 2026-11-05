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
| `[BG ...]` | `type` (`normal`, `anim`), `spriteno`, `actionno`, `layerno` (0 behind, 1 in front), `start`, `delta`, `tile`, `tilespacing`, `velocity`, `trans`, `mask`, `scalestart`, `scaledelta`, `angle`, `xshear`, `window`, `maskwindow`, `windowdelta`, `spacing` |

Layer transform parameters (Ikemen / M.U.G.E.N 1.1), applied by both the live renderer and the stage-select preview:

| Key | Meaning |
|---|---|
| `scalestart = x, y` | Base scale of the sprite, anchored at its axis (default `1, 1`). |
| `scaledelta = x, y` | Scale change per stage unit of camera movement, multiplied by `delta`: `scale = scalestart + scaledelta * camera * delta`, so the layer grows or shrinks as the camera moves (default `0, 0`). |
| `angle` | Rotation in degrees around the sprite axis, counter-clockwise for positive values. |
| `xshear` | Horizontal shear: each row is shifted by `xshear` times its distance from the axis. |
| `window = x1, y1, x2, y2` | Clipping rectangle in stage units. |
| `maskwindow = x1, y1, x2, y2` | Second clipping rectangle; when both are present the layer is clipped to their intersection. |
| `windowdelta = x, y` | How fast the clipping window scrolls with the camera (defaults to the layer `delta`). |
| `spacing = x, y` | Gap between tiles; used when `tilespacing` is absent (`tilespacing` wins when both are set). |

Planned: `[Music]`, configurable shadows and reflections, `type = parallax`.

## Reading the stage from scripts

`StageVar(<name>)` returns the stage value in the stage's own `localcoord` units. The section prefix is optional: `StageVar(camera.boundleft)` and `StageVar(boundleft)` are the same query.

| Name | Value |
| --- | --- |
| `info.name`, `info.displayname`, `info.author`, `info.mugenversion` | text, compared with `StageVar(info.author) = "name"` (case-insensitive) |
| `camera.startx`, `camera.starty`, `camera.boundleft`, `camera.boundright`, `camera.boundhigh`, `camera.boundlow` | numbers |
| `camera.verticalfollow`, `camera.tension`, `camera.zoomin`, `camera.zoomout` | numbers |
| `playerinfo.p1startx`, `playerinfo.p2startx`, `playerinfo.leftbound`, `playerinfo.rightbound` | numbers |
| `playerinfo.autoturn` | 1 unless the stage sets `autoturn = 0` |
| `stageinfo.zoffset`, `stageinfo.localcoord.x`, `stageinfo.localcoord.y`, `stageinfo.roundtime` | numbers (`localcoord` alone means the width; `roundtime` is -1 when the stage does not set it) |
| `shadow.intensity`, `shadow.yscale`, `reflection.intensity` | numbers |

Text queries evaluate to 0 when used as a number. Unknown names evaluate to 0 and print one warning when the expression is compiled. On the built-in flat and native stages the values come from the engine defaults.

## Camera parameters

The `[Camera]` group accepts these extra keys. Every key is optional and its default reproduces the previous camera, so a stage that omits them behaves exactly as before.

| Key | Default | Effect |
| --- | --- | --- |
| `verticalfollow` | 0.2 | Fraction of the highest fighter's height the camera follows upward |
| `tension` | 50 | Horizontal dead zone: the camera only pans when a fighter comes closer than this to a screen edge, so a cornered fighter stays put while the opponent walks in |
| `verticalfollowdelta` | 0 | Dead zone in stage pixels: the vertical target only changes when it moved at least this much, which stops small jumps from shaking the camera |
| `yscrollspeed` | 0.3 | Share of the remaining vertical distance covered per tick (clamped to 0..1) |
| `zoominspeed`, `zoomoutspeed` | 0.04 | Share of the remaining zoom difference covered per tick when the camera zooms in or out (clamped to 0..1; 0 holds the zoom, 1 snaps in one tick) |
| `lowestcap` | 0 | When 1 the camera never sinks below `starty` |
| `dynamicboundhigh` | 0 | When 1 `boundhigh` is raised by `boundhighdelta` for every unit of zoom-out above 1x, so the sky stays reachable while zoomed out |
| `boundhighdelta` | 0 | Used by `dynamicboundhigh` |

`autoturn` in `[PlayerInfo]` (default 1) controls whether fighters turn around by themselves when the opponent crosses over them in the stand, walk and crouch states. `autoturn = 0` keeps their facing until a controller changes it (`AssertSpecial flag = noautoturn` still disables it per character). `StageVar(camera.<key>)` and `StageVar(playerinfo.autoturn)` read all of these.

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

## Built-in pixel art

The seven native arenas use embedded full-color PNG artwork in `assets/stages/`: `dojo.png`, `neon.png`, `bamboo.png`, `plains.png`, `spirit.png`, `volcano.png` and `brazil.png`. Generation prompts and tool provenance are recorded in `assets/stages/prompts.json`. No external image service or filesystem access is needed at runtime.

`StageArt::Illustrated` draws the background with nearest-neighbor filtering, aligns the fighting floor at 85% of the image height, and clamps horizontal camera movement to the image edges. Team zoom retains the native ground proportion. Fighter shadows, reflections and combat effects remain active; the old procedural prop animations are exclusive to procedural artwork.

The stage selector displays the same artwork, cropped to each tile without changing its aspect ratio. Random selection and imported M.U.G.E.N stages retain their existing gradient tiles. Imported stages keep their own artwork during combat.

Set `UME_STAGE_ART=procedural` to compare the original procedural stages. The selector continues to show the default pixel-art illustrations. Capture the selector with `UME_SHOTS_ONLY=stage-select ultimate-mugen --shots out/ --res 1280x720 --mute`, or all seven fights with `UME_SHOTS_ONLY=stages`.

`brazil` (Praça do Interior — Brasil) adds a golden-hour town square with colonial houses, a church, a bandstand, flowering ipê and jacaranda trees. It uses the plains music and has no legacy procedural counterpart.
