
## Scrolling stages

Each arena has a wide strip in `assets/stages/story/<arena>.png`, one segment per fight (4 waves plus the boss). Every fight camera sits in its own segment, so the level reads as a long walk. Regenerate the strips with `python3 tools/story_segments.py`; the original stage art is never modified.

Remaining health carries over between fights and recovers a little after each wave.
