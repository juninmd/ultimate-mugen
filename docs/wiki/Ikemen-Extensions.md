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

| Trigger | Returns |
|---|---|
| `Map(name)` | Named value, 0 when unset |
| `RedLife` | Red life |
| `Dizzy`, `DizzyPoints`, `DizzyPointsMax` | Dizzy flag and points |
| `GuardBreak`, `GuardPoints`, `GuardPointsMax` | Guard break flag and points |

`data.dizzypoints` and `data.guardpoints` in the `[Data]` section default to `data.life`.

`data.dizzypoints` e `data.guardpoints` na seção `[Data]` usam `data.life` como padrão.
