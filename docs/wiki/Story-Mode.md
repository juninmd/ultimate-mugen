## Scrolling stages and wave villains

Each arena has a wide strip in `assets/stages/story/<arena>.png`, one segment per fight (4 waves plus the boss). Wave villains have around 10 % of the protagonist's maximum life (`villain_life_ratio` in `src/story/stages.rs`: 9 % to 12 % depending on archetype). When a wave is defeated the screen is released: the protagonist keeps control and explores the segment freely, a GO arrow shows at the right edge, and the camera only scrolls to the next segment once the player walks to the exit; the next wave then waits frozen off screen (the screen clamp ignores it) and walks in from the right edge once the scroll lands and the screen locks again (`LaneRules::gate`, `Match::lane_scroll_step` in `crates/engine/src/lane.rs`, `App::story_advance_in_place` in `src/app/story/advance.rs`). Upon reaching the rightmost segment at the end of the stage, the stage boss awaits. Regenerate the strips with `python3 tools/story_segments.py`; the original stage art is never modified.

## Animated backdrops

The scrolling backdrop is alive in every segment. While a stage's eight ambient loop frames are loaded (see [Stages](Stages.md)), the renderer rebuilds the strip from the current frame on each draw instead of showing the static PNG: the first segment is the arena itself, the three middle segments are mirror chains of a window of the arena, and the boss segment is the arena flipped. Colour grading follows the same ramp as the baked strip and the 70 px seams are cross-faded with mirrored slices, so the result lines up with the static strip (a unit test compares both pixel by pixel outside the seams). The loop advances every eight simulation ticks, in step with versus mode, and costs no extra texture memory because it samples the frames the stage already loaded.

The layout lives in `src/stages/story_loop.rs` (`runs`, `pieces`, `grade_at`, `seam_mix`) and the drawing in `src/render/story_loop.rs` (`Gfx::draw_backdrop`). The static strip stays as the fallback when no loop frames exist and still defines the strip size. To capture the evidence set, run `UME_TOUR=storyloop target/release/ultimate-mugen --shots <dir>` (add `UME_TOUR_BURST=1` for GIF frames); results are in `screenshots/story-loop/`.

Remaining health carries over between fights and recovers a little after each wave.

## Breakables, food and bonus rounds

Crates and barrels along each strip break when struck and can drop food, a gem or a battery; stages 1 and 3 end with a bonus round (wreck the car, break the stash). See [Stage objects](Stage-Objects.md).

## Final boss and epilogues

The last stage (Kingpin Rooftop) ends against **Valdric Morn**, commander of the Red Star Empire and the kingpin behind the street gang. A player who picked Valdric faces **Kaito Arashi** instead (`final_boss` in `src/story/epilogue.rs`, which reads the last entry of `STAGES`).

Beating the final boss plays an epilogue before the `STORY COMPLETE` card. Every fighter of the roster has its own, in English, Portuguese and Spanish, made of five pages:

| Page | Scene | Content |
|---|---|---|
| 1 | Rooftop at night, boss on the ground | the boss speaks to this fighter |
| 2 | Rooftop, fighter in the victory pose | the fighter answers |
| 3 | The fighter's home scenery at dawn | narration |
| 4 | Home scenery in daylight, victory pose | narration |
| 5 | Home scenery, dimmed | `THE END` card with the fighter and the epilogue title |

The scenery is the illustration of an existing stage with a slow pan, drifting light motes, letterbox bars, a fade when the scene changes and text typed letter by letter. `OK` reveals the line and then advances, `ESC` skips to the final card.

The fighter on screen plays its own animations: the walk cycle (`20`) carries it into the home scene, the taunt (`195`) or win pose (`181`) plays while it answers, and the win pose closes the story. Animations a fighter does not have fall back to the next one in the chain and finally to idle (`src/story/epilogue_pose.rs`).

Texts live in `assets/story/epilogues.json`, keyed by fighter id. A fighter with no entry (imported M.U.G.E.N characters or custom ones) gets an invented epilogue: one of the `*:tale1` to `*:tale6` templates, picked by a stable hash of the fighter id, with `{name}` replaced by the fighter's display name. `*` stays as the last resort when the name is empty:

```json
{
  "mika": {
    "scene": "neon",
    "title": { "en": "Storm Season", "pt": "Temporada de Tempestade" },
    "boss": { "en": "Line the final boss says to this fighter." },
    "reply": { "en": "The fighter's answer." },
    "pages": [{ "en": "First narration page." }, { "en": "Second narration page." }]
  }
}
```

`scene` is any built-in stage id. A missing language falls back to English. To change or add epilogues without rebuilding, save a file with the same shape as `story-epilogues.json` next to the executable (or in browser storage): its entries replace the bundled ones fighter by fighter.

Code: data and lookup in `src/story/epilogue.rs`, flow in `src/app/story/epilogue.rs`, drawing in `src/app/story/epilogue_draw.rs` and `epilogue_text.rs`. Evidence in `screenshots/story-epilogues/`, captured with `UME_TOUR=epilogue ultimate-mugen --shots <dir> --res 1280x720 --mute`; the browser check is `node scripts/web-epilogue-smoke.js web/dist`.

The loading screen is no longer a hero-versus-villain card in the story. Wave fights show a stage card (stage number, title, wave and hero name; `src/app/story/stage_card.rs`) so no enemy is exposed; the classic versus screen is kept for boss fights only, with a BOSS tag and the stage title over it.

Clearing a stage boss now pays a life bonus on top of the boss bonus: `StageTally` (`src/story/tally.rs`) gives `remaining life x 2000 x stage number`, and the Stage Clear screen lists both (`STAGE BONUS +n   LIFE +n`). The browser hook `?e2e_clear=1` opens that screen for smoke checks.

Story juice (`src/vfx/story_juice.rs`, active only while lane rules are on): footstep dust while the hero walks, sparkles over pickups, a `K.O.!` pop-up with ring and sparks when a foe falls, a ring and sound when the screen is released, and an animated GO cue (pulse plus sliding chevrons). Everything is procedural, no sprite is touched.

When the scroll lands and the screen locks again, `story_juice` adds a screen shake, a thud, rings on both screen edges and dust bursts under the waiting wave; while the hero is below 25 % life the vignette pulses.
