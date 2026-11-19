# Make a stage (quick start)

**English** · [Português (BR)](pt-BR/Criar-um-Cenario.md)

There are three kinds of stage work. Pick the one you need:

| You want | Needs code? | Go to |
|---|---|---|
| A new background from your own art, with the M.U.G.E.N format | No | [M.U.G.E.N stage](#1-a-mugen-stage) |
| More content for one of the five story arenas (objects, music, animated sky) | No | [Story arenas](#2-story-arenas) |
| A new arena in the story mode or a new built-in arena | Yes, Rust | [What needs code](#3-what-needs-code) |

## 1. A M.U.G.E.N stage

1. Put two files in `mugen/stages/`: `my-stage.def` and `my-stage.sff` (the art).
2. Start from [`docs/examples/stage/my-stage.def`](../examples/stage/my-stage.def).

The `.def` file is plain text with sections:

| Section | What to set |
|---|---|
| `[Info]` | `name`, `displayname`, `author` |
| `[Camera]` | `startx`, `boundleft`, `boundright` (how far the camera can go), `boundhigh`, `boundlow`, `verticalfollow`, `tension` |
| `[PlayerInfo]` | `p1startx`, `p2startx` (where each fighter starts) |
| `[StageInfo]` | `zoffset` (height of the floor line on screen), `localcoord` (the drawing size, for example `320, 240`) |
| `[BGdef]` | `spr`: the `.sff` file |
| `[BG name]` | One section per layer (see below) |

Each background layer:

```ini
[BG Sky]
type = normal          ; normal (one sprite) or anim (an action from the .air file)
spriteno = 0, 0        ; group, image of the .sff
layerno = 0            ; 0 = behind the fighters, 1 = in front
start = 0, 0          ; position in local units
delta = .2, .2         ; parallax: 1, 1 moves with the floor, smaller moves slower
tile = 1, 0            ; repeat on x, on y (0 = no repeat)
```

Every key of the stage format, including scale, rotation and clipping windows, is in the [stage reference](Stages.md#mugen-stage).

Text after `;` is a comment. Key names are the M.U.G.E.N ones, so a stage copied from a M.U.G.E.N pack is a good starting point.

Limits of this path: the animated loop, story strips, breakables and per-stage music from the next sections do not apply to M.U.G.E.N stages.

## 2. Story arenas

The story mode has five arenas: `brazil`, `neon`, `bamboo`, `volcano` and `dojo`. Each one can be extended with data:

| What | Where | Notes |
|---|---|---|
| Backdrop strip | `assets/stages/story/<arena>.png` | One segment per fight (four waves and the boss). Regenerate with `python3 tools/story_segments.py` |
| Animated background | `assets/stages/animation/<arena>/` | Eight opaque 1672×941 keyframes (`00.png` through `07.png`) and eight interleaved `-mid.png` frames. Playback advances every four ticks and wraps after 64 ticks. Keep the camera and fighting floor registered throughout the loop. |
| Breakables, food, bonus round | `assets/story/objects.toml`, table `[stages.<arena>]` | Full format in [Stage objects](Stage-Objects.md) |
| Fight music | `tweaks.toml`, section `[music]`, key `<arena id>` | Path to `.ogg`, `.mp3`, `.wav` or `.flac`, relative to `tweaks.toml`. Desktop only. See [Configuration](Configuration.md) |

Example `tweaks.toml`:

```toml
[music]
fight = "music/my-fight.ogg"   ; every fight without its own track
dojo = "music/dojo.ogg"        ; wins over fight, only in the dojo
```

Example `objects.toml` entry:

```toml
[stages.dojo]
objects = [
  { fight = 0, kind = "crate", x = -78.0, lane = -22.0, drop = "apple" },
]
```

Verify with the [Stage objects](Stage-Objects.md#data-assetsstoryobjectstoml) evidence tour: `UME_TOUR=storyobjects` and `UME_TOUR=storyloop`.

## 3. What needs code

These things cannot be done with files today. Each one is a feature gap, not a bug in your files:

- **A new built-in arena.** Native stages are Rust functions in `src/stages.rs` (`builtin()`). A new one needs code, an embedded PNG and a thumbnail. See [Creating stages](Stages.md#native-stage).
- **A new story arena.** The list of arenas is fixed in `src/stages/story_strip.rs` (`STORY_ARENAS`) and the story mode data in `src/story/stages.rs`.
- **An animated sky for a M.U.G.E.N stage.** Loop frames are only read for built-in arena ids.
- **A `[Music]` section in a `.def`.** Music per stage comes from `tweaks.toml` only.
- **A `type = parallax` layer.** Planned, not in the format.

## Test

```bash
cargo test --release -p ume-engine --test wiki_examples
UME_SHOTS_ONLY=stages cargo run --release -- --shots out/   # screenshots of every stage
```

The first command checks the sections of the example `.def`. The second saves screenshots of the stages to `out/`.

## Next

[Stage reference](Stages.md) · [Stage objects](Stage-Objects.md) · [Story mode](Story-Mode.md) · [Make a character](Make-a-Character.md) · [Wiki home](Home.md)
