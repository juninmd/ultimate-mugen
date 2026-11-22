# Photo Mode

Photo mode freezes a fight and gives you a free camera, color filters and a clean frame to save as a PNG. It works in normal fights, training, survival and replays, so you can pause a replay on the perfect hit and shoot it.

## Open and close

1. Pause the fight (`P`, `Esc`, `Enter` or the pause button). In a replay, pause with `Space`.
2. Press `C`. The pause menu disappears and the camera is yours.
3. `Esc`, `P` or `C` leaves photo mode and returns to the pause menu.

Photo mode is not available in online matches, where the simulation can never stop.

## Controls

| Key | Action |
|---|---|
| `W A S D` / arrows | Pan the camera (slower the closer you are) |
| `+` / `-`, `PageUp` / `PageDown`, mouse wheel | Zoom between 0.6x and 2.5x |
| `F` | Cycle the filter: Natural, Cinema, Noir, Vivid, Retro |
| `H` | Show or hide the HUD (hidden by default) |
| `.` | Advance one frame to find the exact moment |
| `R` | Recenter the camera |
| `Enter` / `Space` | Take the photo (desktop only) |

The overlay (state line, key help and rule-of-thirds guides) is not part of the saved image.

## Where photos go

Desktop builds write `photos/ultimate-mugen-<milliseconds>.png` next to the executable at the window resolution. The web and mobile builds have the camera and filters but no save, because the browser or the OS screenshot already covers it.

## How it works

- Camera: `crates/engine/src/photo_camera.rs` is display-only, like the [Replay Camera](Replay-Camera.md). It temporarily changes the camera position, the view size and the ground offset, draws, and restores every field, so the simulation, replay result and checksums never change. Panning is clamped to the stage bounds, vertical pan to 120 units and zoom to 0.6x to 2.5x.
- Filters: each filter is a set of values for the post-processing `Look` (`saturation`, vignette, grain, grade, chroma, bloom). The look is swapped for the draw and restored right after, and the post-processing level is raised to at least 2 so filters also show with effects set to minimum.
- HUD: `Hud::hidden` skips the HUD, cut-in bars and banners.
- Cost: no extra draw pass. The only addition is the same post-processing the game already runs.

## Verification

`UME_SHOTS_ONLY=photo xvfb-run -a target/release/ultimate-mugen --shots <dir> --res 1280x720 --mute` writes the pause screen, a clean frame, the HUD frame, the four filters and a wide shot. Tests: `cargo test -p ume-engine --test photo_camera` and the `photo` unit tests in the root crate.
