# Format coverage matrix

> 🇧🇷 Versão em português: [Matriz de cobertura de formatos](pt-BR/Matriz-Formatos.md).

Status by file format, checked against the decoders in `crates/formats` (reading of the code, not yet fuzzed against a large pack collection).

- **Implemented**: decoded and used by the engine.
- **Partial**: decoded, with a known limit listed in the notes.
- **Missing**: not decoded; the file is skipped or reported as a warning.

## SFF (sprites)

| Feature | Status | Notes |
|---|---|---|
| SFF v1.01, PCX RLE sprites | Implemented | `sff.rs`, `pcx.rs` |
| SFF v1, PCX uncompressed rows | Implemented | `bytes_per_line` honored |
| SFF v1, 8-bit PCX (256-colour trailer palette) | Implemented | |
| SFF v1, 1/4-bit or 24-bit PCX | Missing | Header bit depth is ignored; such files decode as garbage. Rare in real packs |
| SFF v1, shared palette (`same pal` flag) | Implemented | Inherits the previous sprite's palette |
| SFF v1, linked sprites (`length = 0`) | Implemented | |
| SFF v2.00 / 2.01, raw 8-bit | Implemented | |
| SFF v2, raw 24-bit and 32-bit | Implemented | Converted to RGBA |
| SFF v2, RLE8 / RLE5 / LZ5 | Implemented | `sff/codecs.rs` |
| SFF v2, PNG 8 / 24 / 32 | Implemented | Broken PNG gives an empty, transparent sprite |
| SFF v2, palettes (RGB and RGBA, linked palettes) | Implemented | |
| SFF v2, palette-per-sprite selection | Implemented | |

## SND (sounds)

| Feature | Status | Notes |
|---|---|---|
| `ElecbyteSnd` container, group/index keys | Implemented | First duplicate key wins, as in M.U.G.E.N |
| Subfile chain with corrupt-link guard | Implemented | |
| WAV PCM 8 / 16-bit (the normal case) | Implemented | Decoded by `symphonia` in the audio layer |
| WAV ADPCM, 24-bit and other WAV codecs | Partial | Depends on what `symphonia` accepts; not verified per codec |

## AIR (animations)

| Feature | Status | Notes |
|---|---|---|
| `[Begin Action N]`, frames, `time` | Implemented | Duplicate actions: first wins |
| Frame flips `H` / `V` | Implemented | |
| Blending `A`, `A1`, `AS<n>D<n>`, `S` | Implemented | |
| `Clsn1` / `Clsn2` and `Clsn1Default` / `Clsn2Default` | Implemented | |
| `xscale`, `yscale`, `angle` per frame | Implemented | |
| `Loopstart` | Implemented | |
| `Interpolate` (blend, offset, angle, scale) | Missing | Line accepted and ignored (`air.rs`); no interpolation between frames |

## ACT (palettes)

| Feature | Status | Notes |
|---|---|---|
| 768-byte RGB palette | Implemented | Index 0 transparent, order reversed as in MUGEN |
| Short files (fewer than 256 colours) | Implemented | Missing entries stay black |

## DEF and CNS (characters and stages)

| Feature | Status | Notes |
|---|---|---|
| Character `.def` (`[Info]`, `[Files]`, `localcoord`) | Implemented | |
| `localcoord` 320x240 and 640x480 and 1280x720 | Implemented | Positions and distances are converted per player |
| Stage `.def` (`[Info]`, `[StageInfo]`, `[Camera]`, `[Player]`, `[Bound]`) | Implemented | |
| Stage `[BG]` layers: `normal`, `anim`, `parallax`, `trans`, `mask`, `maskwindow` | Implemented | |
| `[BGCtrlDef]` / `[BGCtrl]`: `null`, `visible`, `enable`, `velset`, `veladd`, `posset`, `posadd`, `anim`, `sinx`, `siny` | Implemented | |
| Stage `zoffset`, `autoturn` | Implemented | Used by the engine (`stagevar.rs`, `matchs.rs`) |
| Stage `resetbg` | Missing | Not read anywhere in the workspace |

## Screenpacks

| Feature | Status | Notes |
|---|---|---|
| `system.def` `[Info] localcoord` (4:3 and 16:9 packs) | Implemented | Converted to the HUD space |
| `system.def` `[Files]`, `[Music]`, `[Title Info]`, `[Select Info]` | Implemented | |
| `system.def` `[<name>BGdef]` / `[<name> N]` backgrounds | Implemented | |
| `select.def` `[Characters]`, `[ExtraStages]`, `[Options]` | Implemented | |
| `fight.def` `[Files]`, `pN.*` bars, element prefixes | Implemented | |
| Storyboards (`[SceneDef]`, `[Scene N]`, fades, layers, `bgm`) | Implemented | |
| `.fnt` v1 (embedded PCX, `[Def]`, `[Map]`) | Implemented | |
| `.fnt` v2 (bitmap and truetype) | Partial | Parsed in `fnt.rs`; truetype rendering not verified |
| `[Versus Screen]`, `[Victory Screen]`, `[Round Info]` | Partial | Read through the generic element prefixes; not every key has a dedicated renderer; not verified on packs |
| WinMUGEN and MUGEN 1.0 / 1.1 quirks | Partial | Unknown keys are ignored, not rejected; no per-version switch |

## Known gaps (next PRs)

Small PRs, in order:

1. AIR `Interpolate` (keyword currently ignored).
2. Stage `resetbg`.
3. SFF v1 low bit depth (1 and 4-bit PCX). Needs a header palette path in `sff/`; `sff.rs` is on the file-size baseline, so move the v1 code into its own module first.
4. Verify WAV codecs beyond PCM 8/16-bit with real packs.
5. Screenpack keys without a dedicated renderer (victory and versus screens).

This list is what the audit found; it is not yet proven complete against a large pack collection.
