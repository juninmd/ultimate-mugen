# MUGEN to UMF conversion checklist

What `mugen2umf.py` (skill `mugen-to-ume-migrator`) carries from a M.U.G.E.N fighter into a native UMF fighter, checked against Goku SSJ4. Every run writes `coverage.md` next to the JSON with the fate of each `Statedef`.

## Migrates today

| Area | How it lands in UMF | Checked |
|---|---|---|
| Sprites | every SFF sprite as PNG, 1:1 scale, axis kept | 198/198 |
| Sounds | the whole `.snd` as WAV | 19/19 |
| Animations and boxes | `.air` with `Clsn1`, `Clsn2`, `Clsn1Default` | all actions |
| Commands and moves | `Statedef -1` routes, `when` stand/crouch/air/fly, `cost`, `cancels` | 86 moves |
| Intro, win and taunt | states 170-199 as verbatim `scripts.cns`, own `Statedef 190` at round start | intro text and sound identical |
| Ki charge | `abilities.charge` on **R** (Start) with start, loop and end animations and sounds | sounds identical |
| Flight and fly moves | `abilities.fly` plus moves with `when: "fly"` | fly normals 300-340 |
| Teleports | multi-step `teleport`, or `abilities.teleport` when `p2dist` is used | travel within 3 px |
| Long cinematics | chains up to 12 states, timed exits, `landAt`, `followUp` hit/contact/hitNow | Meteor plays all 6 sounds |
| HitDef and projectiles | hits without cap, `recover`, `missTime`, guard, pause, sparks | ok |
| FX | Explod, AfterImage, EnvShake, EnvColor, SuperPause, NotHitBy | ok |
| Extras | `--extra` deep merge keeps detected abilities and adds forms (Limit Breaker), clones, passives, quotes | ok |

## Partial or approximated

- Custom throw states (Goku 616, 810, 820, 821, 1025-1028) are replaced by the UMF `throw` block.
- Charge speed follows the native gain ramp (0 to 3000 in 150 ticks, the original reaches 930).
- Damage is 59-74 % of the original because native combo proration scales hits.
- `AnimElem` triggers on looping animations map only the first loop.

## Not migrated yet

| Controller | Effect of the gap |
|---|---|
| `VelMul` | jumps in long cinematics go higher (Meteor -132 vs -103) |
| `PosFreeze` | the fighter is not pinned during a few frames |
| `PosSet` | absolute repositioning inside a move |
| `PalFX` | colour flash inside a move |

Dead code (states no command reaches) and shadowed states (a command already taken by an earlier route, so the state never fires in M.U.G.E.N either) are listed in `coverage.md` and skipped on purpose.

## Evidence

Run the original and the UMF side by side:

```sh
UME_CHAR_DIR=<mugen-folder> UME_TEST_UMF_JSON=<out>/<id>.json UME_EVIDENCE_DIR=<dir> \
  cargo test --release -p ume-engine --test umf_parity_evidence -- --nocapture
```

Results for Goku SSJ4 are in `docs/evidence/umf-converter/` (`parity.md` and one sheet per scenario: top row original, bottom row UMF).
