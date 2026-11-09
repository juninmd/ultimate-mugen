---
name: character-forge
description: Generate MUGEN character assets locally from generated art: slice sprite sheets, build palette variants, auto collision boxes and TOML-driven move generation (AIR, CNS, CMD, Statedef -1). Use when asked to create or extend a character, its sprites, animations or moves.
---

# Character forge

Offline Rust pipeline in `crates/charforge` (`charforge` binary: `slice`, `palettes`, `air`, `moves`, `check`). Never write into `assets/chars` sprites or stages unless the user asks; send output to a scratch or user-chosen folder.

## Steps

1. Slice art: `cargo run --release -p ume-charforge -- slice sheet.png out/sprites --group 9000 --axis feet` (grid sheets: `--mode grid --grid COLS ROWS`). Check `manifest.json` (axis, `exact_palette`; false means quantized).
2. Palettes: `cargo run --release -p ume-charforge -- palettes base.act out/pal --only red blue dark --keep 1 2`.
3. Describe moves in a TOML spec (see `crates/charforge/examples/moves.toml`): `kind`, `state`, `command`, `frames = [[group, item, ticks, "hit"?]]`, `hit`, `sounds`, `vfx`, `cancel`, `phases`, `superpause`.
4. Generate: `cargo run --release -p ume-charforge -- moves moves.toml out/moves` writes `moves.air`, `moves.cns`, `moves.cmd`, `statedef-1.cns`. Re-running overwrites; edit the TOML, not the output.
5. Merge the files into the character, then run `cargo run --release -p ume-charforge -- check <char_dir> --strict` (engine loader diagnostics, same as `mugen_check`) and play it in Training with hitboxes on (F2).
6. Run `cargo test --release -p ume-charforge` after changing the tools.

## Rules

- Moves are data: change numbers in the TOML and regenerate.
- Command inputs use MUGEN notation (`~D, DF, F, x`); commands reuse the name `spec_<state>`.
- Code has no comments; docs live in `docs/wiki/Sprite-Toolkit.md` and `docs/wiki/Move-Generator.md` with pt-BR copies.
- Credit: approach inspired by CharFactory (MIT, eduardovbe); see `REFERENCES.md`.
