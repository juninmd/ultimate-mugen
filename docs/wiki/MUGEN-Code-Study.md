# M.U.G.E.N character code study

Read-only study of public M.U.G.E.N packages: Goku GT (Fighter Factory pack from the project folder) and four Mortal Kombat Project 4.9.3 fighters (Scorpion UMK3, Sub-Zero UMK3, Liu Kang, Shao Kahn) fetched from archive.org. Only text files (`.def`, `.cmd`, `.cns`, `.st`) were extracted and read; nothing was executed and no art, audio or code was copied into the repository.

## Package shapes

| Package | Code size | Statedefs | Layout |
|---|---|---|---|
| Goku GT | ~31k lines | ~175 | `cmd` + `cns` + `Especiales`, `Supers`, `SS1`..`SS4` (one file per form), `IA` for `Statedef -2/-3` |
| Scorpion UMK3 | 23.6k | 221 | `1.cmd`, `1.cns`, `Basic_Moves.st`, `Sp.st`, `Combos.st`, finishers |
| Sub-Zero UMK3 | 28.4k | 324 | same plus `2.st`, `3.st` (alternate moves), `Stage_Finishers.st` |
| Liu Kang | 19.4k | 161 | `Special_Punches.st` for the kick/punch specials |
| Shao Kahn | 27.0k | 262 | `Bmoves.st`, `Sp.st`, helpers (100 `Helper` controllers) |

## Patterns

1. **Short motion commands.** MK characters use 2 to 3 step inputs (`B,B,a`, `D,F,x`, `F,F,b`) instead of quarter circles, with `time` windows around 10 ticks. UMF accepts the same notation in `moves[].input`.
2. **Fast, long moves = helpers.** Scorpion's spear is a `Helper` (state 1010) spawned on `animelem = 4`, with `DestroySelf` at `time = 150`, and the owner waits in state 1000 until `p2stateno` is the pull state (1020/1021), then switches to 1001 to pull. In UMF this is a `projectile` plus a `throw` follow-up.
3. **Teleports.** Scorpion's 1100 (behind) and 1200 (forward) are plain `PosSet`/`PosAdd` states; UMF has `teleport` frame or `abilities.teleport`.
4. **Specials per button.** One motion, a different button picks the variant (Liu Kang `F,F` plus `x`, `a`, `b` or `y`: fireball, flying kick, bicycle kick, etc.). UMF equivalent: one move per button plus EX versions.
5. **Heavy explod use.** 440 to 530 `Explod` controllers per character for sparks, ice, fire and blood; UMF handles these through `spark`, `hitAnim` and `fx`.
6. **Combos stored in states.** `Combos.st` holds scripted chains. UMF models this with `cancels` routes.
7. **AI as input scripts.** 100+ commands named `AI001..` (long button sequences) drive the computer opponent. UMF uses the built-in AI instead of per-character command scripts.
8. **Finishers** (fatality, brutality, pit) are separate states keyed off round end; not migrated.

## Fast versus strong in these kits

- Fast: Sub-Zero slide (`B,F,a` kick) and rolls (`F,F,F`, `B,B,B`), Scorpion teleports and spear; low damage per hit, cancels into specials.
- Strong: Shao Kahn hammer and crab specials (`F,F,b`, `D,F,b`), armor through long recovery, command attacks (`B,F,y` run punch).

These map to the `lunge`, `falcondive`, `shoulder` and `meteordrop` moves added to the native roster, see [Fighter study](Fighter-Study.md).

## Census: how much of a package is repetition

`ume-tools mugen-census <zip-or-dir>...` (`crates/devtools/src/mugen_census.rs`) reads only the text files of a package (`.cns`, `.st`, `.cmd`, `.air`) and counts states, controllers, commands and animation actions. Three packages measured, nothing extracted or copied:

| Package | Code files | Code lines | Statedefs | Common states redeclared | Controllers | Commands | AI commands | AIR actions |
|---|---|---|---|---|---|---|---|---|
| Goku GT (DBFZ style) | 11 | 35092 | 329 | 56 | 3114 | 78 | 2 | 529 |
| Goku UI (DBFZ style) | 7 | 27742 | 258 | 68 | 2241 | 78 | 2 | 294 |
| Kung Fu Man (bundled) | 6 | 10646 | 118 | 10 | 711 | 74 | 0 | 234 |

| Controller | Uses | Share | UMF equivalent |
|---|---|---|---|
| `changestate` | 997 | 16% | generated from `cancels`, `when` and move `kind` |
| `playsnd` | 813 | 13% | `voice` and per-level hit sounds |
| `explod` | 552 | 9% | `fx`, `spark`, `hitAnim` |
| `velset` | 352 | 6% | `vel` on frames |
| `helper` | 311 | 5% | `projectile`, `throw`, abilities |
| `hitdef` | 272 | 4% | `hits[]` |
| `changeanim` | 249 | 4% | `anim` per move |
| `destroyself` | 224 | 4% | projectile `removeTime` |

What the numbers say:

1. **A fighter is 10k to 35k lines of code**, and the eight controllers above are 61% of all controller blocks. Every one of them is something UMF generates from a field instead of a hand-written state.
2. **Common states are copied and edited by hand.** Each package redeclares 10 to 68 of the engine's common states (stand, walk, jump, guard, get-hit) to tweak them. UMF fighters never touch them: the differences are fields in `stats` and `anims`.
3. **Variants are full copies.** EX and form versions of a move are whole new statedefs. In UMF a variant is one entry with `"use": "<move>"` and the fields that change.
4. **Nothing is shared between packages.** Two Goku packages by the same author family share no file. UMF has `extends` and kits, so a new fighter can inherit a whole fighter or a move pack and stay in sync with it.

The same measurement on the native roster (21 fighters, 652 moves): 355 moves have a body identical to a move of another fighter, and 1136 of 1535 animations equal the most common version of that animation. That duplication is what `extends`, `use` and kits remove, see [Reuse](UMF-Character.md#reuse-extends-use-and-kits). The example fighter `docs/examples/umf/mika.json` is 671 bytes and plays with 40+ moves.

## Limits

UMF cannot yet express scripted finishers, stage interactions (pit) or AI command scripts. Life conditions are now available through `requires` (see [UMF character](UMF-Character.md)).
