# Dissection of four M.U.G.E.N fighters

Read-only study of four public packs, taken apart file by file with `tools/mugen2umf`: Aizen TYBW (Fighter Factory Ultimate, SFF v2), Gogeta SSJ4 (Pocket_Danyell and Sarevok), Goku SSJ GT (Yamcha Z) and Goku SSJ4 (_Vision). The packs are third-party files: nothing from them is committed, only what we learned. Companion studies: [M.U.G.E.N code study](MUGEN-Code-Study.md) and [Fighter study](Fighter-Study.md).

## Size of each pack

| | Aizen TYBW | Gogeta SSJ4 | Goku SSJ GT | Goku SSJ4 |
|---|---|---|---|---|
| Sprite format | SFF v2.01, 1380 sprites, 696 palettes, 29 MB | SFF v1, 523 images | SFF v1, 116 images | SFF v1, 198 images |
| Sounds | 272 (19 groups, up to 4 MB each) | 33 | 5 | 19 |
| AIR actions | 262 | 232 | 84 | 171 |
| Clsn1 / Clsn2 boxes | 83 / 1538 | 301 / 1202 | 115 / 477 | 309 / 1469 |
| CMD commands | 67 | 65 | 62 | 65 |
| Statedefs (own, no common) | 85 + 68 + 19 + 2 | 101 + 2 | 29 | 109 + 5 + 2 |
| Controllers (own) | 650 + 676 + 173 + 57 | 789 + 117 | 223 | 761 + 30 + 25 |
| Life / attack / defence | 2250 / 115 / 125 | 2600 / 110 / 140 | 2100 / 120 / 110 | 2400 / 130 / 130 |

Take-aways: Clsn2 outnumbers Clsn1 about 5 to 1 everywhere, because the body is split into 4 to 6 small boxes per frame (head, torso, arms, legs) and the attack is one to three boxes. This is the same rule as the native hit box audit.

## File layout

| File | Role | Seen in |
|---|---|---|
| `.def` | `[Info]` name and author, `[Files]` list, `pal1..pal12`, `pal.defaults` | all |
| `.cmd` | `[Remap]`, `[Defaults]` (`command.time`, `command.buffer.time`), `[Command]` list, `[Statedef -1]` input to state routing | all |
| `.cns` | `[Data]`, `[Size]`, `[Velocity]`, `[Movement]` constants then states | all |
| `st1..st3` extra state files | `IA.cns` (`-2`/`-3` AI), `Especials.cns`, `Supers.cns`, `-final`, `-story-intros` | Aizen, Goku SSJ4 |
| `common1.cns` | Copy of the shared states, overridden by redeclaring a state number | all |
| `.air` | Actions: frames `group,image,x,y,ticks[,flip,trans]`, `Clsn1` (attack) `Clsn2` (hurt), `Clsn2Default`, `Loopstart` | all |
| `.sff` / `.snd` | Sprites and sound banks, referenced by `group,image` and `group,index` | all |
| `.act` | 256-colour palettes, one per colour option | Gogeta, Goku SSJ4 |

## Constants worth copying

- Walk 2.3 to 2.4 forward, run 4.6, jump `-8.1` to `-10.1`, `yaccel` .43 to .44, `airjuggle` 15, ground width 15 (10 for the thin Aizen), height 60, `attack.dist` 160. The native roster already uses the same scale.
- `fall.defence_up` 50, `liedown.time` 60, `sparkno` -1 when the pack draws its own sparks.
- Aizen sets `airjump.num = 6` and a walk of 7: a fast "floaty" movement type, built from constants only.

## Command layer

- Quarter circles `~D, DF, F, a`, half circles `~B, DB, D, DF, F, a`, charge `~60$D,U,a` and `~60$B,F,a` (Goku SSJ), double taps `FF`, `BB`, `~D,U` for super jump, 3-button combos `a+b`.
- Button routing: `/$B,x`, `/$D,y` style commands hold a direction while pressing, giving command normals.
- Buffers: `command.time = 10` and `buffer.time` 1 in most packs, 15 to 30 for the packs that favour easy inputs.
- Specials keyed off one motion, one button each: Aizen `D,DF,F` with a, b or c is Silent Slash, Dark Waves or Kurohitsugi. UMF does the same with one move per button.
- Gogeta and Goku SSJ4 keep `ai..ai13` commands (button strings) that only set an AI flag in `-2`; the engine has its own AI, so they are not migrated.
- Cancels are written as extra triggers on the target move: `trigger2 = numhelper(700) > 0` (Aizen cancels into specials while his clone lives), `movehit` and `movecontact` (165 uses in Aizen, 22 in Gogeta, 14 in Goku SSJ4). UMF names the same idea `cancels` and `rekka`.

## Normal attack numbers

Standing light (200), standing heavy (210), crouching light (400), crouching heavy (410), jumping light (600), jumping heavy (610). Values are `damage` of the hit, `pausetime` of attacker and defender, `ground.hittime`, `juggle` points and `poweradd`:

| State | Gogeta | Goku SSJ | Goku SSJ4 | Aizen |
|---|---|---|---|---|
| 200 stand light | 20, pause 4,5, hit 11, juggle 4, power 22 | 23, pause 7,8, hit 11, juggle 1, power 20 | 23, pause 5,8, hit 7, juggle 1, power 20 | 20, pause 9,9, hit 15, juggle 2, power 40 |
| 210 stand heavy | 55, pause 4,5, hit 11 | 55, pause 11,11, hit 15, juggle 4, power 60 | 63, pause 12,12, hit 15, juggle 5, power 65 | 30, pause 9,9, hit 15 |
| 400 crouch light | 23, pause 10,11, `guardflag = L` | 23, pause 10,12, `L` | 23, pause 5,8 | 20, pause 0,15 |
| 410 crouch heavy | 37, pause 12,12, `M` | 62, trip, `fall = 1`, juggle 7 | 37, pause 12,12, `M` | 30, pause 0,60 |
| 600 jump light | 20, pause 7,8, `HA` | 12, pause 7,8, `HA` | 12, pause 7,8 | 30, pause 10,10 |
| 610 jump heavy | 72, pause 12,12, hit 17 | 70, pause 9,10 | same values as 600 | no direct HitDef |

Rules the packs follow: light normals pause 4 to 8 ticks and give 15 to 25 power, heavy normals pause 11 to 12 and give 55 to 70, ground `hittime` 11 to 15, crouching lows use `guardflag = L` and `ground.type = Low`, jump normals use `HA`, and the heavy crouch is the sweep (`Trip` plus `fall = 1`, Goku SSJ).

## Move types and how they are built

| Type | Example | How it is built |
|---|---|---|
| Fireball | Goku SSJ 1000 | Statedef `movetype = A`, `Projectile` at `Time = 14` with `ProjAnim`, `ProjHitAnim`, `velocity = 5.5,0`, `projpriority`, own `HitDef` params, `AnimTime = 0` back to 0 |
| Air fireball | Goku SSJ 1005 | Same, `type = A`, `physics = N` until the throw, `ProjHeightBound` to remove it on the floor |
| Beam super | Goku SSJ 1800 | `poweradd = -2000`, `SuperPause` on `AnimElem = 1`, `AfterImage` for the whole move, three `Projectile` controllers on the same frame: a visual with `projremovetime`, a long one with `projhits = 30`, `projmisstime = 5`, `damage = 19,4` |
| Multi-hit rush | Goku SSJ 1050 | One long action, `VelAdd` pairs on alternating `AnimElem` (+4 then -4), `EnvShake` on the last element |
| Throw | Goku SSJ4 1601 | `TargetBind` every element, `TargetLifeAdd` at `AnimTime = 0`, `TargetState` to a wake-up state |
| Clone | Aizen 700 | A `Helper` with `anim = 9999`, `DestroySelf` at `time = 15` or when the root is gone, and `numhelper(700)` used as a cancel trigger by every attack |
| Hit spark | Aizen 995 | `Helper` on `movehit = 1 && numhelper(995) = 0`, `postype = p2`, scaled by `Helper(999999)` variables |
| Counter | Aizen Kyouka Suigetsu | `ReversalDef` or `HitOverride` window, then a state that teleports with `PosSet` |
| Teleport | Aizen Shunpo | `PosAdd` by `p2dist x + 40` then `Turn` (the exact code UMF emits for `teleport`) |
| Form | Gogeta Kaio Ken | `var(11)` flag, `-2` adds power each tick, `AttackMulSet 1.2`, `DefenceMulSet 1.2`, `Explod` aura every 10 ticks |
| Power up | Goku SSJ 730, 735, 740, 741 | Override of the common power-up states, `PalFX` plus `EnvColor` and sound |

Aizen is the only pack that builds almost everything from helpers: 98 `Helper` controllers, 140 `Explod`, 128 `AssertSpecial` (`NoShadow`, `Invisible`), 48 `AngleDraw`. Goku SSJ is the opposite: no helper, no explod, 12 `Projectile`. Gogeta sits between them with 32 `Projectile` and 109 `Explod`.

## Controller frequency

| Pack | Top controllers |
|---|---|
| Aizen | PlaySnd 142 + 122, ChangeState 68 + 46, Explod 57 + 57, Helper 37 + 41, DestroySelf 37 + 27, HitDef 30 + 28 |
| Gogeta | ChangeState 145, Explod 109, PlaySnd 95, HitDef 38, VelAdd 37, Projectile 32, EnvColor 21, SuperPause 18 |
| Goku SSJ | VelAdd 38, HitDef 37, ChangeState 34, Projectile 12 |
| Goku SSJ4 | PlaySnd 137, ChangeState 135, HitDef 71, VelSet 63, AfterImage 29, AfterImageTime 24, Explod 24 |

## Sound and sprite organisation

- Sprites: group 0 idle, 20 walk, 40 jump, 100 and 105 turn and crouch start, 200 to 240 stand attacks, 400 crouch, 600 jump, 5000 to 5120 get-hit, 9000 portrait, 9000,1 small portrait. Specials use free groups from 1000 up. Goku SSJ4 keeps 121 groups for 198 images, Gogeta has 523 images in 109 groups (63 linked, sharing pixels).
- Sounds: group 0 attack whooshes and voices, 5 hits, 6 guard, 11 KO, 20 and above for specials. Aizen has 104 sounds in group 0 alone.
- Palettes: `pal1..pal12` in the `.def`; Goku SSJ4 ships 5 `.act`, Gogeta 5, Aizen embeds its palettes in the SFF v2.
- Timing: a normal is 8 to 17 ticks in total (Goku SSJ4 stand light 4 + 11 + 2, Aizen 2 + 2 + 4 + 4, Gogeta stand heavy 2 + 4 + 2); the hit frame is the middle one.

## What the dissection changed in UME

1. Native fighters must read like these packs: chain normals, rekka follow ups, command normals, air versions, one projectile per button, teleport, counter and a form.
2. Hit and hurt boxes follow the limb: 3 hurt boxes minimum, attack boxes only where the frame has pixels (see the native hit box audit).
3. Frame data targets the table above: light 4 to 8 tick pause, heavy 11 to 12, ground hit time 11 to 15.
4. The Captain Marlow rebuild (see [Characters](Characters.md)) is the first fighter made with this method, see [Remaking a M.U.G.E.N fighter as UMF](Remaking-a-MUGEN-Fighter.md).
