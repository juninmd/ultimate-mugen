# Daichi candidate validation

The original six pixel-art poses and matching geometry were recovered from before PR 299. The four running frames are sprite-gen candidates aligned by alpha centroid; runtime visual approval is still pending.

Daybreak retains its two active frames, damage, startup, invulnerability and input. The first hit now requests 24 ticks of hitstun and smaller knockback (-1, -3), while the finishing hit requests (-3, -8). The intent is to retain the victim near the rising attack before the finisher; an actual two-hit simulation is required before claiming this improves connection consistency or balance. These changes remain unapproved candidates.

All four running frames were inspected in the rebuilt runtime probe at `.tmp/daichi-scaled-shots/pose-daichi-run-{0,1,2,3}.png`. Reference height 360 and axis (200,363) apply only to the new 400x400 canvases. The four original animation durations remain four ticks each. The contact sheet exposed horizontal drift with foot-centroid alignment; extraction was rerun with centroid alignment before import. New frames retain transparent backgrounds and distinct alternating limb poses. Full match movement and Daybreak connectivity validation remain pending.

## Controlled Daybreak comparison

The forced-state 1001 probe completed for baseline and candidate at distances 24, 36, 48 and 60. Baseline: two impacts and 128 damage at 24/36; one impact and 81 damage at 48/60. Candidate: two impacts and 128 damage at all four distances. Damage values include character attack/defense scaling. The result supports connection improvement in this mirror-match setup, not universal balance or command-input correctness. The candidate retains the original damage values and active-frame timings.

## Workspace regression suite

`cargo test --workspace --release` completed successfully (exit 0). Detailed output is retained locally in `.tmp/daichi-workspace-tests.log`. This confirms the branch workspace test suite; continuous motion playback, guard and airborne-target balance checks remain separate pending requirements.

## Locomotion simulation

`native_locomotion_probe` passed: run frames 0/1/2/3 visited, forward displacement 291.13544, zero ground error, final state 0, y=0 and vx=0 after input release. This validates simulation traversal and release, not visual motion continuity.
