# Replay Camera

The replay camera is a cinematic director for playback. It only chooses where the shot goes; the replay itself still re-simulates the same inputs, so the match result never changes.

Inputs per frame: one `Subject` per fighter (`x` and whether it is attacking). Output: a `Shot` with the focus `x` and `zoom`.

- Focus is the midpoint of the fighters, leaning 35% toward the attacker so hits are framed.
- Zoom tightens when fighters are close, from 1.0 up to 1.6, and stays at 1.0 when they are far apart.
- Pan and zoom ease toward their targets, so the shot never snaps.
- Non-finite positions are ignored; with no valid fighters the last shot is kept.

Pure and deterministic: no clock, no randomness, so the same replay gives the same camera path on every machine.

Code: `crates/engine/src/replay_camera.rs`. Tests: `crates/engine/tests/replay_camera.rs`.
