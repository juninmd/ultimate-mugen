# ADR 0009: LZ4 sprite container for fighters and M.U.G.E.N. characters

- Status: Accepted
- Date: 2026-11-22
- Supersedes: the "cooked texture pack" proposal of [ADR 0008](0008-umf-schema-and-storage-review.md) (QOI for every sprite)

## Context

Decode time and resident size of sprites dominate loading, most of all on `wasm32`, which has one thread. M.U.G.E.N. characters are 8-bit indexed (palette applied on the GPU); native fighters are RGBA and the web packs already quantise them to a palette.

## Measurements

Release build, 62 real characters (54,381 indexed sprites, 2.33 GB decoded), each sprite compressed and decoded on its own, one thread:

| Codec | Pack MB | µs per sprite |
|---|---|---|
| Original `.sff` (RLE8/LZ5) | 854 | 61 |
| LZ4 | 618 | 28 |
| zstd level 3 (C) | 412 | 48 |
| zstd level 19 (C) | 341 | 55 |
| deflate 6 | 378 | 73 |
| PNG8, `png` crate / own decoder | 502 | 92 / 130 |
| QOI RGBA | 968 | 84 |
| ruzstd (pure Rust) | 412 | 140 |

With 1, 2, 4 and 8 threads on 4 cores the order does not change; LZ4 scales 3.6x from 1 to 4 threads. In `wasm32` (Node), LZ4 decodes 672 sprites in 66 ms, the fastest codec that builds without a C toolchain. QOI RGBA stores four bytes per pixel and loses the GPU palette. On 2,610 native RGBA sprites, QOI is 42% smaller than LZ4 after gzip and decodes about 10% slower.

## Decision

1. LZ4 (`lz4_flex` 0.14, safe mode) is the one compression for stored sprite pixels, for M.U.G.E.N. and native fighters alike.
2. Two containers share it: `UML1` per sprite and `UMSF` per `.sff` (see the wiki page [Sprite Container](../Sprite-Container.md)). Readers detect the magic, so PNG and classic `.sff` still load.
3. Web fighter packs now carry `UML1` indexed sprites instead of PNG8.
4. PNG stays the authoring and interchange format; native art is never rewritten.

## Consequences

- Chars cook to about 27% less data and decode in about 62% of the time; loading stays bit-exact (`cook-sff` verifies every sprite).
- zstd would save a third more bytes at 1.7x the decode time and needs a C toolchain for `wasm32`; it is not adopted.
- QOI would shrink native RGBA sprites further; a later per-sprite codec byte can add it without breaking `UML1` readers.
- New dependency: `lz4_flex` (MIT, pure Rust, no `unsafe` in the enabled features).
- Not in this change: lazy decode on first draw, and cooking desktop loose PNGs or imported zips at install time.
