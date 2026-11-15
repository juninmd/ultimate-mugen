# Oboro jump-family candidate prompts

Generate and review one frame per image-generation call. Attach `assets/chars/sprites/oboro/01_idle.png` first as the identity authority and the current target PNG second for pose guidance only. Preserve the generated source and normalized 400×400 candidate separately until it passes native-size review. Keep the same pale-skinned swordswoman, long violet ponytail tied with red, red-trimmed black outer kimono, cream inner robe, red sash, sandals, and facial markings. Preserve the pose and direction from the second image while replacing its masked dark-ninja outfit with Oboro's canonical kimono. Use hard-edged pixel clusters, the canonical pack's limited palette and finish; no antialiasing, gradients, glow, particles, shadow, scenery, text, or checkerboard. Map each result only to the target named below. Pack geometry for each target is axis (200,392), `refHeight` 320.

## `jumpDown.png`

Create the descending jump pose from the second reference, with the body angled downward and the sword held in the same direction. Keep the full body readable and the kimono, ponytail, and sash responding to the fall. Map only to `jumpDown.png`.

## `jumpTop.png`

Create the compact apex pose from the second reference, knees tucked and sword held defensively. Keep the ponytail and kimono folds readable without widening the silhouette. Map only to `jumpTop.png`.

## `jumpUp.png`

Create the rising jump pose from the second reference, with the sword and leading knee maintaining its upward motion. Keep the long ponytail and robe in the same canonical design. Map only to `jumpUp.png`.
