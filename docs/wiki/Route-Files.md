# Route and unlock files (TOML)

Data-driven replacement for the Lua arcade paths and `unlock` conditions of Ikemen GO. See [Lua scripts](Lua-Scripts.md) for the reasoning. There is no scripting runtime: a route is a TOML file, and conditions use a small expression language checked when the file loads.

## Where the files live

| File | Used for | Source |
|---|---|---|
| `route_<fighter id>.toml` | Arcade route of that fighter | Data folder (next to the executable, app storage on Android, browser storage on the web) |
| `unlocks.toml` | Unlock and hide rules for fighters and stages | Same place; when absent the bundled `assets/routes/unlocks.toml` is used |

A file with an error is reported on the console with the step or rule number and then ignored, so the game keeps its built-in behaviour.

## Arcade route

```toml
[route]
id = "kfm-arcade"

[[step]]
storyboard = "chars/kfm/intro.def"

[[step]]
repeat = 6
fight = { order = 1 }

[[step]]
when = "consecutive_wins == 6"
fight = { p2 = ["SuaveDude"] }
else = { fight = { order = 3, winscreen = true } }

[[step]]
action = "clear"
```

Each `[[step]]` has exactly one of `fight`, `storyboard` or `action`.

| Key | Meaning |
|---|---|
| `fight` | `order` (1 picks the next regular opponent, 2 or more picks the boss), `p2` (fighter ids or names, first match wins), `stage`, `winscreen` |
| `storyboard` | Parsed and validated, not played yet: the engine reports it and continues |
| `action` | `clear` ends the route |
| `repeat` | 1 to 99 times; `when` is checked on every repetition |
| `when` | Condition; the step runs only when true |
| `else` | Same keys as a step body (`fight`, `storyboard`, `action`); runs when `when` is false |

The route is planned when arcade starts, assuming the player wins every fight, which is the only path a run can take. A lost fight ends the run as usual.

## Conditions

Numbers, quoted text, `true`, `false`, variables, `== != < <= > >=`, `&&`, `||`, `!` and parentheses.

| Variable | Value |
|---|---|
| `stats.arcade.clear` | Arcade clears saved so far |
| `consecutive_wins`, `match_no` | Fights already planned in this run |
| `mode` | `arcade`, `story`, `survival`, `cpu`, `versus`, `training`, `watch` or `team` |
| `netplay` | 1 in an online match, else 0 |
| `team_size` | 1 or 2 (routes only) |
| `player` | Id of the fighter the player chose |

An unknown variable is a load error that lists the known ones.

## Unlocks

```toml
[[unlock]]
target = "char:oboro"
when = "stats.arcade.clear >= 1"

[[hide]]
target = "char:oboro"
when = "mode == \"bossrush\""
```

A target is `char:<id>` or `stage:<id>`. A target with `unlock` rules is available only while all of them are true; a target with `hide` rules is hidden while any is true. Targets without rules stay open. Fighter rules apply to the select screen; stage rules are parsed but not applied yet.

## Not covered

Arbitrary logic, per-round results (`perfect_rounds`), labels and `goto`, and story mode arcs are not part of this version.
