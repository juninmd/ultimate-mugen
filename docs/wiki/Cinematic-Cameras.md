# Cinematic cameras

The first-round intro and every round-win pose run a short camera shot made of beats. The shot is a pure function of the match clock, so replays and rollback stay deterministic.

- Beats live in `assets/cinematic.toml`. A copy named `cinematic.toml` next to the settings file replaces the `[[intro]]` and `[[victory]]` sections it defines; an invalid file is ignored.
- Each beat: `subject` (`p1`, `p2`, `both`, `winner`), `duration` seconds, `ease` (`linear`, `in`, `out`, `in_out`, `back`), and `[from, to]` pairs for `zoom` (added to the stage zoom), `focus` (0 wide to 1 on the subject), `yaw` (radians, 3D fighters only) and `shake` (trauma).
- Values are clamped, so a bad file cannot break the view. `enabled = false` turns every shot off.
- Fighters without a 3D model get the same zoom and pan on their sprites; 3D fighters also orbit.
- The intro shot ends when the existing intro skip fires; Reduce motion disables zoom, pan, shake and orbit.
- Smoke: `node scripts/web-cinematic-smoke.js web/dist` (needs `bash scripts/build-web.sh`).
