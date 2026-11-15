# Lian full-family independent visual review — 2026-11-12

Scope: all 79 Lian animation families in `full-family-static-agent-5.json`, checked against the current native contact sheet `lian-sprites.png` and canonical `assets/chars/sprites/lian/04_special.png` at native and nearest-neighbor enlarged scale. This is static evidence only; no runtime or collision approval is implied.

Canonical anchor SHA-256: `248d8a4e6b75c14ea3528c9482301246d34efb05afcab5a8abc1f66f07c6d452`.

## Classification

Static pass (36): `idle`, `turn`, `walk`, `walkBack`, `run`, `hop`, `hitLight`, `hitMedium`, `hitHeavy`, `hitAir`, `fall`, `bounce`, `bounceAir`, `lieDown`, `dead`, `trip`, `win`, `lose`, `draw`, `intro`, `jab`, `strong`, `lkick`, `hkick`, `lotus`, `phoenix`, `flow`, `storm`, `hurl`, `lotusEx`, `stormUlt`, `stringA`, `stringB`, `stringC`, `powerUp`, `taunt`.

Correction required (41): `crouchStart`, `crouch`, `crouchEnd`, `crouchTurn`, `jumpStart`, `jumpUp`, `jumpFwd`, `jumpBack`, `jumpDown`, `land`, `guardStart`, `crouchGuardStart`, `airGuardStart`, `guard`, `crouchGuard`, `airGuard`, `guardEnd`, `guardHit`, `crouchGuardHit`, `airGuardHit`, `hitLowLight`, `hitLowMedium`, `hitLowHeavy`, `hitCrouch`, `getUp`, `airRecover`, `cjab`, `clk`, `sweep`, `ajab`, `akick`, `crane`, `super`, `dive`, `craneEx`, `defeat1`, `defeat2`, `defeat3`, `defeat4`, `lowKick`, `airLowKick`.

Unresolved (2): `petalFx`, `petalHit`; these are effect-only procedural families without a resolved body PNG.

## Findings

The canonical Lian identity is the black-haired woman with twin-bun/tie silhouette, green-and-gold tunic and cream trousers. The correction-required families retain the broad identity but show auxiliary poses with simplified or inconsistent hair, outfit and pixel finish relative to the anchor; the queued crouch, guard, airborne, recovery, attack, defeat and newly added low-kick frames therefore remain open for replacement and family-level review. `lian/crouch0.png` was previously integrated, but this does not close the affected crouch family or runtime sequence.

## Evidence and hashes

The exact ordered frame paths for all 79 rows and their current per-file SHA-256 values are authoritative in `full-family-static-agent-5.json` and `native-size-page-manifest.json`; the Lian subset contains the active resolved PNGs referenced by those rows. The canonical hash above pins the identity reference. `lian-sprites.png` is the reviewed native contact sheet; transformation and low-kick runtime captures are retained separately and do not substitute for this family audit.

Conclusion: Lian is not visually standardized. Only the 36 listed families pass static review; 41 require correction and 2 remain unresolved. No runtime or PR completion claim is supported.
