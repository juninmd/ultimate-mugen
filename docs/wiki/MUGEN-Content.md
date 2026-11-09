# Adding M.U.G.E.N content

## Characters

1. Put the folder in `mugen/chars/<name>/` with the `.def` inside, or drop the `.zip` straight into `mugen/`.
2. Open the game: the character shows up in the select screen. `Tab` filters between all, originals and M.U.G.E.N.

To download content from a GitHub repository instead, see the [GitHub marketplace](Marketplace.md).

Use Main menu › **Import M.U.G.E.N** (the only import button; on the web it opens the browser picker, on desktop the system file dialog) or drag the `.zip` onto the web window. Imports are idempotent: the same archive, or the same fighter or stage found again under another path, name or letter case, is registered only once (identity is a hash of the normalized `.def` plus its sprite file). Packs saved in the browser from older versions are deduplicated automatically on the next load.

A web import shows a full-screen loading overlay from the moment the files are selected until the last archive is registered. Archives are read, decompressed and registered **one per frame**, with at most two raw packs waiting in wasm memory; a batch of large packs no longer runs in one frozen frame. The same overlay appears on startup while the packs persisted in IndexedDB are restored, one at a time.

Hundreds of characters are registered without decoding everything; only the item under the cursor is loaded, and an LRU cache bounds memory.

Before a character reaches the roster, the catalogue checks that every file its `.def` names under `[Files]` exists inside the character's own folder (or the pack root, or `data/`): `sprite`, `anim` and `cns` are required. A character that fails the check is **quarantined**: it is kept out of the select screen, and the reason is listed. Files are never borrowed from another character's folder, so a copied `.def` that points to another fighter's sprites cannot load them by name. The check is structural (file presence), so it stays fast enough for the startup scan; a fighter whose files exist but whose animations reference missing sprites still loads and is shown with its missing frames.

Open **Personagens MUGEN** on the main menu to see the summary: how many characters loaded, how many are in quarantine and the most common reasons, plus a gallery of every loaded character (portrait, name and author; `←/→` change pages) and the quarantine list (`TAB` switches the view; `ESC` returns).

## Stages

Put `<name>.def` and the `.sff` in `mugen/stages/`, or inside a `.zip` in `mugen/`.

## What is supported

| Item | Status |
|---|---|
| `.sff` v1 and v2 (PCX, RLE8, RLE5, LZ5, PNG 8/24/32) | Supported |
| `.air`, `.cmd`, `.cns`, `.st`, `.snd` (PCM WAV 8/16 bit), `.act` | Supported |
| `localcoord` (HD 720p characters) | Supported |
| ~80 state controllers and 100+ triggers | See [CNS scripts](CNS-Scripts.md) |
| `ReversalDef`, `HitOverride`, `RemapPal`, `SndPan`, `VictoryQuote` | Planned |
| `[BGCtrl]` in stages, screenpacks, `fight.def` lifebars, `.fnt` fonts | Planned |
| Ikemen GO extensions (ZSS, Map, Dizzy, Tag...) | Planned |

## Validate a character

```bash
cargo run --release -p ume-engine --example mugen_check -- mugen/chars/kfm
cargo run --release -p ume-engine --example sprite_audit -- mugen/chars/kfm
cargo run --release -p ume-engine --example hover_check -- mugen/chars/kfm
```
cargo run --release -p ume-formats --example mugen_refs -- mugen/chars
cargo run --release -p ume-formats --example mugen_lint -- mugen/chars
cargo run --release -p ume-engine --example mugen_pairs -- --matrix mugen/chars
cargo run --release -p ume-formats --example mkselectdef -- mugen > mugen/select.def
```

| Tool | Shows |
|---|---|
| `mugen_check` | Loads and simulates a fight without a window; lists parse errors, invalid states and unsupported controllers, triggers and parameters (`--strict` fails on them); `--lint` adds source checks: duplicate parameters, invalid `trans`/`postype`/`space` values, `persistent` in negative states, `[State N]` not matching its `[Statedef]`, garbage lines, duplicate `[Command]` keys, duplicate and empty `.air` actions (`TRACE=1` lists each line) |
| `sprite_audit` | Missing or blank sprites referenced by the `.air` |
| `hover_check` | Fighter stuck off the floor (`TRACE=1` shows transitions) |

Characters whose license forbids redistribution must not be committed: `mugen/` is git-ignored.
