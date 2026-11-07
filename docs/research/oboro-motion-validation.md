# Oboro sprite and movement candidate

Restore the six original pixel-art poses and their matching geometry. Add a four-frame centroid-aligned running cycle on 400x400 canvases, axis (200,363), reference height 360, keeping the original frame durations. Earlier rows with repeated poses or inconsistent extraction grids were rejected; this candidate used an additional reference only for motion phases.

All four runtime frames were inspected in `.tmp/oboro-candidate-shots/pose-oboro-run-{0,1,2,3}.png`. Identity and a consistent floor line were visible. Character aura visible in these captures is a runtime effect, not baked into the new PNGs.

Draft: ability improvements, continuous-motion review, guard/airborne-target balance checks and the full branch workspace suite remain pending. This PR does not claim that all requested character work is finished.
