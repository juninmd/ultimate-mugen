# Replay Camera

The replay camera is a display-only director. It frames the fight during replay playback but never changes the simulation: the camera is saved before drawing and restored right after, so the replay result and checksums stay the same.

Inputs per frame: one `Subject` per fighter (`x` and whether a hit is active). Output: the focus `x` and the `zoom` for the draw.

- Focus is the midpoint of the fighters, leaning 35% toward the attacker so hits are framed.
- Pan eases 12% of the way toward the focus each frame, on top of the simulation camera.
- Zoom tightens when fighters are close, up to 1.6x the match zoom, and stays at the match zoom when they are far apart.
- With no valid fighters the previous framing is kept.

Pure and deterministic: no clock, no randomness.

Code: `crates/engine/src/replay_camera.rs` (framing applied in `src/app/fight_screen.rs` around the draw). Tests: `crates/engine/tests/replay_camera.rs`.
