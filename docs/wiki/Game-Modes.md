# Game modes

**English** · [Português (BR)](pt-BR/Modos-de-Jogo.md)

The main menu offers single fights (Arcade, Versus CPU, 2 Players, Training, CPU vs CPU) and three team modes. Team size is chosen on the select screen with `Q` / `E`.

| Mode | Fighters in play | Round ends when |
|---|---|---|
| Team · Sequence | 1 per side, the next teammate enters after a KO; the winner keeps its life | one side runs out of fighters |
| Team · Parallel | the whole team at once (2 to 5); `Tab` hands your controller to the next teammate | every member of a side is KO'd |
| Team · Tag | 1 active + the rest on standby (2 or 3 per side) | every member of a side is KO'd |

## Arcade / Story Mode

`Arcade` in the main menu runs the Scarlet Eclipse Tournament story campaign with full cutscenes, dynamic villain progression, and secret fighter unlocks:

- **Prologue (`Cutscene kind: 0`).** Each of the 15 native fighters has a unique opening narrative and personal vow before entering the tournament ladder.
- **Dynamic Boss Rotation.** The 8-opponent ladder culminates in a dedicated final boss battle with tailored pre-boss dialogue cutscene and boss arena:
  - **Tier 0 (First Run):** The supreme architect **Mikazuki Oboro** (`oboro`) at Gate of Spirits (`spirit`).
  - **Tier 1 (After Beating Oboro):** **Garan the Ashen Oni** (`garan`) becomes the new main villain at the Volcanic Crater (`volcano`).
  - **Tier 2 (After Beating Garan):** **Valdric Morn** (`valdric`) ascends as the imperial tyrant final boss at Neo-Metropolis (`neon`).
  - **Subsequent Clears:** Rotates cyclically among the 3 main villains. If the player selects the active villain, the engine shifts to an alternate boss to prevent mirror fights.
- **Boss Empowerment & Oboro Phases.** When fighting as the arcade final boss, villains receive true boss status (`boss = true`, 2000 HP, 125% attack multiplier, boss AI level, power gauge auto-generation, aura VFX, and a dedicated `CHEFE FINAL` banner). When facing Oboro, Phase 1 features his awakened Mangekyo (`form = 1`); defeating him triggers the **Awakening: Absolute Eclipse** cutscene (`kind: 3`) and Phase 2 against **Eclipse Absoluto** (`form = 2`) with boosted life, passive regeneration, and exclusive dark moves.
- **Character Ending Cutscenes (`Cutscene kind: 2`) & Oboro Unlock.** Defeating the final boss plays a 5-page ending cutscene unique to the player's chosen character (`src/app/cutscene/endings.rs`). Upon completion, the game unlocks **Mikazuki Oboro** on the character select screen, advances the villain rotation, and returns directly to the character selection screen with an unlock toast announcement.

## Survival

`Sobrevivência` in the main menu is an endless run against the whole roster:

- **Opponents.** After you pick your fighter, the game picks the opponent each round (never the hidden boss, never a broken pack). The stage rotates every round.
- **Escalation.** The CPU level climbs every two fights, starting from your difficulty setting and capped at 8.
- **Life carries over.** The bar you end a fight with is the bar you start the next one with, plus the recovery configured under Gameplay › *Life recovered (Survival)* (0 / 20 / 35 / 50 / 100 %). A KO ends the run.
- **Score.** Each win banks the engine's own `Score`/`ScoreTotal` for your side plus a `1000 × round` clear bonus, so the number is deterministic and comes from the simulation itself.
- **Game over.** The end screen shows the score, how many fights you won and the local best; the best is persisted in `ultimate-mugen.cfg` (`survival_best`) and a new record is highlighted.
- **Replays.** The last fight is archived like any other and can be watched from the game-over screen (R button) or the Replays browser; the replay keeps the starting life so the re-simulation matches exactly.

Engine score stats used: `Fighter::score` and `Fighter::score_total`, filled by `HitDef.score` and the `ScoreAdd`/`TargetScoreAdd` controllers and read by the `Score`/`ScoreTotal` triggers.

## Two-player co-op (Arcade and Survival)

Press `V` on the character select screen (or click the **CO-OP 2P** button) to start Arcade or Survival with two
human players **on the same side**:

- P1 and P2 pick one fighter each; both are on the field together (2 versus 1) using the engine's simultaneous-team
  support, so the normal pushboxes, targeting and camera work apply.
- The side only loses the round/run when **both** fighters are KO'd (`team_simul` KO rule).
- Inputs are independent: P1 uses the player-1 keys/pad, P2 the player-2 ones (`Tab` is not needed). The HUD tags
  each fighter `P1` / `P2` by its pilot slot instead of by side.
- Arcade keeps both fighters across the ladder and swaps the opponent each round; Survival does the same each wave.
- Replays record both slots, so a co-op fight replays exactly.

Engine switches: `Match::team_simul`. Slots are added interleaved (`P1`, opponent, `P2`), so `roots()[0]` and
`roots()[2]` are the human pair.

## Tag team

Each side lists 2 or 3 fighters. Only the first one is in play at the start of every round; the others wait on standby.

- **Swap.** Press the Tag button (`F` for P1, Num Enter for P2; rebindable under Controls, action "Troca (Tag)"). On gamepads and touch the awakening button (R3 / SELECT) swaps instead, because awakening is disabled in this mode. The CPU swaps by itself when its fighter is below 30 % life and the partner is clearly healthier.
- **Cooldown.** After every swap the side waits `tag_cooldown_max` ticks (default 180, three seconds) before it can swap again. The HUD shows a bar and a `TAG` mark under each teammate's portrait.
- **Tag cancel window.** A swap is accepted only when the active fighter has control, or during the first `tag_cancel_window` ticks (default 30) after one of its attacks touched the opponent (hit or block). So you can cancel the recovery of a connected move into the partner, but not in the middle of a whiff or while being hit.
- **Standby fighters.** They are invisible, cannot be hit, pushed or targeted (`enemy`, `P2`, `EnemyNear` skip them), receive no input and keep their life. They follow the active fighter's position so they enter exactly where the swap happened, in state 0 with control.
- **KO.** When the active fighter dies, the next living teammate enters automatically after 50 ticks (cooldown ignored). The round ends only when every member of a side has no life left; the round timer decides by the summed life of the whole team.
- **Life and variables** of a swapped-out fighter are untouched; everybody starts the next round with full life.
- **Scripts.** `TeamMode` returns `3` in this mode. `TagOut` / `TagIn` swap the fighters for real (see [Ikemen extensions](Ikemen-Extensions.md)); `Standby` is `1` for the waiting fighters and `NumPartner` counts the teammates.
- **Replays** record the Tag button like any other input, so a Tag fight replays exactly.

Engine switches (`Match`): `team_tag`, `tag_cooldown_max`, `tag_cancel_window`. Events: `Event::Tag { side, fighter }`.
