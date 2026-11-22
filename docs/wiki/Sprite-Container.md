# Sprite Container (LZ4)

Every pixel block the engine stores for later use is LZ4-compressed, with one rule for both fighter kinds. Decoders are pure Rust (`lz4_flex`, safe mode, builds for `wasm32`), the format is lossless, and PNG stays the authoring format.

## Formats

| Magic | Holds | Written by | Read by |
|---|---|---|---|
| `UML1` | one sprite: indexed (8-bit + palette) or RGBA | `ume-tools pack-sprites` (web fighter packs) | `ume_formats::lz4_sprite`, `ume_native` sprite loader |
| `UMSF` | a whole `.sff`: palettes, sprite table, one `UML1` block per distinct pixel buffer | `ume-tools cook-sff` | `ume_formats::sff::decode_sff` and `decode_sff_only` |

Readers recognise the magic, so a PNG, a classic `.sff` and a cooked file are interchangeable under the same name. A cooked file can replace the original `.sff` in a character folder or zip.

## `UML1` layout (little endian)

```text
"UML1"  kind u8 (0 RGBA, 1 indexed)  colors u16  width u16  height u16
crop x u16  crop y u16  crop w u16  crop h u16
palette: colors * 4 bytes (RGBA), indexed only
LZ4 block of the cropped pixels (absent when the crop is empty)
```

The crop removes only the border that is exactly zero (index 0, or RGBA `0,0,0,0`), so decoding restores every byte. Colour hidden under zero alpha elsewhere is kept.

## `UMSF` layout

```text
"UMSF"  version u8 (1)  sff version u8  2 bytes zero
unique palettes   u32 count, each u32 length + bytes
palette table     u32 count, each group i32, index i32, palette ref u32
blocks            u32 count, each u32 length + UML1 bytes (no palette)
sprites           u32 count, each group, index, axis x, axis y i32; width, height u32;
                  palette_index i32; palette ref u32; block ref u32 (u32::MAX = none)
```

Sprites that shared one pixel buffer in the decoded `.sff` (links) share one block, and decode back to one shared `Arc`, so memory behaviour is unchanged. Blocks decode in parallel through `ume_formats::par::map` (single thread on `wasm32`). `decode_sff_only` skips blocks of sprites that are not wanted.

## Limits

A sprite is at most 8192 px per side and 16 Mpx, and a `UMSF` file may declare at most 512 Mpx of wanted sprites. Counts and lengths are checked against the file size; truncated or corrupt files return an error and never panic.

## Commands

```bash
cargo run --release -p ume-devtools --bin ume-tools -- cook-sff in.sff out.sff
cargo run --release -p ume-devtools --bin ume-tools -- pack-sprites web/dist
```

`cook-sff` decodes the original, cooks it, decodes the result and fails unless every palette and every sprite (pixels, axes, palette, links) is identical.

## Measurements

62 real M.U.G.E.N. characters from `mugen-assets` (54,382 sprites), one thread, per-sprite blocks: LZ4 decodes at about 1.5 GB/s of indexed pixels, 28 µs per sprite against 61 µs for the original RLE8/LZ5 decode. Totals, original `.sff` against `UMSF`: 853.6 MB against 624.1 MB, decode 2.30 s against 1.42 s with the default worker count. The comparison with zstd, PNG8, QOI and deflate is in [ADR 0009](adr/0009-lz4-sprite-container.md).
