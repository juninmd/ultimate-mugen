# Ikemen extensions / Extensões do Ikemen

State controllers and triggers from Ikemen GO that characters can use in CNS or ZSS.

Controllers e triggers do Ikemen GO que personagens podem usar em CNS ou ZSS.

| Controller | Parameters | Effect |
|---|---|---|
| `MapSet` | `map`, `value`, `type` (1 = add) | Sets a named value on the player |
| `MapAdd` | `map`, `value` | Adds to a named value |
| `MapReset` | `map` (omit for all keys) | Removes one key, or every key, from the player's map |
| `TeamMapSet` / `TeamMapAdd` / `TeamMapReset` | same as `MapSet` / `MapAdd` / `MapReset` | Same operations on the map shared by every player and helper on the same side |
| `RootMapSet` / `RootMapAdd` / `RootMapReset` | same as `MapSet` / `MapAdd` / `MapReset` | Same operations on the root player's map |
| `ParentMapSet` / `ParentMapAdd` / `ParentMapReset` | same as `MapSet` / `MapAdd` / `MapReset` | Same operations on the parent's map; does nothing without a parent |
| `RootVarSet` / `RootVarAdd` | `var(n)`, `fvar(n)`, `sysvar(n)`, `sysfvar(n)` or `v` / `fv` + `value` | Writes variables on the root player, like `ParentVarSet` does on the parent |
| `RedLifeSet` / `RedLifeAdd` | `value`, `absolute` | Recoverable (red) life, kept between `life` and `lifemax` |
| `DizzyPointsSet` / `DizzyPointsAdd` | `value` | Stun points, `0..data.dizzypoints` |
| `DizzySet` | `value` | Dizzy flag |
| `GuardPointsSet` / `GuardPointsAdd` | `value` | Guard points, `0..data.guardpoints` |
| `GuardBreakSet` | `value` | Guard break flag |
| `Text` | `text`, `params`, `pos`, `velocity`, `scale`, `color`, `align`, `removetime`, `id` | Draws text on the fight screen; `%d`, `%f`, `%s` and `\n` in `text` take values from `params` (`%s` prints the mode name for a bare `TeamMode`, otherwise the integer) |
| `RemoveText` | `id` (omit for all) | Removes the player's texts |
| `TagOut` / `TagIn` | `self`, `stateno`, `ctrl`, `partnerstateno`, `partnerctrl` | Moves the player to or from standby (hidden, cannot be hit or pushed); `partnerstateno` swaps with the first teammate |
| `ModifyProjectile` | `id`, `velocity`, `accel`, `velmul`, `removetime`, `projhits`, `projpriority`, `projscale` | Changes the player's live projectiles |
| `ModifySnd` | `channel`, `pan`, `abspan`, `volumescale` | Changes sounds already playing on a channel |

| Trigger | Returns |
|---|---|
| `Map(name)` | Named value, 0 when unset (use `Root, Map(name)` or `Parent, Map(name)` to read another scope) |
| `TeamMap(name)` | Value in the team map of the player's side, 0 when unset |
| `RedLife` | Red life |
| `Dizzy`, `DizzyPoints`, `DizzyPointsMax` | Dizzy flag and points |
| `GuardBreak`, `GuardPoints`, `GuardPointsMax` | Guard break flag and points |
| `StandBy` | 1 while tagged out |
| `NumText` | Texts the player has on screen |
| `ClsnOverlap(clsn1, id, clsn2)` | 1 when the player's box type overlaps the box type of the player with that `id` |
| `NumPartner` | Teammates on the same side |
| `TeamMode` | 0 single, 1 simul (Parallel), 2 turns (Sequence), 3 tag (reserved, no tag mode yet). Also accepts `TeamMode = Simul`, `TeamMode = turns`, `TeamMode = 1` (names are case-insensitive) |

Map keys are case-insensitive. Each map holds at most 256 keys of at most 64 characters; writes that would exceed either limit are ignored.

Chaves de map não diferenciam maiúsculas. Cada map guarda no máximo 256 chaves de até 64 caracteres; escritas acima desses limites são ignoradas.

`data.dizzypoints` and `data.guardpoints` in the `[Data]` section default to `data.life`.

`data.dizzypoints` e `data.guardpoints` na seção `[Data]` usam `data.life` como padrão.
