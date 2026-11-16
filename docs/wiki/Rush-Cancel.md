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

## Whiff cancel

Pressing Start during a move that has not touched anything also cancels it, for a higher price. It needs 1000 meter (a Rush Cancel costs 500), works from 10 to 60 ticks into the move and shares the Rush Cancel cooldown. It returns control like a Rush Cancel but gives no Overdrive, no slow motion and does not count as a Rush Cancel. A move that has connected or been blocked still uses the cheaper Rush Cancel. Constants live in `crates/engine/src/rush/whiff.rs` (`WHIFF_COST`, `WHIFF_MIN_TICKS`, `WHIFF_MAX_TICKS`).

## Dash cancel

A connected normal (jab through heavy, standing or crouching) can be cancelled into a forward or back dash by double tapping the direction within 20 ticks of contact, for 250 meter. It is cheaper than a Rush Cancel, does not trigger Overdrive and does not use the cooldown. It needs contact (hit or guard) and enough meter, so a whiffed normal cannot be dash cancelled. Code: `crates/engine/src/rush/dash.rs`.

## Perfect guard

Pressing back within 6 ticks before a hit lands, with the guard not already held, makes the block perfect: no chip damage and the guard stun is a third shorter. Holding back early is a normal guard, and releasing back locks the window for 12 ticks, so mashing back does not work. It applies to grounded native fighters only and gives no meter (meter from guarding fed Shion's flight indefinitely). Code: `crates/engine/src/rush/perfect.rs`.

## Chase

During a connected attack (normal, special, not a super), double tap up within 30 ticks of the hit to teleport next to a juggled enemy for 500 meter. You land in the air with control, 38 pixels from them and facing them, so air combos keep going after a launcher. It needs the enemy airborne and in hitstun, so it never works on a standing or guarding opponent. Code: `crates/engine/src/rush/chase.rs`.

## Engine map

| Piece | File |
|---|---|
| State, buffer, constants | `crates/engine/src/rush.rs` |
| The cancel itself (`Match::rush_step`, `rush_ready`) | `crates/engine/src/rush/cancel.rs` |
| Cancel window query for the HUD (`Match::rush_window`: `None` when no cancel is possible, otherwise the share of the window left) | `crates/engine/src/rush/cancel.rs` |
| "RUSH CANCEL" marker with a shrinking window bar under the lifebar | `src/hud/rush_marker.rs` (`UME_SHOTS_ONLY=rush` captures it) |
| Event for the presentation | `Event::Rush { fighter, side, guarded }` in `crates/engine/src/matchs/types.rs` |
| Effects (rings, glow, sparks, tag, Overdrive embers) | `src/vfx/rush.rs` |
| Rollback checksum | `overdrive`, `cooldown` and `cancels` in `crates/netplay/src/checksum.rs` |
| Tests | `cargo test --release -p ume-engine --test rush_cancel` |

`rush_step` runs once per simulation tick after the input phase and is allocation-free (the afterimage trail only grows when a cancel happens). Everything is plain data on `Fighter::rush`, so `Match` stays `Clone` and replays and rollback are unaffected.
