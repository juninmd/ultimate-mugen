# Rush Cancel

Every fighter (native UMF and M.U.G.E.N) can **cancel a connected move into neutral** by spending half a meter bar. It is the engine's universal combo extender: no character data is needed.

## How to use

Press the energy key (**R**, the `s` / Start input) while the move you are performing has just hit or been guarded.

| Rule | Value |
|---|---|
| Cost | 500 power |
| Window | 1 to 45 ticks after contact (`CONTACT_WINDOW`) |
| Input buffer | The press is remembered for 6 ticks, so pressing a hair early still works |
| Cooldown | 40 ticks between cancels |
| Condition | `movetype = A`, the move hit or was guarded, the fighter is alive and the round is running |
| Not allowed | Whiffed moves, helpers, a fighter in the lying state, a frozen super pause |

The same key still charges energy when you are idle: a fighter with control and no attack in progress charges as before. After a cancel the key is masked until it is released once, so holding it never turns the cancel into a charge.

## What happens

1. 500 power is spent and the fighter changes to the neutral state of its stance (`0` standing, `11` crouching, `50` airborne) with control.
2. The attacker's hit pause and shake are cleared and the air-dash counter is refilled, so the next normal, special or dash can start at once.
3. Every rival that is in a hit or guard state receives 8 extra ticks of stun (`VICTIM_STUN_BONUS`), which is what lets the follow-up connect.
4. The match drops into slow motion for 15 ticks, the fighter flashes blue and leaves an afterimage trail.
5. **Overdrive** starts: for 150 ticks the fighter deals 20 % more damage (`effective_attack`, applied on top of passives and under their ceilings).

## Engine map

| Piece | File |
|---|---|
| State, buffer, constants | `crates/engine/src/rush.rs` |
| The cancel itself (`Match::rush_step`, `rush_ready`) | `crates/engine/src/rush/cancel.rs` |
| Event for the presentation | `Event::Rush { fighter, side, guarded }` in `crates/engine/src/matchs/types.rs` |
| Effects (rings, glow, sparks, tag, Overdrive embers) | `src/vfx/rush.rs` |
| Rollback checksum | `overdrive`, `cooldown` and `cancels` in `crates/netplay/src/checksum.rs` |
| Tests | `cargo test --release -p ume-engine --test rush_cancel` |

`rush_step` runs once per simulation tick after the input phase and is allocation-free (the afterimage trail only grows when a cancel happens). Everything is plain data on `Fighter::rush`, so `Match` stays `Clone` and replays and rollback are unaffected.
