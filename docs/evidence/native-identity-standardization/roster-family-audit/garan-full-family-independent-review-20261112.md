# Garan full-family independent visual review — 2026-11-12

Scope: all 80 Garan animation families in `full-family-static-agent-2.json`, reviewed against `assets/chars/sprites/garan/04_special.png` at native and nearest-neighbor 2×. This is static evidence only; runtime timing, collision and transformation playback are not approved.

Canonical anchor SHA-256: `1079c6081d3c350989b7389790dc6a7a6ae22ea9c709d3479aad9c4d46cf8b0a`.

## Classification

Static pass (56): `idle`, `turn`, `walk`, `walkBack`, `crouchStart`, `crouch`, `crouchEnd`, `crouchTurn`, `jumpStart`, `land`, `guardStart`, `crouchGuardStart`, `airGuardStart`, `guard`, `crouchGuard`, `airGuard`, `guardEnd`, `guardHit`, `crouchGuardHit`, `airGuardHit`, `hitLight`, `hitMedium`, `hitHeavy`, `hitLowLight`, `hitLowMedium`, `hitLowHeavy`, `hitCrouch`, `bounce`, `bounceAir`, `jab`, `strong`, `lkick`, `hkick`, `cjab`, `clk`, `sweep`, `hellorb`, `super`, `snare`, `grasp`, `oniRush`, `hellorbEx`, `oniRushUlt`, `stringA`, `stringB`, `stringC`, `defeat1`, `defeat2`, `defeat3`, `defeat4`, `powerUp`, `taunt`, `lowKick` (plus the remaining resolved attack/utility rows recorded in the source manifest).

Correction required (12): `jumpUp`, `jumpFwd`, `jumpBack`, `jumpDown`, `hop`, `airRecover`, `ajab`, `akick`, `ascent`, `emberDive`, `ascentEx`, `airLowKick`.

Unresolved (12): `run`, `hitAir`, `fall`, `lieDown`, `getUp`, `dead`, `trip`, `lose`, `hellFx`, `hellHit`, `brimstone`, `pillarFx`.

## Identity versus pose/finish

The 12 correction-required families contain wrong-character sprite use: the small dark-clothed horned fighter appears in jump/air and several attack sequences instead of Garan's canonical red oni body, white mane, yellow horns and purple hakama. These are identity failures, not mere polish issues, and must be regenerated from `04_special.png` while preserving each target pose. The static passes retain Garan identity and readable pose/finish.

The unresolved body families (`run`, `hitAir`, `fall`, `lieDown`, `getUp`, `dead`, `trip`, `lose`, `brimstone`) are blocked by boundary-contact/alpha placement evidence; their identity was not marked wrong, but floor/edge geometry needs an independent native review. `hellFx`, `hellHit` and `pillarFx` are effect-only families with no resolved fighter PNG.

## Evidence and hashes

Ordered frame paths and per-file SHA-256 values are authoritative in `full-family-static-agent-2.json` and `native-size-page-manifest.json`; the Garan subset contains the active resolved PNGs. The canonical hash above pins the identity reference. No active asset or metadata was changed during this audit.

Conclusion: Garan is not visually standardized. 56 families pass static review, 12 contain wrong-identity frames and 12 remain unresolved. No runtime or PR completion claim is supported.

## Promotion update — `jumpTop`

`jumpTop.png` has since been replaced with the isolated canonical candidate documented in `garan-jumpTop-integration-20261112.json`. It is shared by `jumpUp`, `jumpFwd`, `jumpBack`, `ajab`, `akick` and `airLowKick`. A frame-level check confirmed `cjab0.png` and `ajab.png` themselves are canonical; the `ajab` family remains open because another resolved frame was wrong. `jumpDown` remains a separate unresolved correction. The 56/12/12 family counts above are the pre-promotion classification and require regeneration after all art changes.
