# Volta motion and Tesla candidate

Restore the original six pixel-art poses and matching geometry. The new four-frame running cycle uses centroid-aligned 400x400 canvases, axis (200,363), reference height 360 and the original four-tick frame durations. All four frames were inspected in the runtime captures `.tmp/volta-candidate-shots/pose-volta-run-{0,1,2,3}.png`.

A controlled Tesla state-1001 mirror-match probe at distances 24/36/48/60 produced 2/2/1/1 impacts before and 2/2/2/2 afterward. Two-hit damage remains 114 in this setup. Damage definitions, timing, inputs and invulnerability are unchanged. The starter requests 24 hitstun ticks and knockback (-1,-3); the finisher requests (-3,-8).

Full-motion playback, guard and airborne-target balance review and the full workspace suite remain pending. Static runtime captures and a forced-state probe do not prove those requirements.

## Workspace suite

`cargo test --workspace --release --target-dir ../character-daichi/target` completed successfully (exit 0): 645 passed, 0 failed, 2 ignored. The local log is `.tmp/volta-workspace-tests.log`. This updates the workspace-suite pending item above; continuous motion and guard/airborne-target balance review remain pending.
