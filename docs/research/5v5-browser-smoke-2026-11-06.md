# 5v5 Parallel Browser Smoke Test — 2026-11-06

## Scope

Smoke test of `TEAM · PARALLEL 5v5` with M.U.G.E.N. characters imported in the browser from a
`.zip`, on the web/PWA build (`wasm32-unknown-unknown`).

## Environment

- Linux/Windows CI runners and a local Windows build; headless Chromium (Playwright + SwiftShader).
- Build: `cargo build --release --target wasm32-unknown-unknown`, `wasm-bindgen --target web`, static
  server over `web/dist`.
- Pack: `cell.zip` (1 character) and `pack5s.zip` (`cell`, `Aizem`, `Sado`, `Matsumoto`, `Rukia`,
  3.4 MB, 5 characters), imported with `window.umeImport([File])` (the page's "Importar MUGEN"
  button uses the same call).
- Flow: import → title → `Team · Parallel 5v5` → MUGEN filter (`Tab` twice) → pick 5 → stage →
  fight. The CPU team is chosen by the game from the remaining roster.

## Findings and PRs

| Theme | Evidence | PR | State |
|---|---|---|---|
| Solid-dark `9000,1` portraits left black avatars in the roster and the 5v5 team strip | Rukia's `9000,1` is 142x92, 13064 visible pixels, average luminance 1.6; `9000,0` is 32x28, average 55.1. Before/after screenshots in `screenshots/portrait-placeholder-*.png`. | [#317](https://github.com/juninmd/ultimate-mugen/pull/317) | merged |
| Import status toast stayed pinned over the fight footer for the whole match | Visible at +1 s and still present 90 s into the match; gone at +9 s after the six-second TTL change. `screenshots/status-toast-*.png`. | [#320](https://github.com/juninmd/ultimate-mugen/pull/320) | merged |
| Unsupported Unicode glyphs rendered as `?` in the web shell | U+21EA in the import button, U+21BB in the portrait hint. Replaced with ASCII `+` / `&rarr;`. `screenshots/web-glyph-*.png`. | [#321](https://github.com/juninmd/ultimate-mugen/pull/321) | merged |

## 5v5 benchmark

`cargo run --release -p ume-engine --example tick_bench -- <char_dir> 20 5` (20 s, 5 fighters per
side, CPU vs CPU; the engine simulates helpers too):

| Source | Fighters on field | ticks/s | p50 | p99 | p99 / 16.6 ms |
|---|---|---|---|---|---|
| `GotenksZ2` | 61 | 524–596 | 1.4–2.0 ms | 2.3–3.7 ms | 13.6–22.4 % |
| `cell` | 12 | 11374–12137 | 0.078–0.081 ms | 0.14–0.19 ms | 0.8–1.2 % |

The 61-fighter case is the worst smoke sample (10 Gotenks plus helpers); it stays well inside the
60 Hz simulation budget on this machine. Rendering cost is additional and depends on the GPU.

## Final verification

- Final browser run on `master` after all three merges: 5v5 parallel, 5 imported characters on the
  player team, all team-strip portraits visible, no stale import toast, import button glyph fixed.
- `cargo check --release` and the CI matrix (`test`, `wasm`, `desktop`, `android`, `ios`, `lint`,
  `validate`) pass on the merged PRs.
- `cargo test --release -p ume-engine --lib portrait_tests` passes (4 tests).
