# Cross-review of the 16 playable transformations — 2026-11-12

Scope: the three resolved `powerUp` body frames for each playable fighter, compared at native 400×400 and nearest-neighbor 2× with that fighter's `04_special.png`, plus the existing stable post-transformation runtime capture. The source frame/hash inventory is `transformation-static-audit/powerup-source-review.json`; runtime GIF and final-frame hashes are in `transformation-runtime-audit-current/transformation-runtime-audit.json`.

## Per-fighter frame sources

| Fighter | Power-up frames | Static identity/wardrobe/colour result | Stable runtime result |
|---|---|---|---|
| Akio | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Bruno | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Daichi | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Garan | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Haruki | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Jin | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Kaito | `powerup0.png`, `powerup1.png`, `powerup2.png` | pass; 3 distinct body images | pass, settled idle only |
| Kaname | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Lian | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Marlow | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Oboro | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Shion | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Suzume | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Sylene | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Valdric | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |
| Volta | `15_power0.png`, `16_power1.png`, `17_power2.png` | pass; 3 distinct body images | pass, settled idle only |

## Hash and state evidence

The exact SHA-256 for every listed frame is recorded beside its path in `transformation-static-audit/powerup-source-review.json`; the corresponding runtime GIF and final-frame SHA-256 for each fighter are recorded in `transformation-runtime-audit-current/transformation-runtime-audit.json`. The 16 declared forms and their palette/identity mappings are the current native JSON declarations. All 48 power-up body frames resolve to the current pack identity; no old or cross-character body image was observed in the native/2× comparison.

## Open transition states

The runtime evidence captures the final settled idle after a 20-frame transform-only clip. It does not prove every intermediate transition frame, form-specific attack family, or collision alignment. Therefore the transformation transition itself remains open for all 16 fighters even though settled identity, clothing and palette pass. The transformation audit also does not clear unresolved or correction-required base attack families listed in `family-review-checklist.json`.

Conclusion: all 16 playable transformations pass the available static power-up source and settled-idle identity checks; all 16 still need explicit transition-cycle and form-attack runtime validation before a full visual-standardization claim or PR completion claim.
