# Remaking a M.U.G.E.N fighter as UMF

Recipe used for the Captain Marlow Reyes rebuild, based on the [dissection of four fighters](MUGEN-Dissection.md). It turns what a `.cmd`/`.cns`/`.air` pack does into data in one `assets/chars/<id>.json`. Art stays PNG only: a rebuild reuses the fighter's pixel-art frames and never draws a frame in code.

## Steps

1. Dissect the pack: `python3 tools/mugen2umf inspect <dir>` for files, commands, states and controller counts, then `... hitdefs <file.cns>` for the normal attack table.
2. List the features, not the code: normals and chains, one projectile per button, anti-air, throw, counter, teleport, helper or clone, form or power mode, supers. Each one has a row in the mapping below.
3. Pick the frames: only poses with a PNG in `assets/chars/sprites/<id>/` (`ls` the folder, check `meta.json`). A frame given as an inline joint pose object is procedural art and is not allowed in anims.
4. Write the anims as lists of `{pose, ticks, hit?}` with the tick counts from the dissection table, then the moves that use them. `at` is the one-based frame that carries the hit, projectile or velocity.
5. Keep the first special a quarter circle plus `x`, and charge inputs on ground-only moves, because the roster smoke test and the real-input audit depend on both.
6. Run the audits listed in [Dev tools](Dev-Tools.md), then capture the move showcase and a browser smoke test.

## Mapping from M.U.G.E.N to UMF

| M.U.G.E.N pattern | UMF data |
|---|---|
| `Statedef 200..610` with `HitDef` | `moves[]` with `kind: normal`, `when`, `hits[].pausetime`, `hits[].hitstun` |
| `trigger = movehit`, `numhelper` cancels | `cancels`, `rekka {after, window, contact}` |
| `Projectile` with `projhits`, `projpriority` | `projectile {at, speed, hits, priority, removeTime}` |
| Air fireball statedef | Automatic: every ground special gets an air state |
| `~60$B,F,a` charge | `"input": "~40$B, F, x"` on a ground-only move |
| `PosAdd p2dist x + 40` plus `Turn` (Shunpo) | `teleport: <frame>` on the move, `cost`, `invuln` |
| `ReversalDef` / `HitOverride` counter | `parry {window}` |
| `TargetBind` throw | `throw {damage, bind, launch}` or the cinematic `throw.steps` |
| Clone helper (Aizen) | `helpers.clones` (`count`, `maxActive`, `duration`, `attacks`) |
| `var(11)` power mode (Kaio Ken) | `abilities.forms` |
| `SuperPause`, `AfterImage`, `EnvShake` | `kind: super`, `fx`, `shake` |
| Constants `[Data]`/`[Velocity]` | `stats` (`life`, `walk`, `run`, `jump`, `airJumps`, `airDashes`) |

## Captain Marlow Reyes, rebuilt

What changed in `assets/chars/marlow.json`, all of it data:

| Change | Source of the idea |
|---|---|
| Every inline joint pose removed from the anims, 13 frames now use PNG poses (`rs0`, `ws1`, `ws2`, `al1`, `mf1`, `str1`, `cjab0`, `jumpDown`) | Sprites-only rule |
| Light normals pause 6/7 with 12 ticks of hit stun, heavy normals pause 11/12 with 15 | Normal attack table of the four packs |
| `tide_step` (`~D, DF, F, c`, 200 power): teleports behind the opponent on frame 3, 58 damage, cancels into `riptide` or `spout` | Aizen Shunpo and Kaname Flash Step; on the free `c` button (Aizen uses Z for his Shunpo) because `D, D, a` loses to the crouching normals and a half circle contains the quarter circles of the other specials |
| `flurry` (`~D, DF, F, a`): five light hits and a launching finisher while stepping forward | Goku SSJ 1050 rush |
| `broadside` (`~B, F, x`): three-hit projectile with priority 2, a quick back-forward motion instead of a charge because every special also gets an air version and the real-input audit cannot reach an air version of a 40-tick charge | Goku SSJ charge fireballs |
| `helpers.clones` `Phantom Crew` (`F, B, F`, 500 power, 300 ticks, one active): an autonomous copy that chases and jabs; a direction-only input because the summon block is compiled after every move, so any input that includes a button is taken first by a normal, and it only starts from the standing state | Aizen clone (state 700) |
| `stats.airDashes = 1` | Aizen and Gogeta air mobility |

Art status: no new art was made. Marlow's frame set has gaps for a rush and a teleport (the poses are reused and retimed), so a dedicated `flurry` and `tide_step` frame set is the open art task for the person who recreates the sprites.
