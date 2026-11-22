# Sylene native sprite review

The character keeps the existing portrait, fighter definition, geometry metadata, move declarations, and sound files. This change updates 57 combat PNGs to bring their costume and silhouette into line with Sylene's canonical design, while preserving the other 62 native combat poses.

Every combat sprite remains a transparent 400 × 400 RGBA image. The full roster contains 119 combat images; all 119 were reviewed in labeled contact sheets on light and dark backgrounds. The comparison sheet shows representative original and revised poses. The in-game captures come from the built WebAssembly bundle running in headless Chromium at 1280 × 720.

## Evidence

- `comparison-antes-depois.jpg`: six original/revised pose pairs.
- `sylene-final-01-*` through `sylene-final-04-*`: all combat poses, reviewed on both backgrounds.
- `fight.png` and `match.gif`: live web fight capture with Sylene selected.

## Checks

- Native sprite audit: 87 animation families, no missing sprites.
- Native box/sprite fit audit: 6,708 frames checked, no boxes outside opaque sprite pixels.
- Sylene roster lab: all 49 attack states connected with finite values.
- Move audit: Sylene has 23 normals, 13 specials and 3 supers; whole-roster audit reported zero problems.
- Web build: passed; Playwright smoke reached title, menu, select, stages and fight (`WEB SMOKE OK`).
- Sound files remain unchanged; six WAVs passed PCM format, duration and non-silence checks.

The visual pass is based on the image sheets and game capture. Audio validation here checks file integrity and activity levels; it does not claim subjective listening review.
