# Story Mode Scripting (TOML)

Ultimate MUGEN's Story Mode is driven by a declarative TOML campaign script. You can customize every stage, wave, boss, dialogue beat, recovery rate, and credit count — or disable Story Mode completely.

## Loading Order

1. `UME_STORY_SCRIPT=<path>` environment variable (if set and readable).
2. `story/campaign.toml` in the working directory or next to the game executable.
3. Built-in default campaign (`assets/story/default.toml`).

If an external script fails to parse or is invalid, the engine falls back safely to the built-in default campaign.

## Disabling Story Mode

You can disable Story Mode in either of two ways:

- **In-game:** **Settings -> Gameplay -> Story mode (TOML) -> OFF**.
- **Via TOML:** Create `story/campaign.toml` with:

```toml
[campaign]
id = "disabled"
enabled = false
```

## Schema Reference

```toml
[campaign]
id = "my_campaign"
enabled = true
credits = 3
wave_recovery = 0.2
stage_recovery = 0.6
boss_life_factor = 1.5
boss_attack_factor = 1.1

[campaign.title]
en = "CYBER REBELLION"
pt = "REBELIÃO CIBER"
es = "REBELIÓN CIBER"

[[stages]]
arena = "neon"
accent = [0.4, 0.9, 1.0]
lane_far = -22.0
lane_near = 8.0
boss = "volta"
alt_boss = "akio"
waves = [
  ["npc_razor", "npc_spitfire"],
  ["npc_colossus"],
]

[stages.title]
en = "SECTOR ZERO"
pt = "SETOR ZERO"
es = "SECTOR CERO"

[stages.beats]
en = ["The grid is locked.", "Break the firewall."]
pt = ["A rede está trancada.", "Quebre o firewall."]
es = ["La red está bloqueada.", "Rompe el cortafuegos."]
```

- `title` and `beats` accept either a plain string / string list (used in all languages) or a `{ en, pt, es }` table.
- `alt_boss` is used automatically when the player picks the stage's primary `boss`.

## Story Studio (in-game editor)

Settings → Gameplay → **Story Studio** edits the active campaign without leaving the game: credits, life regained per wave and per stage, boss life and attack, and the four dialogue lines of each stage in the current language. Every change is validated, saved as TOML (`story-campaign.toml` in the app storage, localStorage on the web) and applied immediately. **Restore original script** deletes the saved copy.

Lookup order for the campaign: `UME_STORY_SCRIPT` → Story Studio save → `story/campaign.toml`.

### Epilogue editor

Story Studio → **Epilogues…** edits the ending of any fighter (or the `*` fallback): title, the boss line and hero reply on the rooftop, and each narration page, in the current language. Saves go to `story-epilogues.toml` (one `["fighter"]` table per edited epilogue, merged over the bundled `epilogues.json`); **Restore original epilogue** removes the override. A `●` in the header marks an edited epilogue.
