# Suzume sprite rework review

## Scope and decisions

- Reviewed all 118 combat PNGs in the native pack. Replaced 53 outlier poses across movement, attacks, jumps, falls, get-up and defeat sequences; retained the 65 poses that already matched the approved Suzume identity and finish.
- Preserved `01_idle.png` as the identity anchor, the original 400×400 canvas, `meta.json`, and the high-resolution portrait. The portrait SHA-256 remains `a93a4c9716510850362816c6e7bbf69d7cd7c864ea4cd387ea8c4bb426d414ad`.
- Kept `ks0`–`ks2` unchanged. They are the substitution clone effect poses and their glow is present in the declared move.
- Re-reviewed candidate poses against the canonical idle at light and dark backgrounds. Contact sheets cover the full current pack; per-pose review composites were also made for the replaced frames.

## Structural and gameplay checks

- All 118 combat images are 400×400 RGBA with binary alpha (0 or 255); none has semitransparent edge pixels. See `sprite-inventory.json`.
- `native_roster_lab` passed for Suzume: 56 attack states connected, dealt damage and returned control.
- `native_box_sprite_fit` passed on the Suzume-only dataset: 337 animation frames checked, zero box-to-visible-pixel violations.
- All six sound groups referenced by the character data (200/0–5) resolve to existing WAV files. Audio files and move data were not changed; this is a structural reference check, not a subjective listening review.
- Windows native release build and pixel/move showcases completed. The revised idle and live fight were inspected in-game.
- `scripts/build-web.sh` completed. Chromium smoke returned `WEB SMOKE OK`; the roster selection and live fight screenshots here specifically show Suzume. The optional luminance check was skipped because `pngjs` was unavailable in this environment. The screenshot produced after the script's synthetic blur event was black, so it is excluded as evidence; the live fight captured before that event rendered correctly and was inspected directly.
- `node scripts/file-size.mjs` passed; `git diff --check` passed.

## Visual evidence

- `03-p1.jpg` and `05-fight.jpg`: Suzume selected and fighting in the browser build.
- `suzume-idle.jpg` and `suzume-defeat.jpg`: native game captures.
- `walk-cycle-after.gif`: animated walk cycle.
- `suzume-01` through `suzume-08` (light and dark): full-pack contact sheets.
- `M-suzume-shadow-step.jpg`: native ability capture.
