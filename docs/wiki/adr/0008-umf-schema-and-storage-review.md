# ADR 0008: UMF schema in CI, and JSON + PNG storage reviewed

- Status: Accepted for the schema and for keeping JSON; Proposed for the cooked texture pack (superseded by [ADR 0009](0009-lz4-sprite-container.md): LZ4 instead of QOI)
- Date: 2026-11-22

## Context

The UMF 2 format of [ADR 0007](0007-umf2-definitive-fighter-format.md) was documented only in prose. The loader ignores unknown keys (serde), so a typo such as `hitz` or `lyfe` passed every test and did nothing. The question was also open whether JSON + PNG is the best storage for performance and maintenance.

## Decision

1. `docs/schema/umf-2.schema.json` (JSON Schema 2020-12) describes fighters and `umf-kit/1` kits and rejects unknown keys. `scripts/umf-schema.mjs` validates with a small in-repo validator (no dependency) and `scripts/umf-schema.test.mjs` runs it on every shipped fighter, kit and example in the `release-scripts` CI job. Packs outside the repo run `node scripts/umf-schema.mjs <file>`.
2. Keep JSON for fighter definitions. Keep PNG as the authoring and interchange texture format.
3. Proposed, not implemented: a build-time cooked pack that trims transparent borders and stores the frames as QOI, decoded by a small in-repo decoder. See the numbers below.

## Measurements

Rust release builds, 21 shipped fighters and the 137 sprites (400x400 RGBA) of `kaito`. Native = this container; wasm = the same code built for `wasm32-wasip1` and run in Node (V8), which is not a browser. Medians of 5 to 15 runs; the container is noisy (about 20% between runs), and every conclusion held in all three full runs. The atlas could not be run under Node WASI (it crashed on the 56 MB image).

Definitions, all 21 fighters:

| Format | Bytes | gzip | Parse native / wasm (ms) |
|---|---|---|---|
| JSON as shipped | 1,158,382 | 191,011 | 12.9 / 12.5 |
| JSON minified | 907,786 | 167,811 | 9.9 / 10.7 |
| MessagePack | 656,470 | 175,755 | 8.3 / 11.1 |
| CBOR | 664,551 | 173,171 | 14.9 / 18.9 |

Typed `NativeChar` parse, all fighters: JSON 10.7 ms, MessagePack 12.4 ms, CBOR 23.2 ms. bincode and postcard write a fighter but cannot read it back (`Value`, untagged enums and `deserialize_with` need a self-describing format).

Textures, 137 sprites:

| Format | MB | gzip MB | Decode native / wasm (ms) |
|---|---|---|---|
| PNG, own decoder | 7.89 | 5.58 | 102 / 134 |
| PNG, `png` crate | 7.89 | 5.58 | 70 / 122 |
| WebP lossless | 5.44 | 4.94 | 91 / 191 |
| QOI | 4.84 | 3.00 | 28 / 32 |
| LZ4 raw RGBA | 7.27 | 5.16 | 35 / 38 |
| PNG, trimmed | 4.53 | 4.52 | 44 / 89 |
| QOI, trimmed | 4.63 | 2.98 | 21 / 27 |
| One atlas, PNG / QOI | 5.35 / 5.41 | 5.25 / 3.39 | 96 / 56 (native) |

Trimming transparent borders keeps 48% of the pixels: decoded RAM per fighter goes from 87.7 MB to 42.1 MB.

## Consequences

- Definitions parse in about 0.6 ms per fighter, so format is not a cost. MessagePack saves 8% after gzip and is not faster typed; it would cost diffs, hand editing and the schema. JSON stays.
- The real cost is texture decode and memory. Cooked trimmed QOI would cut decode about 80% on both targets, the gzip download by 47% and RAM by 52%. PNG stays the source of truth, so artists keep their tools and the sprite art itself is never rewritten. A single atlas is worse than separate files: one big image decodes slower and blocks lazy loading.
- The cooked pack needs a change to the sprite pipeline (`pack_build.rs`, `decode_cache.rs`) and the wasm build, so it waits for approval and its own pull request with a browser smoke test.
- Swapping the own PNG decoder for the `png` crate would gain about 30% native and 9% wasm but adds a dependency; not taken.
- Limits: one fighter's sprites, Node instead of a browser, no GPU upload time.
