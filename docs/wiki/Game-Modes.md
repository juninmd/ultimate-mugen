# Game modes

**English** · [Português (BR)](pt-BR/Modos-de-Jogo.md)

The main menu offers single fights (Arcade, Versus CPU, 2 Players, Training, CPU vs CPU) and three team modes. Team size is chosen on the select screen with `Q` / `E`.

| Mode | Fighters in play | Round ends when |
|---|---|---|
| Team · Sequence | 1 per side, the next teammate enters after a KO; the winner keeps its life | one side runs out of fighters |
| Team · Parallel | the whole team at once (2 to 5); `Tab` hands your controller to the next teammate | every member of a side is KO'd |
| Team · Tag | 1 active + the rest on standby (2 or 3 per side) | every member of a side is KO'd |

## Arcade / Story Mode

`Arcade` in the main menu runs the Scarlet Eclipse Tournament story campaign:

- **Prologue (`Cutscene kind: 0`).** Each of the 15 native fighters has a unique opening narrative and personal vow before entering the tournament ladder.
- **Pre-Boss Confrontation (`Cutscene kind: 1`).** After defeating eight rivals, the player reaches the Scarlet Eclipse Hall and confronts the hidden final boss, **Mikazuki Oboro**, with character-specific dialogue.
- **Boss Phase 1 & In-Match Awakening.** Oboro fights with flight, shadow teleportation, super speed, and his first awakening (**Mangekyo Desperto**, `form = 1`). If Oboro loses a round during the match, he automatically breaks his seal at the start of the next round (`Match::awaken_defeated_boss`).
- **Boss Transformation Cutscene (`Cutscene kind: 3`) & Final Challenge (Phase 2).** Defeating Oboro the first time triggers the **Awakening: Absolute Eclipse** cutscene (`kind: 3`), launching Phase 2 (`Match::awaken_boss_phase(2)`) where Oboro starts in **Eclipse Absoluto** (`form = 2`) with boosted max life (`1.15x`), dark regeneration, accelerated meter gain, level-8 AI, and Phase 2 moves (*Corte do Eclipse Carmesim*, *Chama do Eclipse Eterno*, *Julgamento do Eclipse Absoluto*).
- **Climax Battle Cutscene (`Cutscene kind: 4`) & Supreme Awakening (Phase 3).** Defeating Oboro in Phase 2 triggers an animated supersonic combat clash cutscene (`src/app/cutscene/battle.rs`) with striking animations, weapon sparks, and speedlines where Oboro ascends into his ultimate celestial form: **Eclipse Primordial** (`form = 3`). Phase 3 features supreme stats (1.62x attack, 1.48x defense, 1.30x life), supernova cyan aesthetic, enhanced AI, and exclusive moves (*Colapso Primordial*, *Supernova do Eclipse Primordial*).
- **Finishing Strike Cutscene (`Cutscene kind: 5`) & Shattered Eclipse.** Defeating Oboro's supreme form triggers a climactic finishing blow cutscene where the hero delivers a supersonic decisive strike, an anime impact frame, shatters Oboro's cursed blade, and breaks the eclipsed moon into radiant golden dawn rays.
- **Character Ending Cutscenes (`Cutscene kind: 2`) & Main Menu Return.** Following the defeat cutscene, the player's character-specific ending cutscene plays (`src/app/cutscene/endings.rs`) with personalized victory epilogue narrative and returns directly to the Main Menu (`Screen::Menu`).

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
