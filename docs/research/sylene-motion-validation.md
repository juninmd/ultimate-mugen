# Sylene sprite and movement candidate

Restore the six original pixel-art poses and matching geometry. Add four running PNGs using the original identity reference plus a separate motion reference. Centroid-aligned 400x400 canvases use axis (200,363), reference height 360 and original four-tick durations.

All four runtime frames were inspected in `.tmp/sylene-candidate-shots/pose-sylene-run-{0,1,2,3}.png`. Costume, hair, limb poses and floor line were consistent in those captures. The locomotion simulation visited all four frames, moved forward, stayed at y=0 and returned to state 0 with zero horizontal velocity after input release.

Draft: ability improvements, continuous visual playback, guard/airborne-target balance review and the full branch suite remain pending. The simulation checks do not certify visual continuity or complete character quality.

Probe output: `states={0, 100} run_frames={0, 1, 2, 3} displacement=266.38083 ground_error=0 final_state=0 final_y=0 final_vx=0`
