# Garan current sprite-family audit

Scope: current Garan native PNGs, with native and nearest-2x review of hit reactions, ground/crouch/air guard, crouch jabs, turn, aerial poses, and run frames. The report distinguishes visual findings from structural checks and records active hashes.

## Corrections integrated

- airLowKick top hair crop: the active frame had 27 opaque mane pixels on row 0. The reviewed edge-v2 retouch preserved pose, scale, and axis and moved only the upper mane edge inside the canvas. It was subsequently integrated by the parent agent. Candidate and integrated active SHA-256: ee7c8da21dc0deef320da012bd3336357ca566596cc4cf12a4d6663a6c8cd766.
- sg_run_mid top hair crop: promoted after visual review. The candidate changed only alpha in rows 0–12, keeps all RGB values and body pixels after row 12 intact, and leaves metadata unchanged. Backup SHA-256: cf150843db3171c61a11c6d69d4d97abb581d92a5aacc73763c98018505b09b8. Promoted SHA-256: 685cacd4ae27e67c74f644f940afe6dfc269247636d1ce9d449a94b2e6ca06ef. Structural QA passes in the candidate folder.

## Remaining observation for later review

The run frames use axisY 363 and refHeight 360. sg_run_mid has an opaque bbox height of 357 px after the edge correction; sg_run_run0..3 measure 267, 271, 301, and 264 px. This apparent relative-size difference was documented but not changed by the isolated top-edge correction.

## Reviewed passes

- guard, guardHit, crouchGuard, crouchGuardHit, airGuard, airGuardHit: canonical Garan identity and readable guard poses.
- 09_hit1–11_hit3, hitL1–hitL3, hitCrouch, 12_fall1–14_lie: canonical identity and distinct readable hit/fall poses.
- cjab0, cjab1, and turn: canonical identity and readable poses. The earlier cjab concern is a false positive; no replacement is warranted.
- Current aerial body frames including jumpTop, jumpUp, jumpDown, hop, ajab, akick, airLowKick, and land read as canonical Garan. No confirmed wrong-identity frame remains in these reviewed poses.
- inline_c75becbcbc592dc9.png has an attack effect at the top edge. This audit could not isolate a body/hair crop from the effect, so it is not classified as a defect.

## Stale report reconciliation

The source report full-family-static-review-current.json is epoch NATIVE-ROSTER-2026-11-12-D and predates recent replacements. Its hashes differ from current active Garan sprites jumpTop, jumpUp, jumpDown, hop, airLowKick, and sg_run_mid. The first four are recent integrations; airLowKick is the integrated edge-v2 correction; sg_run_mid is the correction integrated in this audit. These are explained changes, not unexplained drift. Exact hashes are in current-family-audit.json. Refresh the old family-level correction flags against current sources before treating them as open failures. The effect-only hellFx, hellHit, and pillarFx rows remain unresolved because no body PNG could be reviewed.

## Evidence and limits

- current-family-audit.json records active frame hashes, geometry, and stale-report drift.
- Contact sheets: guard-hit-native.png, guard-hit-nearest-2x.png, crouch-cjab-turn-native.png, crouch-cjab-turn-nearest-2x.png, aerial-family-native.png, aerial-family-nearest-2x.png, run-family-native.png, and run-family-nearest-2x.png.
- The run candidate folder contains the exact backup, before/after native and nearest-2x comparisons, and promotion-structural-qa.json. Only structural PNG checks were run after promotion; no engine tests or runtime capture were performed.
