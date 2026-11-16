# Round 3 cross-review — 2026-11-12

Five independent read-only reviewers covered all 21 packs in the current 1,703-family report. This confirms roster and pack coverage only; all rows still have `nativeResolutionReview=pending` and `runtimeReview=not_performed`. The consolidated static tally remains 804 `static_pass`, 741 `correction_required`, and 158 `unresolved`. The 741 values count family rows, not distinct replacement files.

| Pack | Static pass | Correction required | Unresolved |
|---|---:|---:|---:|
| Akio | 58 | 19 | 4 |
| Bruno | 41 | 36 | 2 |
| Daichi | 56 | 21 | 4 |
| Garan | 65 | 12 | 3 |
| Haruki | 24 | 53 | 2 |
| Jin | 27 | 49 | 4 |
| Kaito | 40 | 38 | 3 |
| Kaname | 37 | 44 | 4 |
| Lian | 36 | 41 | 2 |
| Marlow | 53 | 24 | 3 |
| Oboro | 17 | 60 | 10 |
| Shion | 19 | 59 | 4 |
| Suzume | 18 | 60 | 5 |
| Sylene | 73 | 3 | 4 |
| Valdric | 24 | 53 | 4 |
| Volta | 35 | 42 | 4 |
| npc_brute | 39 | 36 | 2 |
| npc_colossus | 0 | 0 | 81 |
| npc_phantom | 1 | 77 | 3 |
| npc_razor | 78 | 0 | 5 |
| npc_spitfire | 62 | 14 | 5 |

## Highest-confidence findings

- Haruki `crouch0.png` matches its canonical long purple-and-gold coat and remains excluded from correction. Other individually adjudicated old-outfit frames remain queued.
- Suzume's canonical identity is the masked purple-haired kunoichi in `04_special.png`; `01_idle.png` agrees. Hooded/covered dark-haired frames with dark navy-violet armor are visual mismatches; palette forms and the explicit Oboro character swap do not explain them. The 60 affected family descriptions in the consolidated JSON were corrected to remove the stale “unmasked” claim.
- Garan's jump/air families include a small dark horned fighter in place of the canonical large red oni; unresolved FX are excluded from body-sprite claims.
- Volta mixes the bulky graphite/cyan armor in `04_special.png` with a thinner white armor/yellow crest in `jab0.png`; declared forms recolor energy and do not explain this body change.
- Valdric `turn.png` uses an older hooded/purple armor look that conflicts with the black/gold armor, dark purple hair, and red cape in `04_special.png`.
- Jin, Kaito, Kaname, Oboro, Shion and the other reviewed packs have material family discrepancies documented in the underlying consolidated report. Each target still needs individual native-scale and runtime approval.
- NPC coverage is not closed by the story-mode capture: `npc_colossus` has 81 unresolved rows because it inherits Daichi's sprite set; `npc_phantom` reuses Jin sprites; NPC recolors/poses still need per-family review.

A cross-reviewer checked the report's pack roster against the 21 sprite directories and found 1,534 distinct referenced PNG paths present with matching hashes. This verifies report integrity, not visual acceptance.

## Shion idle source adjudication — 2026-11-12

A fresh native 400×400 and 2× nearest-neighbor comparison of `01_idle.png`, `07_idle1.png`, `08_idle2.png` and canonical `04_special.png` found no identity or finish mismatch in the idle family. Long white hair, black robe, red trim/sash, gray trousers and the broad sword remain consistent; hair and sword changes follow the idle/attack pose. `01_idle.png` SHA-256 is `b68d85597fe612f657c0769e481fabfe452c1f584d2db32c7fb1663afc2b9eef`, matching the family report. The idle row is already `static_pass`; do not replace these sources based only on their reuse count in other families. Evidence: `shion-idle-native-review-2x.jpg`.
