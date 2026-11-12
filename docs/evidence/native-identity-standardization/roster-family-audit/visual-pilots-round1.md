# Parallel image pilot round 1 — 2026-11-12

Five worker agents produced eight Codex-host image candidates across six confirmed or strongly supported targets. All source PNG hashes were rechecked against each pilot manifest and remain unchanged. All candidate files remain isolated under `candidates/`; none was integrated.

| Target | Attempts | Result |
|---|---:|---|
| Suzume `crouch.png` | 2 | First output was 1254×1254 with 89,576 RGBA colors and 373,450 partial-alpha pixels; it was rejected before integration. Second output also was 1254×1254 and failed strict grid QA (score 0.012, uniformity 0.13). |
| Haruki `jab0.png` | 1 | Identity and broad jab pose retained; 1254×1254, 80,147 RGBA colors, 256,329 partial-alpha pixels; strict grid QA failed (score 0.012, uniformity 0.192). |
| Kaname `cjab0.png` | 1 | Identity and broad pose retained; 1254×1254, 64,267 colors, 372,385 partial-alpha pixels; strict grid QA failed (score 0.009, uniformity 0.333). |
| Garan `jumpDown.png` | 2 | Both candidates changed the falling pose to a near-vertical guard pose and failed strict pixel-grid QA (scores 0.011/0.006; uniformity 0.079/0.439). `airLowKick.png` was directly verified canonical and was excluded from replacement. |
| Valdric `turn.png` | 1 | Candidate changed the turn into a crossed-arm guard; 1254×1254, 102,391 colors, 339,396 partial-alpha pixels; strict grid QA failed (score 0.006, uniformity 0.191). |
| Volta `jab0.png` | 1 | Canonical armor and broad punch pose recovered; 1254×1254, 35,021 colors, 271,106 partial-alpha pixels; strict grid QA failed (score 0.016, uniformity 0.127). |

The strict QA used `pixel_reduce.py --colors 24 --strict --upscale 3` without `--force`; each failure reported no clean logical pixel grid (thresholds: score at least 0.5, uniformity at least 0.9). The rejections are recorded in each pilot's `candidate-review*.json`. These generation results do not satisfy the project pixel-art standard and cannot replace the queued corrections. No runtime, test, or PR claim is made from these candidates.

## Pixel-finish follow-up

The raw generation failures above are pre-processing results, not a blanket rejection of the image route. Haruki `jab0.png` now has a processed candidate that passes the project sprite pipeline's native geometry and strict palette-output gates: 400×400, 43 RGBA colors including transparency, binary alpha, palette-only output pixels, max ΔE 0.097131 under the 0.1 gate, and registered floor/axis `[200,392]`. It is visually coherent with Haruki's canonical coat and preserves the jab setup pose at native and 2× nearest scale. It remains isolated because `jab1.png` is also mismatched and the entire jab sequence needs one shared palette and runtime validation. Evidence: `candidates/haruki-jab0-pilot/pixel-finish-review-20261112.json` and `registered-v1/candidate-comparison-2x*.jpg`.

This establishes a viable normalization/palette route for the other raw candidates; it does not resolve the remaining roster correction queue.
