# Volta independent family review — 2026-11-12

Canonical anchor: `assets/chars/sprites/volta/04_special.png`, SHA-256 `d0b2eb39e2a5c178d816aff8f3184cae5d400a26fc90b1d827d94ac4ae91ed69`. Baseline: 81 animation families in `full-family-static-review-current.json`.

## Disposition

- 35 static pass, 42 correction-required, 4 unresolved (`boltFx`, `boltHit`, `coreFx`, `coreHit`; procedural-effect-only, no body PNG).
- Main body identity is the robust graphite/navy robot with lilac-grey plates, cyan visor/circuit lines and small yellow accents.
- Old families with pale grey/white armor and yellow helmet band remain active in crouch/guard/hit, walk, turn, jumps/land/hop, low hits, get-up and defeat sequences. They appear to be an older finish/outfit of the same robot; this does not prove a foreign identity.
- Exact candidate files and SHA-256 values for the correction targets are in `visual-family-correction-queue.json`; full paths/hashes are in `native-size-page-manifest.json`.

## Attack-family corrections to the current consolidated report

The current manifest's body review marks nine groups as static pass despite visible old-body sources: `jumpStart`, `jumpUp`, `jumpFwd`, `jumpBack`, `hop`, `overclock`, `super`, `relay`, and `relayUlt`. `od0/od1` and `tc0/tc1` show the pale old armor even when a separate cyan aura is present. Reclassify them correction-required and refresh their source hashes after art freeze.

Other old body frames in attack families include `crouch*`, `guard*`, `airGuard*`, `hitCrouch`, `hitL1/2/3`, `getUp1/2/3`, `df*`, `walk0–5`, `turn`, `jumpDown`, and `land`. Canonical-identity body families such as `arcbolt`, `chain`, `strong`, `lkick`, `hkick`, and `stringB/C` appear consistent at the reviewed contact scale; static classification does not prove runtime or timing.

## Jab integration integrity warning

The old page manifest contains 100 Volta PNG records: 98 hashes match current sources, while `jab0` and `jab1` are stale after palette correction. The newly integrated `jab2.png` is not yet in that manifest. Current active hashes are recorded in `volta-jab-integration-20261112.json`; regenerate the full page/hash manifest and Volta contacts after the final artwork is frozen. The prior contact sheets also predate the jab update, so they must not be cited for current jab appearance.

## Evidence and claim ceiling

Reviewed the full Volta contact sheet and four attack-family native/nearest-2× page pairs under `docs/evidence/native-identity-standardization/roster-family-audit/attack-families-valdric-volta/`. The independent review found the integrated jab sequence `jab0 → jab1(hit) → jab2 → idle0`, exact shared palette membership after the root palette fix, and a minor brighter-silver distribution in jab1/jab2. This review is static only; no Volta runtime, collision or transformed-form attack is approved.


## Promotion update

The `jab0/jab1/jab2` sequence has since been integrated and independently reviewed at native/2× size, with shared palette membership and unchanged hit timing. The old family disposition was `correction_required` before integration; reclassify the jab family after refreshing the consolidated manifest. Minor brighter-silver distribution in jab1/jab2 remains a style note, not an observed identity mismatch.
