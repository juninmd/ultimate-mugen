# Authoring tool survey: Fighter Factory Studio

Static study of a public Windows build of Fighter Factory Studio (FFS 3, portable Qt4/Qt5 package, about 2000 files, 57 MB). It is the reference editor of the MUGEN community, so it shows what authors expect when they create characters. Nothing from the package is copied into this repository; only formats, checks and workflow conclusions are recorded here.

## Method and limits

- The archive was extracted into an empty folder and only inspected: file listing, XML/HTML/Lua data files and `strings` on the engine-module DLL. No binary was executed.
- REA (`rea-agents` 6.8.0) installs and runs, but native decompilation needs Hopper, Ghidra, IDA or Rizin, none of which exist in this environment. Native analysis was therefore limited to string tables. A later pass with a provider could decompile `ff.mugen.dll` to confirm the exact rule behind each message.

## What the package contains

| Area | Finding |
| --- | --- |
| Editor modules | Sprites (SFF 1/2, onion skin, layers), Animations (visual and code), Palettes, Sounds, Commands, States, Definitions, Emulator and Debug workspaces |
| Scripting | Lua 5.3 scripts for editor automation (`scripts/tool`, `scripts/project`) |
| Engine module | One module per engine (`engines/mugen`): schema, completion list, one template file per state controller (89), snippets, project templates, per-version docs (1.0, 1.1, common, DOS, Win) |
| Emulator | An in-editor MUGEN-compatible runtime (MUGENext) with training mode, breakpoints by file and line, variable inspection |

## Declarative schema (`syntax.xml`)

The whole `.def/.cns/.cmd/.air` grammar is data, not code. It covers nine file kinds (char, state, command, stage, fight, select, motif, font, storyboard) and almost 3000 parameters. Each parameter has a name and a type (`int`, `float`, `int2`, `int4`, `intvec`, `expi`, `expf`, `expb`, `keyword`, `ustring`, `file`, `hitattr`, `hitflags`, `guardflags`, `keyseq`), plus optional attributes: `required`, `optional`, `minver`, `deprecated`, `list`, `max`. Examples of required fields: `StateDef` needs `type`, `movetype`, `physics`, `anim`; `[Data]` needs `life`, `attack`, `defence`, `fall.defence_up`, `liedown.time`, `airjuggle`.

Lint messages driven by that schema: duplicate parameter in a group, invalid block for the project type, parameter invalid for the project version, parameter deprecated since version X, wrong argument count, duplicate or invalid hit attribute and hit flag, invalid keyword, trigger redirection invalid in this context, syntax error with line.

## Asset checks

- Required sprite missing (group, index) when loading a character.
- Palette or sprite entry without id, file, axis or palette; invalid NPOT mode.
- Duplicate palettes (`noDuplicatePal`, "remove all duplicates") and unused palettes ("clear unused").
- 32-colour palettes stored as 5-bit data; palette remap table.

## Runtime warnings of the emulator

Each warning carries tick, player, state and `file: line`: change to an invalid state, action or anim element; guard state with the wrong move type; invalid `RemapPal` in Helper or Explod; Explod bind id not found; invalid PalFX time, colour or multiplier; trigger redirection problems; invalid AI file ignored.

## Workflow features worth matching

1. Sprite import by file name with group and index (`group_index`, `group-index`, `group,index`), axis presets, three palette modes (own palette, force palette, exchange palette), crop before or after the axis.
2. Aligned export with one common axis ("perfect axis") plus onion skin while aligning.
3. Sprite explorer: create an animation straight from selected thumbnails.
4. Per-frame loop point, scale, angle and interpolation flags.
5. Variable usage list and log per state file (`var`, `fvar`, `sysvar` collisions between states and helpers).
6. Snippets and state controller templates for common moves.

## Gap against our skills and tools

The `mugen-character-creator` skill had no pre-flight checklist for required or version-gated parameters, palette hygiene, variable collisions, or import naming, and no mapping of runtime warnings to causes. The new reference `references/checklist_lint_editor.md` adds it, and the skill now points to it.

## Follow-ups (code, not done here)

- Drive `mugen_check` from a schema of required, version-gated and deprecated parameters written from the public MUGEN documentation.
- Add a variable usage report (`var`, `fvar`, `sysvar`) across states and helpers.
- Add palette duplicate and unused reports next to `sprite_audit`.
- Accept `group_index` style names in `import_sprite_atlas`.
- Print the runtime warnings above, with state and line, in the debug overlay.
