# Marlow full-family independent visual review — 2026-11-12

Scope: all 80 Marlow animation families in `full-family-static-agent-5.json`, reviewed against `assets/chars/sprites/marlow/04_special.png` at native and nearest-neighbor 2× using `marlow-sprites.png`. This is static evidence only; runtime timing, collision and transformation playback are not approved.

Canonical anchor SHA-256: `8f90d80b40e0fd560da681bd1c345d13d13109d1b1ee120782fc25abbc0fd1e3`.

## Classification

Static pass (53): `idle`, `turn`, `walk`, `walkBack`, `jumpStart`, `jumpUp`, `jumpFwd`, `jumpBack`, `jumpDown`, `land`, `run`, `hop`, `guardStart`, `guard`, `guardEnd`, `guardHit`, `hitLight`, `hitMedium`, `hitHeavy`, `hitAir`, `fall`, `bounce`, `bounceAir`, `lieDown`, `airRecover`, `dead`, `trip`, `win`, `lose`, `draw`, `intro`, `jab`, `strong`, `lkick`, `hkick`, `ajab`, `akick`, `spout`, `lunge`, `super`, `ebb`, `keelhaul`, `kraken`, `riptideEx`, `spoutEx`, `krakenUlt`, `stringA`, `stringB`, `stringC`, `powerUp`, `taunt`, `airLowKick`.

Correction required (24): `crouchStart`, `crouch`, `crouchEnd`, `crouchTurn`, `crouchGuardStart`, `airGuardStart`, `crouchGuard`, `airGuard`, `crouchGuardHit`, `airGuardHit`, `hitLowLight`, `hitLowMedium`, `hitLowHeavy`, `hitCrouch`, `getUp`, `cjab`, `clk`, `sweep`, `riposte`, `defeat1`, `defeat2`, `defeat3`, `defeat4`, `lowKick`.

Unresolved (3): `riptideFx`, `splash`, `maelstromFx`; these are effect-only/procedural families with no resolved body PNG.

## Identity versus pose/finish

No wrong-identity family was found. The correction-required families retain Marlow's canonical pirate identity: pirate face, dark tricorn, blue coat with gold trim, white shirt, red sash and boots. The defects are pose/finish consistency: queued crouch, guard, low-hit, crouching attack, recovery, defeat and low-kick frames have simpler rendering, scale or pixel-detail treatment than the anchor. They should be regenerated from the canonical identity while preserving each target pose and existing geometry.

The air-low-kick family is a static pass, while the ground low-kick remains correction-required; this does not establish runtime collision alignment. Existing runtime captures are separate evidence and do not replace this family review.

## Evidence and hashes

All ordered frame paths and current per-file SHA-256 values are authoritative in `full-family-static-agent-5.json` and `native-size-page-manifest.json`; the Marlow subset contains the active resolved PNGs. The canonical hash above pins the identity reference. `marlow-sprites.png` is the reviewed contact sheet. No active asset or metadata was changed.

Conclusion: Marlow is not fully standardized. 53 families pass static review, 24 require pose/finish correction, and 3 remain unresolved effects. No runtime or PR completion claim is supported.
