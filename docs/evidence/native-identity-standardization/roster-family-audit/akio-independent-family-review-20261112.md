# Akio independent family review — 2026-11-12

Scope: all 81 Akio animation families in `full-family-static-review-current.json`, compared against `assets/chars/sprites/akio/04_special.png` and the visible Akio presentation in the native 400×400 and nearest-neighbor 2× contacts. This is a static review only; it does not approve runtime timing, collision alignment, or transformation playback.

Anchor SHA-256: `8667e3340ff9bc0b5df7f1202b76e2608e287a1a8cecc4fdc7080422e67014ae`.

## Disposition

- 58 families: static pass at contact/native review scale.
- 19 families: correction required, with the exact resolved source frames listed below.
- 4 families: unresolved because the resolver exposes effect/inline frames without a body source (`starOrb`, `starHit`, `novaBeam`, `novaHit`).

## Correction-required families

`jumpStart` (`jumpStart`), `jumpFwd` (`jumpUp`, `flip0`–`flip3`, `jumpTop`), `land` (`land`, `jumpStart`, `01_idle`), `hop` (`hop`), `hitLowMedium` (`hitL2`, `hitL1`, `01_idle`), `hitLowHeavy` (`hitL3`, `hitL2`, `hitL1`), `hitCrouch` (`hitCrouch`, `hitCrouch`, `crouch`), `getUp` (`14_lie`, `getUp1`–`getUp3`, `01_idle`), `airRecover` (`flip0`–`flip3`, `jumpDown`), `meteor` (`mh0`, `mh1`, `mh2`, `inline_cd8c2a8f4e49d4c8`), `super` (`nc0`, `inline_d02fd99cdded3e01`, `nc1`, `inline_2d3a94c49f5beaa2`, `sb2`, `01_idle`), `fallstar` (`jumpTop`, `mh1`, `mh2`, `jumpDown`), `stormSuper` (`nc0`, `nc1`, `sb1`, `sb2`, `cu1`, `jumpTop`, `01_idle`), `stormSuperUlt` (same source family as `stormSuper`), `stringC` (`hk0`, `hk1`, `01_idle`), `defeat1` (`dfLie`, `dfProp`, `dfProp2`), `defeat2` (`dfLie`, `dfSit0`, `dfSit1`), `defeat3` (`dfLie`, `dfKneel0`, `dfKneel1`), and `defeat4` (`dfLie`, `dfReach0`, `dfReach1`).

The correction disposition means the ordered family needs a new native-size pose/continuity review and likely replacement; it does not claim every listed PNG is a foreign identity. Repeated idle or fall frames are continuity defects or unresolved style evidence, not proof of an identity switch.

## Unresolved effects

`starOrb`, `starHit`, `novaBeam`, and `novaHit` have no resolved body PNG in the current inventory. They require an effect-specific review with their declared effect assets and cannot be cleared by comparing body sprites alone.

## Static passes

The remaining 58 families, including `idle`, locomotion, guard/hit basics, normal attacks (`jab`, `strong`, `lkick`, `hkick`, `cjab`, `clk`, `sweep`, `ajab`, `akick`), `lowKick`, `airLowKick`, and the ordinary projectile families, showed no confirmed identity switch in the current contact/native review. They still require the repository's final pixel-cluster and runtime gates before a roster-wide approval.

Evidence source: `full-family-static-review-current.json` (81 Akio rows and their resolved frame references). No assets were modified by this review.
