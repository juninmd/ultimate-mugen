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

## Limits

UMF cannot yet express scripted finishers, stage interactions (pit) or AI command scripts. Life conditions are now available through `requires` (see [UMF character](UMF-Character.md)).
