# Suzume full-family independent visual review — 2026-11-12

Scope: all 83 Suzume families from `full-family-static-agent-3.json`, compared with `assets/chars/sprites/suzume/04_special.png` at native and nearest-neighbor 2×. The source report predates the integrated crouch/guard promotion, so the current active files were rechecked separately. This remains static evidence only; runtime timing, collision and transformation playback are not approved.

Canonical anchor SHA-256: `3621301628480a47e63dd77a0084d766f69da01d44bb419a532b73be0f9885be`.

## Current classification

Static pass (23): the original source-report passes `idle`, `run`, `hitLight`, `hitMedium`, `hitHeavy`, `hitAir`, `fall`, `bounce`, `bounceAir`, `lieDown`, `dead`, `trip`, `win`, `lose`, `draw`, `intro`, `powerUp`, `taunt`, plus promoted active `crouch0.png`, `crouch.png`, `crouch1.png`, `crouchGuard.png`, and `crouchGuardHit.png`. The five promoted files were reviewed at native/2× against the canonical anchor with shared palette, 400×400 canvas, binary alpha, axis (200,392), refHeight 320 and floor y=392.

Correction required (55): `turn`, `walk`, `walkBack`, `crouchStart`, `crouch`, `crouchEnd`, `crouchTurn`, `jumpStart`, `jumpUp`, `jumpFwd`, `jumpBack`, `jumpDown`, `land`, `hop`, `guardStart`, `crouchGuardStart`, `airGuardStart`, `guard`, `crouchGuard`, `airGuard`, `guardEnd`, `guardHit`, `airGuardHit`, `hitLowLight`, `hitLowMedium`, `hitLowHeavy`, `hitCrouch`, `getUp`, `airRecover`, `jab`, `strong`, `lkick`, `hkick`, `cjab`, `clk`, `sweep`, `ajab`, `akick`, `volley`, `swallow`, `super`, `judgement`, `spread`, `clone`, `subst`, `backstab`, `veil`, `lightningPalm`, `volleyEx`, `swallowEx`, `stringA`, `stringB`, `stringC`, `defeat1`, `defeat2`, `defeat3`, `defeat4`, `lowKick`, `airLowKick`.

Unresolved (5): `kunaiFx`, `kunaiHit`, `shadow`, `stormKunai`, `flashCat`; these are effect/procedural families without resolved body PNGs.

## Identity versus pose/finish

The correction queue consistently shows a wrong base-family appearance: masked dark-purple/red ninja frames replace the canonical purple-haired Suzume identity in most movement, defense, attack and defeat families. These are identity and outfit failures, not minor finish differences. The integrated crouch/guard files are the approved exception and were rechecked against the canonical purple-haired masked identity; they retain the target poses and common palette. `hitCrouch` remains correction-required because its candidate is isolated and not active.

The five unresolved effect families cannot be judged for body identity or pixel finish. All other correction rows need one-pose-at-a-time replacement and full sequence review; transformation form evidence does not clear the base family.

## Evidence and hashes

Ordered frame paths and current per-file SHA-256 values are authoritative in `full-family-static-agent-3.json` and `native-size-page-manifest.json`; the Suzume subset contains the active resolved PNGs. The canonical hash above pins the identity reference. Integrated crouch/guard promotion reports and native/2× contacts are under `integrated/suzume-crouch-before/` and `integrated/suzume-crouchguard-before/`. No active asset was changed during this audit.

Conclusion: the current active Suzume set is not fully standardized. Five crouch/guard frames pass after promotion, 55 families still require identity/pose correction, and 5 effect families remain unresolved. No runtime or PR completion claim is supported.
hitCrouch.png was subsequently integrated from the v2 canonical-kunai candidate; see suzume-hitCrouch-integration-20261112.json. The report's earlier candidate-pending count is historical; update family dispositions after the full audit refresh.
hitL1.png was subsequently integrated from the independently reviewed v2 candidate; see suzume-hitL1-integration-20261112.json. The original 23/55/5 disposition is pre-promotion; adjusted family counts are 25/53/5 pending consolidated report refresh.
hitL2.png was subsequently integrated from the independently reviewed candidate; see suzume-hitL2-integration-20261112.json. Pre-promotion 23/55/5 dispositions are adjusted to 26/52/5 after hitCrouch, hitL1 and hitL2 promotions, pending consolidated report refresh.
