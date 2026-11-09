# Sprite Toolkit

Local, dependency-light tools (Rust, `charforge` binary) in `crates/charforge/` (`spritekit` module) that turn generated art into MUGEN-ready assets. They never touch `assets/chars` or stages: output goes to a folder you choose.

```
cargo run --release -p ume-charforge -- slice sheet.png out/ --group 9000 --axis feet
cargo run --release -p ume-charforge -- slice sheet.png out/ --mode grid --grid 6 2
cargo run --release -p ume-charforge -- palettes base.act variants/ --only red blue dark --keep 1 2
cargo run --release -p ume-charforge -- air out/manifest.json --action 200 --ticks 4 --attack
cargo test --release -p ume-charforge
```

| Tool | What it does |
|---|---|
| `slice` | Detects the background (transparency or border color), cuts sprites by connected pieces or by grid, sets each axis at the feet (shared floor per row), builds one shared 256-color palette (index 0 transparent, flagged when quantized) and writes indexed PNGs plus `manifest.json` |
| `palettes` | Builds `.act` variants (hue, saturation, brightness presets) from a base `.act`; index 0 and `--keep` indices never change |
| `air` | Emits an AIR action from the manifest with `Clsn2` anatomical boxes (head, torso, legs) and optional `Clsn1` for extended limbs, relative to each sprite axis |

Library functions in `collision_boxes.py` also offer `interpolate_boxes` for in-between frames.

## What was not ported

The move generator is in [Move Generator](Move-Generator.md). CharFactory's GUI editors, AI assistant and MCP server, tick-by-tick simulator and SFF/SND writers overlap with the engine itself (`crates/formats`, `crates/engine`, `mugen_check`) or are large. Candidates for later work are listed in the Move Generator page.

## Acknowledgments

Approach inspired by [CharFactory](https://github.com/eduardovbe/CharFactory) by eduardovbe, MIT License, Copyright (c) 2026 eduardovbe. The code here is a fresh, smaller implementation; the MIT notice is reproduced in `REFERENCES.md`.
