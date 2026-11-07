# Lian sprite and movement candidate

Restore the six original pixel-art poses and matching geometry. Add four running frames generated with the original identity reference and a separate four-phase motion reference. Centroid-aligned 400x400 canvases use axis (200,363), reference height 360 and the original four-tick durations.

All four runtime frames were inspected in `.tmp/lian-candidate-shots/pose-lian-run-{0,1,2,3}.png`. Costume, head silhouette, alternating limbs and a consistent floor line were visible. The green ring is present in the original reference; its rendering and interaction still need continuous-motion review.

The rising attack probe improved from 2/2/1/1 to 2/2/2/2 impacts at four distances, without changing damage definitions or active timing; see lian-uppercut-validation.md.

Draft: continuous-motion playback, guard/airborne-target balance review and the full branch workspace suite remain pending. Static captures and a forced-state mirror-match probe do not prove those requirements.
