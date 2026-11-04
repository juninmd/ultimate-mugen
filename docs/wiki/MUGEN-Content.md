# Adding M.U.G.E.N content

## Characters

1. Put the folder in `mugen/chars/<name>/` with the `.def` inside, or drop the `.zip` straight into `mugen/`.
2. Open the game: the character shows up in the select screen. `Tab` filters between all, originals and M.U.G.E.N.

To download content from a GitHub repository instead, see the [GitHub marketplace](Marketplace.md).

On the web, use the **Import MUGEN (.zip)** button or drag the `.zip` onto the window. Imports are idempotent: the same archive, or the same fighter or stage found again under another path, name or letter case, is registered only once (identity is a hash of the normalized `.def` plus its sprite file). Packs saved in the browser from older versions are deduplicated automatically on the next load.

Hundreds of characters are registered without decoding everything; only the item under the cursor is loaded, and an LRU cache bounds memory.

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

| Tool | Shows |
|---|---|
| `mugen_check` | Loads and simulates a fight without a window; lists parse errors, invalid states and unsupported controllers, triggers and parameters (`--strict` fails on them) |
| `sprite_audit` | Missing or blank sprites referenced by the `.air` |
| `hover_check` | Fighter stuck off the floor (`TRACE=1` shows transitions) |

Characters whose license forbids redistribution must not be committed: `mugen/` is git-ignored.
