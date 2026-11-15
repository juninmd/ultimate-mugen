# Bruno full-family independent visual review — 2026-11-12

Scope: every Bruno family in `full-family-static-agent-1.json` (79 rows), compared with `assets/chars/sprites/bruno/04_special.png` at native size and the existing `bruno-sprites.png` contact sheet. This is a static review only; it does not approve runtime timing, collision alignment, or transformation playback.

Identity anchor SHA-256: `ccca15b3a1b8529db9538dc0ccf1fa23d50ac87e74c0f95fabbe840a537486e9`.

## Disposition

Static pass (41): `idle`, `turn`, `walk`, `walkBack`, `crouchStart`, `crouch`, `crouchEnd`, `crouchTurn`, `jumpStart`, `jumpUp`, `jumpBack`, `run`, `guardStart`, `crouchGuardStart`, `airGuardStart`, `guard`, `crouchGuard`, `airGuard`, `guardEnd`, `guardHit`, `airGuardHit`, `hitLight`, `hitMedium`, `hitHeavy`, `hitLowLight`, `hitLowMedium`, `hitLowHeavy`, `hitAir`, `fall`, `dead`, `trip`, `win`, `lose`, `draw`, `intro`, `jab`, `strong`, `lkick`, `hkick`, `powerUp`, `taunt`.

Correction required (36): `jumpFwd`, `jumpDown`, `land`, `hop`, `crouchGuardHit`, `hitCrouch`, `bounce`, `bounceAir`, `lieDown`, `getUp`, `airRecover`, `cjab`, `clk`, `sweep`, `ajab`, `akick`, `bullrush`, `lariat`, `quake`, `super`, `cornerDive`, `ironClaw`, `slam`, `titanSuplex`, `quakeEx`, `lariatEx`, `titanSuplexUlt`, `stringA`, `stringB`, `stringC`, `defeat1`, `defeat2`, `defeat3`, `defeat4`, `lowKick`, `airLowKick`.

Unresolved (2): `quakeFx`, `quakeHit`. These are procedural-effect families without a resolved body PNG, so identity, clothing, scale and pixel-finish claims cannot be made.

## Visual findings

The contact sheet confirms the correction queue is substantive: multiple attack, recovery, crouch and defeat frames use a dark-haired/head-covered boxer variant with dark shin/knee guards and different linework, while the canonical Bruno is the short brown-haired fighter with bare torso, red trunks with gold trim, gold belt, white wraps and black boots. The mismatched frames are not cleared by structural 400×400 checks. The low-kick and air-low-kick additions are present, but their family appearance still requires the same canonical identity review.

## Frame and hash evidence

Every resolved frame path, ordered family membership and current SHA-256 is preserved in `full-family-static-agent-1.json` and `native-size-page-manifest.json`; the Bruno subset contains 97 unique PNG paths. This report intentionally records the exact anchor hash and disposition list while the machine manifests remain the source of truth for the per-file hashes, preventing stale duplicated hash tables. The reviewed contact sheet is `bruno-sprites.png`; before/after candidate sheets for prior Bruno experiments are retained beside this report.

Conclusion: Bruno is not yet visually standardized. The 41 static passes are limited to the listed families; the 36 correction-required and 2 unresolved families remain open, and no PR-level completion claim is supported.
