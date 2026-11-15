# Customization: comparison and proposal

Portuguese: [customization-comparison.pt-BR.md](customization-comparison.pt-BR.md).

## What other engines offer

| Engine | Customization surface | Cost for a modder |
|---|---|---|
| M.U.G.E.N | `system.def`, `fight.def`, `mugen.cfg`, `motif` files, sprite sheets | Many files, text formats, binary SFF editing, restart to see changes |
| Ikemen GO | Same plus Lua `main.lua`, `config.json`, modules, `motif.ini`, hot-reloadable scripts | Needs Lua and knowledge of the motif layout |
| OpenBOR | Per-entity text scripts, `models.txt`, module folders | Own scripting dialect, rebuild of paks |

## What Ultimate MUGEN offered

Settings through the in-game menu and `ultimate-mugen.cfg` (video, audio, rules, language), key remapping, screenpack loading for labels, select order and lifebar placement, native UMF JSON fighters, and the move generator. Missing: any way to restyle the interface or HUD without editing Rust, and any validation feedback for hand-edited files.

## Gaps

1. No theme data: menu background and HUD side colors are constants in code.
2. No hot reload: every edit needs a restart.
3. No validation: a typo in a config silently does nothing.
4. No single place where a modder starts.

## Proposal and order of work

1. **`tweaks.toml`** (this PR): one file next to the executable (localStorage on web), generated with defaults on first run, sections `[theme]` and `[hud]` with hex colors, reloaded within one second of saving, typo hints and line numbers in a toast and the log. No Lua, no rebuild.
2. Extend `tweaks.toml` with rules defaults, HUD layout offsets and menu labels, reusing the same parser and warnings.
3. A `mods/` folder with per-mod override files layered over the base game.
4. An in-game "Open tweaks" shortcut and a theme gallery.

Step 1 was picked first because it covers the largest gap (no data-driven look), needs no new UI, and creates the loader, hot reload and validation that every later step reuses.
