# Lua scripts in M.U.G.E.N content

Some packs ship `.lua` files. This page says what they are for, what the engine does when it finds them and what to use instead. The engine has no Lua runtime by design (see [Engine](Engine.md) and [Compatibility decisions](Compatibility-Decisions.md)): it keeps the binary small, WASM simple and the simulation deterministic for rollback.

## What was checked

None of the characters stored in the project (the two Goku GT DBFZ packs, Gogeta SSJ4, Goku SSJ4) contain a `.lua` file; they are plain CNS, CMD, AIR, SFF and SND. The notes below about Lua in other packs come from the Ikemen GO documentation, not from a pack in this repository. If a specific fighter ships Lua, send the pack so its scripts can be read and added here.

## Where Lua appears

M.U.G.E.N 1.0 and 1.1 have no Lua. It comes from Ikemen GO, which uses it in three places.

| Place | Purpose | Typical files |
|---|---|---|
| Engine and screenpack layer | Menus, game modes, select screen, win screen, hooks | `main.lua`, `start.lua`, motif and module files |
| External modules | Optional add-ons loaded by the engine: extra modes, stats, training tools | `external/mods/*.lua` |
| Content packs | Rare. A fighter or screenpack bundling a module that adds a menu, a mode or a HUD tweak | `*.lua` next to the `.def` |

Fighter logic is not written in Lua. Fighter logic lives in CNS and ZSS (state controllers, triggers, commands, AI), which the engine already runs. So a `.lua` file inside a fighter folder almost never changes how the fighter fights; it extends menus, modes or screens around it.

## What the engine does today

- The character loader does not read `.lua` files and does not list them as an error. The fighter loads and fights normally.
- If the fighter depended on its Lua (a custom menu entry, a mode, a HUD change), that extra behaviour is simply absent.
- `mugen_check` reports unsupported controllers, triggers and parameters, but not stray `.lua` files.

## Alternatives without Lua

| Lua use | Replacement |
|---|---|
| Menus and screens | Screenpack (`system.def`, `fight.def`) and the Rust menu code, see [Screenpacks](Screenpacks.md) |
| Game modes | Rust modes and data, see [Game modes](Game-Modes.md) |
| Fighter behaviour and AI | CNS or ZSS, or UMF JSON, see [CNS scripts](CNS-Scripts.md) and [Creating a UMF character](UMF-Character.md) |
| Custom rules and tweaks | Data fields (TOML or JSON) read by Rust |

## Recommendation

1. Keep Lua unsupported. Add no runtime.
2. Make the importer detect `.lua` files and report them once per character as "ignored: Lua scripts are not supported", so creators know why a menu or mode is missing.
3. Add the same line to the `mugen_check` report.
4. When a real pack depends on its Lua, read the script and port the effect to Rust or data case by case, instead of interpreting arbitrary Lua.
