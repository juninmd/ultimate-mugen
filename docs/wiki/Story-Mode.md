## Scrolling stages and wave villains

Each arena has a wide strip in `assets/stages/story/<arena>.png`, one segment per fight (4 waves plus the boss). Wave villains have around 10 % of the protagonist's maximum life (`villain_life_ratio` in `src/story/stages.rs`: 9 % to 12 % depending on archetype). As each wave is defeated, the protagonist stays in the stage and advances to the right while the camera smoothly scrolls to the next segment (`Match::lane_scroll_step` in `crates/engine/src/lane.rs`, `App::story_advance_in_place` in `src/app/story/advance.rs`). Upon reaching the rightmost segment at the end of the stage, the stage boss awaits. Regenerate the strips with `python3 tools/story_segments.py`; the original stage art is never modified.

Remaining health carries over between fights and recovers a little after each wave.
