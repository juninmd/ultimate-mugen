# MUGEN to UMF conversion checklist

What `tools/mugen2umf` (skill `mugen-to-ume-migrator`) carries from a M.U.G.E.N fighter into a native UMF fighter, checked against Goku SSJ4. Every run writes `coverage.md` next to the JSON with the fate of each `Statedef`.

## Migrates today

| Area | How it lands in UMF | Checked |
|---|---|---|
| Sprites | every SFF sprite as PNG, 1:1 scale, axis kept | 198/198 |
| Sounds | the whole `.snd` as WAV | 19/19 |
| Animations and boxes | `.air` with `Clsn1`, `Clsn2`, `Clsn1Default` | all actions |
| Commands and moves | `Statedef -1` routes, `when` stand/crouch/air/fly, `cost`, `cancels` | 86 moves |
| Intro, win and taunt | states 170-199 as verbatim `scripts.cns`, own `Statedef 190` at round start | intro text and sound identical |
| Ki charge | `abilities.charge` on **R** (Start) with start, loop and end animations and sounds, plus `gain` read from the `PowerAdd` of the loop (power per tick) | sounds identical, power 858 vs 930 in 150 ticks |
| Flight and fly moves | `abilities.fly` plus moves with `when: "fly"` | fly normals 300-340 |
| Teleports | multi-step `teleport`, or `abilities.teleport` when `p2dist` is used | travel within 3 px |
| Long cinematics | chains up to 12 states, timed exits, `landAt`, `followUp` hit/contact/hitNow | Meteor plays all 6 sounds |
| HitDef and projectiles | hits without cap, `recover`, `missTime`, guard, pause, sparks | ok |
| FX | Explod, AfterImage, EnvShake, EnvColor, SuperPause, NotHitBy | ok |
| Motion and palette | `VelMul`, per-tick `VelAdd` (own gravity, native gravity skipped), `PosFreeze`, `PosSet`, `PalFX` as `velMul`, `velAdd`, `posFreeze`, `posSet`, `palFx` | Meteor jump -103 = original |
| Palettes | `pal2`..`palN` `.act` files as `palettes[]` with an `exact` colour map against `pal1` (one entry per palette index that differs) | 4 palettes for Goku (pal4 and pal5 equal pal1) |
| Grabs | `TargetBind` windows, `PlaySnd`, `TargetLifeAdd`, the `TargetState` release and its `VelSet`/`LifeAdd` as `throw.steps`; the grab animation (`p1stateno`) is spliced after the startup animation | damage 156 vs 138, sound identical |
| Extras | `--extra` deep merge keeps detected abilities and adds forms (Limit Breaker), clones, passives, quotes; a move with an existing `id` only patches the fields it lists | ok |
| Helpers | `Helper` → `spawn`; the helper state and everything it calls (`ChangeState`, `HitOverride`, `HitDef p1stateno/p2stateno`) passes verbatim, renumbered from 70000 | ok |
| Global states | `Statedef -2` / `-3` verbatim; `Parent,StateNo = N` pointing at a converted move becomes `@move(id)` | ok |
| Rekka / hold / release | in-move `ChangeState` on `command` → child move with `rekka`, or `followUp on: "command"` / `"release"` | ok |
| Ground/air controllers | a controller only valid in the air (`StateType = A`) is left out of the ground move and the other way round | ok |
| Own hit sounds | `hitsound = S g,i` → `hitSound` + `ownSound: true` | ok |
| SFF v1 shared palettes | `mugenversion = 1.x`: shared sprites follow the previous sprite's palette (super backgrounds, effects); WinMUGEN keeps the `.act` rule | ok |
| Commented-out routes | `mugenRoutes` in the extra file unlocks a state the `.cmd` left commented (Black Widow S.H.I.E.L.D. hyper) | ok |

## Partial or approximated

- Throws keep their `TargetBind`/`TargetState` choreography as `throw.steps`; `Turn`, `PosAdd` on the victim and the post-throw states 1025-1028 (wall bounce chain) still use the engine defaults.
- Charge gain is a constant per tick measured from the loop. The original is a 13-tick burst per 24-tick animation, so the meter fills in the same time but not in the same steps.
- Palettes recolour only sprites that use the shared `pal1`. Sprites with their own embedded palette keep their colours, and two indices with the same `pal1` colour map to the first index's target.
- Damage is 59-74 % of the original because native combo proration scales hits.
- `AnimElem` triggers on looping animations repeat with every pass only for sounds, Explods and shakes; a repeated `HitDef` is not guessed.
- Black Widow: 32 of 43 routes match the original sounds and damage; the rest are the hypers' looping cinematics, the jump-back Widow's Strike chain and the custom throw.

## Not migrated yet

Controllers with a condition the converter cannot pin to a frame (for example `PosSet` when `Pos Y < -175`) are listed in `coverage.md` as dropped. Goku SSJ4 has none inside its moves.

Dead code (states no command reaches) and shadowed states (a command already taken by an earlier route, so the state never fires in M.U.G.E.N either) are listed in `coverage.md` and skipped on purpose.

## Evidence

Run the original and the UMF side by side:

```sh
UME_CHAR_DIR=<mugen-folder> UME_TEST_UMF_JSON=<out>/<id>.json UME_EVIDENCE_DIR=<dir> \
  cargo test --release -p ume-engine --test umf_parity_evidence -- --nocapture
```

Results for Goku SSJ4 are in `docs/evidence/umf-converter/` (`parity.md` and one sheet per scenario: top row original, bottom row UMF).

Every listed move, original state against the UMF move (`routes.json` written by `convert`), plus moves that exist only in the UMF:

```sh
UME_CHAR_DIR=<mugen-folder> UME_TEST_UMF_JSON=<out>/<id>.json UME_EVIDENCE_DIR=<dir> \
  UME_EVIDENCE_MOVES=qcf_x,qcb_x UME_EVIDENCE_EXTRA=new_move \
  cargo test --release -p ume-engine --test umf_route_parity -- --nocapture
```

Black Widow results are in `docs/evidence/umf-converter/black-widow/` (`route_parity.md` and sheets).
