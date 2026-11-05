# Screenpacks

> 🇧🇷 Screenpacks são os menus, telas de seleção e barras de vida no formato M.U.G.E.N. Coloque o pacote em `data/` e rode o verificador abaixo.

A screenpack is the set of files that draws everything around a fight: title menu, character select, lifebars, fonts, intro and logo storyboards.

**Status:** the runtime now loads a screenpack when one is present (`mugen/data/system.def`, `mugen/screenpacks/<pack>/system.def` or `UME_SCREENPACK`). The native UI stays as the fallback for anything a pack does not provide:

- `system.def` `menu.itemname.*` labels replace the main-menu titles (Arcade, Survival, teams, Training, Watch, Exit).
- `select.def` `[Characters]` order drives the select grid order.
- `intro.storyboard` text scenes are played over the arcade opening cutscene, timed by each scene's `end.time`.
- `fight.def` lifebar `pos`/`range.x` place and size the life bars (converted from the pack `localcoord` to the 426×240 HUD space); the rest of the HUD stays native.
- Parsed sprites/fonts/sounds are kept in `ume_formats::screenpack::Screenpack` for the remaining screens, but system/fight sprite drawing (backgrounds, bar sprites, fonts) is still native.

The checker below tells whether a pack parses; the game never refuses to start because of a broken pack (it falls back to the native UI and logs the warnings).

## Layout

```
data/
  system.def      title menu, select screen, music, storyboards
  select.def      roster grid and stage list
  fight.def       lifebars, power bars, round/KO announcers
  system.sff      sprites used by system.def
  system.snd      menu sounds
  fight.sff/.snd  lifebar sprites and announcer voices
font/
  *.fnt           bitmap fonts (v1 binary or v2 text)
```

File names in `[Files]` are looked up next to `system.def`, then in `data/`, `font/` and the game root.

## Supported files

| File | Sections read |
|---|---|
| `system.def` | `[Info] localcoord`, `[Files]`, `[Music]` (`*.bgm`, `.loop`, `.loopstart`, `.loopend`, `.volume`), `[Title Info] menu.itemname.*`, `[Select Info]` grid, every `[<name>BGdef]` / `[<name> N]` background |
| `select.def` | `[Characters]` (`randomselect`, `empty`, per-character `stage` and `key=value`), `[ExtraStages]`, `[Options]` |
| `fight.def` | `[Files]`, every `pN.*` bar part (`bg0`, `bg1`, `mid`, `front`, `counter`, `pos`, `range.x`) and any element by prefix |
| Storyboards | `[SceneDef] spr, startscene`, `[Scene N]` timings, fades, `clearcolor`, `layerall.pos`, `layerN.*`, `bg.name`, `bgm` |
| `.fnt` v1 | embedded PCX, `[Def] Size, Spacing, Colors, Offset, Type=Fixed/Variable`, `[Map]` with literal or `0xNN` chars |
| `.fnt` v2 | `[Def] Type=bitmap/truetype, Size, Spacing, Offset, File` |

## Elements

Every visual item follows the same pattern, so anything you can place in one screen works in another:

```
prefix.anim = 10
prefix.spr = 5,0
prefix.font = 1,0,1
prefix.text = "Round %i"
prefix.offset = 160,120
prefix.scale = 1,1
prefix.facing = -1
prefix.layerno = 1
prefix.displaytime = 60
```

## Checking a screenpack

```
cargo run -p ume-formats --example screenpack_check -- <game root> data/system.def
```

It prints what was loaded and one `warning:` line per missing or broken file. A screenpack with no `error:` line parses cleanly; displaying it in the engine is not implemented yet.
