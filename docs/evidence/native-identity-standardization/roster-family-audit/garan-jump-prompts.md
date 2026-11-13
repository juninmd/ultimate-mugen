# Garan jump-family candidate prompts

Generate and review **one frame per image-generation call**. For every frame, attach `assets/chars/sprites/garan/04_special.png` first as the identity anchor and that frame's current PNG second as pose guidance only. Preserve the generated source and the normalized 400×400 candidate separately until it passes visual review. Output mapping is fixed by pose name below.

## `hop.png`

Use `garan-hop-prompt.txt`.

## `jumpDown.png`

Create one standalone native combat sprite for Garan. The first attached image is the only authority for identity, anatomy, outfit, colors and pixel-art finish; the second image supplies the descending jump pose only. Show the same massive red-skinned, bare-chested oni, long white mane, two ivory horns, purple beads, dark purple hakama, yellow rope belt, metal bracers and wrapped feet. Keep the second image's descending diagonal body angle and falling leg placement, but replace its old dark top and small silhouette with Garan's canonical anatomy and proportions. Crisp hard-edged pixel clusters, limited palette, no antialiasing, gradients, glow, particles, shadow, scenery or text. Full body, generous transparent margin, square source. Map only to `jumpDown.png`; normalize to 400×400, axis (200,392), `refHeight` 320 after review.

## `jumpTop.png`

Create one standalone native combat sprite for Garan. The first attached image is the only authority for identity, anatomy, outfit, colors and pixel-art finish; the second image supplies the tucked apex pose only. Show the same massive red-skinned, bare-chested oni, long white mane, two ivory horns, purple beads, dark purple hakama, yellow rope belt, metal bracers and wrapped feet. Preserve the compact airborne silhouette and bent knees from the pose reference while replacing its old dark shirt and smaller build. Crisp hard-edged pixel clusters, limited palette, no antialiasing, gradients, glow, particles, shadow, scenery or text. Full body, transparent background, square source. Map only to `jumpTop.png`; normalize to 400×400, axis (200,392), `refHeight` 320 after review.

## `jumpUp.png`

Create one standalone native combat sprite for Garan. The first attached image is the only authority for identity, anatomy, outfit, colors and pixel-art finish; the second image supplies the upward jump pose only. Show the same massive red-skinned, bare-chested oni, long white mane, two ivory horns, purple beads, dark purple hakama, yellow rope belt, metal bracers and wrapped feet. Preserve the rising motion and raised-knee silhouette from the pose reference while replacing its old dark shirt and smaller build. Crisp hard-edged pixel clusters, limited palette, no antialiasing, gradients, glow, particles, shadow, scenery or text. Full body, transparent background, square source. Map only to `jumpUp.png`; normalize to 400×400, axis (200,392), `refHeight` 320 after review.
