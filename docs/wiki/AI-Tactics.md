# CPU tactics

`crates/engine/src/ai.rs` picks moves by distance and dice; `crates/engine/src/ai/tactics.rs` adds reactions for CPU level 3 and above. Works for native (UMF) and M.U.G.E.N fighters alike, because it only reads commands, the move list and fighter state.

| Tactic | Trigger | Action |
|---|---|---|
| Anti-air | Rival airborne, descending (`vel.y > 0`) and within ~105 units | Picks a usable command whose motion starts `F, D, DF` (dragon punch) |
| Whiff punish | Rival attacking, no contact yet, state time over 6 ticks, just out of reach (0.8 to 2.0 times the near range) | Picks a multi-step (ranged) command |
| Hit confirm | Own attack connected (`move_hit > 0`) and control not back yet | Picks a multi-step command; the compiler's cancel rules let it connect |
| Meter discipline | Every random or reactive pick | Needs `power >= cost`; EX moves keep one bar in reserve; supers only while the rival is in hit-stun, below 40 % life, or the fighter is below 50 % life |

Each reaction fires with probability `0.35 + 0.6 * skill` and sets a short cooldown.

## Evidence

`cargo run --release -p ume-engine --example native_duel` (21 fighters, 1260 rounds, level 6, 3 seeds): see `docs/evidence/ai-tactics/`.

| | Before | After |
|---|---|---|
| Knockout rate | 83.4 % | 96.7 % |
| Average round | 70.9 s | 54.5 s |
| Damage per round | 1388 | 1527 |
| Win-rate spread | 24.7 pts | 23.2 pts |
| Rounds with stuck fighter | 0 | 0 |

Tests: `cargo test --release -p ume-engine --test ai_tactics` and the unit tests in `ai/tactics.rs`.
