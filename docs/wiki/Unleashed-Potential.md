# Unleashed Potential

Characters without their own transformation (imported M.U.G.E.N. and UME characters) get a generic one that needs no edits to their files. Press the transform button (`T`, the same as native awakenings) when the fighter is grounded, has control and the round is live.

| Step | Name (pt-BR / en) | Cost | Duration | Effect |
|------|-------------------|------|----------|--------|
| 1st | Potencial Liberado / Unleashed Potential | half a power bar | 12 s | attack x1.2, damage taken x0.9 |
| 2nd | Modo Fúria / Fury Mode | full power bar | 9 s | attack x1.4, damage taken x0.75 |

Each step is available once per round. A character has at most three transformations in total, its own forms included, so a character with three own forms gets none of these. Fury can only follow Unleashed Potential. Characters with their own forms (`forms` or the legacy anim 196) keep their own transformation and never get this one.

Visuals: the fighter's aura and tint change (blue for Potential, red for Fury), a short ring and tag play on activation, and the HUD shows a timer strip under the power meter. Native forms also show a form strip on the HUD.

Code: `crates/engine/src/potential.rs` (state and multipliers), `crates/engine/src/matchs/unleash.rs` (activation), `src/vfx/awaken.rs` (effects), `src/hud/passive_meter.rs` (strips). Test: `crates/engine/tests/potential.rs`.
