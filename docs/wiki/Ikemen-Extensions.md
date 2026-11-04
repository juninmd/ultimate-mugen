# Ikemen extensions / Extensões do Ikemen

State controllers and triggers from Ikemen GO that characters can use in CNS or ZSS.

Controllers e triggers do Ikemen GO que personagens podem usar em CNS ou ZSS.

| Controller | Parameters | Effect |
|---|---|---|
| `MapSet` | `map`, `value`, `type` (1 = add) | Sets a named value on the player |
| `MapAdd` | `map`, `value` | Adds to a named value |
| `RedLifeSet` / `RedLifeAdd` | `value`, `absolute` | Recoverable (red) life, kept between `life` and `lifemax` |
| `DizzyPointsSet` / `DizzyPointsAdd` | `value` | Stun points, `0..data.dizzypoints` |
| `DizzySet` | `value` | Dizzy flag |
| `GuardPointsSet` / `GuardPointsAdd` | `value` | Guard points, `0..data.guardpoints` |
| `GuardBreakSet` | `value` | Guard break flag |
| `Text` | `text`, `params`, `pos`, `velocity`, `scale`, `color`, `align`, `removetime`, `id` | Draws text on the fight screen; `%d`, `%f` and `\n` in `text` take values from `params` |
| `RemoveText` | `id` (omit for all) | Removes the player's texts |
| `TagOut` / `TagIn` | `self`, `stateno`, `ctrl`, `partnerstateno`, `partnerctrl` | Moves the player to or from standby (hidden, cannot be hit or pushed); `partnerstateno` swaps with the first teammate |
| `ModifyProjectile` | `id`, `velocity`, `accel`, `velmul`, `removetime`, `projhits`, `projpriority`, `projscale` | Changes the player's live projectiles |
| `ModifySnd` | `channel`, `pan`, `abspan`, `volumescale` | Changes sounds already playing on a channel |
| `TargetAdd` | `id` (player id), `targetid` (default: current HitDef `id`) | Adds the player to the target list without a hit; ignored for the player itself, its own helpers and parent and for duplicates. Normal target drop rules still apply: a target that is not in a hit state is dropped once the controller's state is 2 ticks old |
| `TargetRedLifeAdd` | `value`, `absolute`, `id` | Adds to the targets' red life (negative values are divided by the target's defence unless `absolute`) |
| `TargetDizzyPointsAdd` / `TargetGuardPointsAdd` | `value`, `id` | Adds to the targets' dizzy or guard points, clamped to `0..max` |
| `TargetScoreAdd` / `ScoreAdd` | `value`, `id` (target only) | Adds to the score of the targets' root / of the player's own root |
| `RoundTimeSet` / `RoundTimeAdd` | `value` in ticks | Sets or adds to the round timer, clamped to `0..100000`; reaching 0 ends the round through the normal time-over flow |

| Trigger | Returns |
|---|---|
| `Map(name)` | Named value, 0 when unset |
| `RedLife` | Red life |
| `Score` | Score of the player's root (`ScoreAdd`, `TargetScoreAdd`) |
| `Dizzy`, `DizzyPoints`, `DizzyPointsMax` | Dizzy flag and points |
| `GuardBreak`, `GuardPoints`, `GuardPointsMax` | Guard break flag and points |
| `StandBy` | 1 while tagged out |
| `NumText` | Texts the player has on screen |
| `ClsnOverlap(clsn1, id, clsn2)` | 1 when the player's box type overlaps the box type of the player with that `id` |
| `NumPartner` | Teammates on the same side |

`data.dizzypoints` and `data.guardpoints` in the `[Data]` section default to `data.life`.

`data.dizzypoints` e `data.guardpoints` na seção `[Data]` usam `data.life` como padrão.

PT-BR: `TargetAdd` registra um alvo sem golpe; `TargetRedLifeAdd`, `TargetDizzyPointsAdd`, `TargetGuardPointsAdd` e `TargetScoreAdd` alteram os alvos (filtro opcional `id`); `ScoreAdd` altera o placar do próprio jogador (trigger `Score`); `RoundTimeSet`/`RoundTimeAdd` mexem no cronômetro do round (ticks, limite 0..100000).
