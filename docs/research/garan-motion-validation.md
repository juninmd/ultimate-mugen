# Garan sprite and movement candidate

Restore the six original pixel-art poses and matching geometry from before PR 299. Add four centroid-aligned running PNGs on 400x400 canvases, axis (200,363), reference height 360, retaining the original four-tick animation durations. All four runtime frames were inspected in `.tmp/garan-candidate-shots/pose-garan-run-{0,1,2,3}.png`; identity, alternating limb poses and a consistent floor line were visible.

The rising attack connection probe improved from 2/2/1/1 to 2/2/2/2 impacts at distances 24/36/48/60. Damage definitions, startup and invulnerability are unchanged; see garan-uppercut-validation.md for limits.

This remains a draft: continuous movement playback, guard/airborne-target balance checks and the full branch workspace suite are pending. Runtime frame captures and forced-state collision probes do not establish these requirements.
