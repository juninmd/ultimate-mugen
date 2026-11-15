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
| Fighter DEF | `arcadepath` in `[Arcade]`: that fighter's own arcade route, storyboards and rivals | `arcade.lua` next to the `.def` |
| Content packs | A screenpack or pack bundling a module that adds a menu, a mode or a HUD tweak | `*.lua` next to the `.def` |

Fighter combat logic is not written in Lua. It lives in CNS and ZSS (state controllers, triggers, commands, AI), which the engine already runs. So a `.lua` file in a fighter folder, such as its arcade path, does not change how the fighter fights; it shapes the modes and screens around it.

## Examples from the Ikemen GO wiki

Source: the official Lua page of the Ikemen GO wiki. Reference modules live in the `ikemen-engine/modules` repository.

A fighter can point to a Lua file from its own DEF, under the `[Arcade]` section. The file is a custom arcade path: it decides the opponents, storyboards and branches for that fighter.

```ini
[Arcade]
arcadepath = arcade.lua
```

```lua
launchStoryboard('chars/kfm/intro.def')
for i = 1, 6 do
	if not launchFight{} then return end
end
if getConsecutiveWins(1) == 7 then
	if not launchFight{p2char = {"SuaveDude"}} then return end
else
	if not launchFight{order = 3, winscreen = true} then return end
end
launchStoryboard('chars/kfm/ending.def')
setMatchNo(-1)
```

`select.def` accepts a Lua condition after `unlock =` to unlock a hidden fighter, for example `stats.modes ~= nil and stats.modes.arcade ~= nil and stats.modes.arcade.clear >= 1`, or `gameMode() ~= "bossrush"` to hide a fighter from one mode.

Modules are `.lua` files in `external/mods`, listed in `Common.Modules`, or loaded by a screenpack with `module = file.lua` in its `[Files]` section. They can add game modes and attach to hooks in the default scripts.

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

The TOML replacement for arcade paths and unlock conditions is described in [Route files](Route-Files.md).
