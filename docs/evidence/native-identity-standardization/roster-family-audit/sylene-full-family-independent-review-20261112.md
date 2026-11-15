# Sylene full-family independent visual review — 2026-11-12

Scope: all 80 Sylene families from `full-family-static-agent-5.json`, compared against `assets/chars/sprites/sylene/04_special.png` in the native contact sheet `sylene-sprites.png` and nearest-neighbor 2× inspection. Static evidence only; runtime, hitbox and transformation playback are not approved here.

Canonical anchor SHA-256: `bbb8675275a0ce31293f82b4a7a9898519c1b020e5a11250842309c8c4179c58`.

## Classification

Static pass (73): `idle`, `turn`, `walk`, `walkBack`, `crouchStart`, `crouch`, `crouchEnd`, `crouchTurn`, `jumpStart`, `jumpUp`, `jumpFwd`, `jumpBack`, `jumpDown`, `land`, `run`, `hop`, `guardStart`, `crouchGuardStart`, `guard`, `crouchGuard`, `guardEnd`, `guardHit`, `crouchGuardHit`, `hitLight`, `hitMedium`, `hitHeavy`, `hitLowLight`, `hitLowMedium`, `hitLowHeavy`, `hitCrouch`, `hitAir`, `fall`, `bounce`, `bounceAir`, `lieDown`, `getUp`, `dead`, `trip`, `airRecover`, `win`, `lose`, `draw`, `intro`, `jab`, `strong`, `lkick`, `hkick`, `cjab`, `clk`, `sweep`, `ajab`, `akick`, `lance`, `spire`, `glide`, `super`, `blizzard`, `embrace`, `crown`, `lanceEx`, `spireEx`, `crownUlt`, `stringA`, `stringB`, `stringC`, `defeat1`, `defeat2`, `defeat3`, `defeat4`, `powerUp`, `taunt`, `lowKick`, `airLowKick`.

Correction required (3): `airGuardStart`, `airGuard`, `airGuardHit`.

Unresolved (4): `lanceFx`, `iceHit`, `spireFx`, `winterBeam`. These are effect-only/procedural families without resolved body PNGs, so identity and body-pixel claims cannot be made.

## Identity versus finish/pose findings

No wrong-identity family was found in this pass. The three correction-required families retain Sylene's canonical identity: long silver/cyan hair, pale blue-white mage robe, blue panels and bare face. Their issue is finish/pose consistency: the air-guard frames are flattened and simplified compared with the canonical mage silhouette and pixel clusters. Correct them by restoring volume, cluster detail and readable air-guard pose while preserving the existing identity and palette.

The low-kick and air-low-kick families are static passes in the current inventory and have dedicated body frames, but this classification does not prove runtime collision alignment. Existing transformation/runtime captures remain separate evidence and do not replace the complete family audit.

## Evidence and hashes

Ordered frame paths and current per-file SHA-256 values for all 80 rows are authoritative in `full-family-static-agent-5.json` and `native-size-page-manifest.json`; the Sylene subset contains the active resolved PNGs. The canonical hash above pins the identity reference. The reviewed contact sheet is `sylene-sprites.png`; no active asset was changed during this audit.

Conclusion: Sylene is largely standardized in the static inventory, with 73 passes, 3 finish/pose corrections and 4 effect-only unresolved families. Runtime and PR completion remain unverified.
