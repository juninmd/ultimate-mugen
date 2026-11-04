# Screenpacks

> 🇧🇷 Screenpacks são os menus, telas de seleção e barras de vida no formato M.U.G.E.N. Coloque o pacote em `data/` e rode o verificador abaixo.

A screenpack is the set of files that draws everything around a fight: title menu, character select, lifebars, fonts, intro and logo storyboards.

**Status:** `crates/formats/src/screenpack.rs` parses and validates these files, but the engine does not draw them yet. The title, select and fight screens are the native ones, and no screenpack is loaded at runtime until the screenpack work in the [parity tracker](https://github.com/juninmd/ultimate-mugen/issues/135) is done. The checker below only tells whether a pack parses.

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
