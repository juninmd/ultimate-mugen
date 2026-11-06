# Kaname sprite and movement candidate

Restore the six original pixel-art poses and matching geometry from before PR 299. Replace rig running with four new pixel-art frames, centroid-aligned 400x400 canvases, axis (200,363), reference height 360, retaining four-tick frame durations. The original generated row repeated pose pairs and was rejected. Regeneration used the original Kaname image for identity and a separate row only for motion phases.

All four runtime frames were inspected in `.tmp/kaname-candidate-shots/pose-kaname-run-{0,1,2,3}.png`; the sheathed weapon stays on the same side and the floor line remains consistent. The rising attack probe improved from 2/2/1/1 to 2/2/2/2 impacts at four distances without changing damage definitions or active timing; see kaname-uppercut-validation.md.

Draft: continuous-motion playback, guard/airborne-target balance review and the full branch workspace suite remain pending. Static frame captures and a forced-state probe do not establish those requirements.
