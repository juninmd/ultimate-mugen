# MUGEN to UMF conversion checklist

`tools/mugen2umf` (Rust crate, skill `mugen-to-ume-migrator`) migrates a M.U.G.E.N fighter to a UMF bundle with identical behavior. It no longer rewrites states as native UMF moves: the bundle carries the original data and the engine loads it through the same code as a M.U.G.E.N character (`ume_engine::loader::assemble_mugen_char`). See [UMF Character](UMF-Character.md#mugen-parity-exact-migration) for the format.

## What is carried

| Area | In the bundle |
|---|---|
| States and controllers | `mugen.cns`, `mugen.st`, `mugen.stcommon` verbatim (CNS and ZSS), including helpers, `Statedef -2/-3`, AI and custom throws |
| Commands | `mugen.cmd` verbatim: `[Defaults]`, every `[Command]` and the `Statedef -1` routes |
| Constants | parsed from the original `[Data]`, `[Size]`, `[Velocity]`, `[Movement]` with the same `localcoord` scaling |
| Animations | `mugen.anims`: every action with offset, ticks, flip, blend and alpha, scale, angle, every Clsn1/Clsn2 box, loopstart |
| Sprites | one indexed PNG per SFF sprite (same indices and palette), true color as RGBA, SFF order and duplicates kept for palette selection |
| Palettes | SFF palettes (`sffpal_<k>.png`) and `pal1.act`..`pal12.act`: palette swaps and `RemapPal`/`PalFX` see the same colors |
| Sounds | every `.snd` entry as `snd_g_i.wav` |
| UMF-only fields | `profile` (country, nationality, origin, bio, title, fighting style, affiliation in en/pt/es), `series`, quotes, `rivals`: `profile_facts` gathers the facts, the skill writes, `set_profile` validates |
| Improvements | `--extra`: move names (`mugen.names`), `mugenRoutes` for states the `.cmd` left without a command, new moves as `scripts` (CNS/ZSS) |

## Proof

`mugen2umf parity <original> <bundle>` (also run by `convert`) compares:

- data: states, commands, constants, animations, sprites (pixels and axes), indexed sprites and palettes, sounds, sizes;
- behavior: every `Statedef -1` command is played from its real input on the original and on the bundle, 150 ticks in the same match, and every tick of every fighter (state, animation, frame, position, velocity, control, life, power, helpers, sounds) must match.

The verdict is `IDENTICAL` or the first tick where they part. `cargo test --release -p mugen2umf` converts Kung Fu Man from `assets/mugen/elecbyte.zip` and requires `IDENTICAL`.

## Limits

What the engine does not support from M.U.G.E.N is listed in the bundle's `report.md` (engine diagnostics). It affects the original and the bundle the same way: fix it in the engine, not in the converter.
