# Character tuning

Settings → Gameplay → **Character tuning** changes per-character parameters without touching the original files. Values are percentages (100% = unchanged), kept in `char_tuning.toml` next to the executable (web: browser storage).

| Parameter | Applies to | Mechanism |
|---|---|---|
| Life | MUGEN and UME | `data.life` |
| Damage dealt | MUGEN and UME | `data.attack` (fighter attack multiplier) |
| Damage taken | MUGEN and UME | `tuning.defence` read by the fighter defence multiplier |
| Walk, run, jump | MUGEN and UME | `velocity.*` constants |
| Gravity | MUGEN and UME | `movement.yaccel` |
| Power bar | MUGEN and UME | `data.power` |
| Energy cost | MUGEN and UME | negative `PowerAdd` values scaled by `tuning.power_cost` |

Ranges are 25–300% (power bar and energy cost 0–300%). A character with all values at 100% is not written to the file. Reset a character with "Reset to default".

File format:

```toml
["native:kaito"]
life = 150
defence = 75
```

Code: `src/char_tuning/` (model, store, tests), `src/registry/tuning.rs` (applied when a character is stored), `src/app/char_tuning_screen.rs` (screen).
