# Research: the browser M.U.G.E.N port

Reproducible evidence for [Ikemen-Web-Reverse-Engineering.md](../../wiki/Ikemen-Web-Reverse-Engineering.md).

Target: <https://w8.snokido.com/games/html5/dbz-vs-marvel-vs-capcom/> — 78 M.U.G.E.N fighters, 74 stages, running as a Go/WebAssembly build of Ikemen-GO with a WebGL2 backend.

## Scripts

| Script | Produces |
|---|---|
| `capture.js [outDir]` | every response (status, content-type, bytes), the frame tree, and the saved bodies: `runtime-engine.bin`, `runtime-ui.js`, `local-runtime.js`, `catalog.shvd.json`, the content packs |
| `wasm-artifacts.js <bin\|wasm>` | WASM imports/exports, the Go build info string, and the Go source files named by the symbol table |
| `frame-stats.js` | per-frame WebGL counters read from inside the engine iframe (`frame.html`), plus rAF pacing, canvas size, heap and long tasks |

Playwright + Chromium are required and are resolved through `NODE_PATH` (the same idiom the scripts in `scripts/` use), because the install usually lives outside the repository:

```console
$ NODE_PATH="$(npm root -g)" node frame-stats.js
```

## Numbers from two runs

| Metric | Run 1 | Run 2 |
|---|---|---|
| `drawArrays`/rAF p50 / p95 / max | 82 / 163 / 172 | 105 / 113 / 224 |
| uniforms/rAF | 322 | 397 |
| `useProgram`/s | 30–51 | 76 |
| `bindTexture`/s | 1377–2312 | 6213 |
| engine frames/rAF | 0.892 | 0.775 |
| worst long task during the match | 1276 ms (asset decode) | 1095 ms |
| heap | 72–74 MB | 9.5 MB |

The spread between runs is large because the rasterizer is software and the machine was busy while sampling: report ranges, not single values.

## Findings in one screen

| Question | Answer | How it was established |
|---|---|---|
| What runs the game? | Ikemen-GO compiled to `GOOS=js GOARCH=wasm`, WebGL2 backend | `wasm-artifacts.js`: 25 `gojs` imports, `run`/`resume`/`getsp` exports, Go symbol paths `Ikemen-GO/src/render_webgl.go` etc., build id `ikemen-GO v0.0.0-20260927200105-cae6cd63ba4a+dirty` |
| Are the sprites re-encoded? | No. Packs hold the original `.sff`/`.snd`/`.air`/`.cns` | unzipped `character-hdbz-gokuz2` pack, sha256 matches the catalog |
| Internal resolution | 640×480 into an offscreen FBO, then one blit to a 1280×720 canvas | `frame-stats.js`: `canvas.width/height = 640/480`, `clientWidth/Height = 1280/720` |
| Draw calls | 1 per sprite: p50 **82–105**/rAF, p95 113–163, max 172–224 | `frame-stats.js` per-rAF histogram (two runs) |
| GL traffic per sprite | 1 `bufferData` (64 B) + 1 `drawArrays(4)` + ~4 uniforms + ≤1 bind; text runs share one upload | `frame-stats.js` counters; `render_webgl.go:1133/1146`, `render.go:305` |
| How GL crosses to JS | one `CopyBytesToJS` + one `Invoke` per flush through a 256 KB command buffer (23 opcodes) | `render_webgl_cmdbuf.go`, opcode table also present as a string in the WASM |
| Update rate | 0.892 engine frames per rAF (fixed 60 Hz, rAF gated by `Video.Framerate`) | `frame-stats.js`, `__ikemenEngineFrames` counter |
| Palette swaps | 256×1 palette texture sampled from the sprite's red channel; 1 KB re-upload | `sprite.frag.glsl` (also embedded in the WASM), `render_webgl.go:458` |
| Texture atlases | fonts only (256×256 skyline + extrude); sprites keep one texture each | `render.go:1032-1224`, `font_webgl.go:440` |
| Cost of a first match | boot→menu 1.9 s, menu→select 4.0 s, select→fight 37 s (software renderer, ~50 MB download) | `frame-stats.js` timestamps |

## Caveats

* Headless Chromium with ANGLE/SwiftShader: frame times are a software-rasterizer floor, not GPU numbers.
* The 37 s to the first fight is dominated by downloading `core` (7 MB) plus two fighter packs (8–24 MB each) and a stage in that sandbox.
* Counts are per rAF callback, which includes the engine's gated frames; the engine frame delta is reported separately.
* `gl.drawArrays` inside a worker/OffscreenCanvas would not be visible to these hooks; verified it is the main frame's canvas (`frame.html`, `#ikemen-canvas`).
