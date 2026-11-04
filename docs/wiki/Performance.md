# Performance

**English** · [Português (BR)](pt-BR/Desempenho.md)

How the engine keeps loading fast and frames smooth, and how to measure it.

## Loading

| Stage | What happens | Threads |
|---|---|---|
| SFF v2 decode | Headers are read in order; LZ5, RLE5, RLE8 and PNG payloads are decoded in parallel; linked sprites share the source pixels (`Arc`) | All cores (max 8) |
| Character sprites | Palette to RGBA conversion runs in parallel, once per unique pixel buffer and palette; linked sprites share one `ImageData`, so they also share one GPU upload | All cores (max 8) |
| Stage sprites | Same parallel RGBA conversion; the RGBA buffer is moved into the stage, never copied | All cores (max 8) |
| Select-screen portraits | `load_mugen_portrait` decodes only sprites 9000,0, 9000,1 and the first idle frame (plus their link sources) via `decode_sff_only` | 1 |
| Native (UMF) rig | Every unique frame is rasterized in parallel | All cores (max 8) |
| Whole character | Loaded on a `char-load` thread while the VS screen is shown | 1 background thread |

The helper is `ume_formats::par::map`. On the web (`wasm32`) it runs on the calling thread, since the browser build has no threads.

## Smooth frames

| Technique | Where | Effect |
|---|---|---|
| Frame pacing | `ume_shell::Pacer` | Frame times within 4% of a common refresh rate (30 to 240 Hz) become the exact period; the leftover is paid back once it adds up to a full period, so game time stays real time. With vsync this gives exactly one 60 Hz tick per frame instead of 0/2 jitter |
| Fixed step | `Session::update` | 60 Hz simulation with a tiny tolerance and at most 5 ticks per frame (no spiral of death) |
| Texture prefetch | `Gfx::prefetch`, `Texture2D::prefetch` | Fighter sprites are uploaded to the atlas when the fight starts (up to 50% atlas), so the first special move does not upload mid-match |
| Sound prewarm | `Audio::preload_match` | Every fighter sound is decoded in parallel during the VS screen; sounds of characters no longer in the match are freed |

## GPU memory

| Technique | Where | Effect |
|---|---|---|
| Palette in the shader | `Gfx::sprite_tex`, `IndexedTexture`, `Palette` | M.U.G.E.N sprites with indexed pixels go to the 8-bit index atlas and are colored by a palette row in the shader: 4x less upload and atlas space than RGBA. `RemapPal` swaps the palette row instead of baking a new RGBA image |
| Separate atlas resets | `ume_shell::quad` | The RGBA atlas (UI, text, true-color sprites) and the index atlas (+ palette rows) each reset on their own when full, so filling one never forces the other to re-upload |

## Measuring

| Command | Measures |
|---|---|
| `cargo run --release -p ume-engine --example mugen_load_bench [zip] [runs]` | M.U.G.E.N character load time (default: bundled `elecbyte.zip`) |
| `cargo run --release -p ume-engine --example load_bench` | Native (UMF) compile time |
| `cargo test -p ume-engine --release --test tick_alloc` | Zero allocations per simulation tick |

Reference (4 cores, release): `kfm720` went from 85 ms to 27 ms, `kfm` from 12 ms to 7 ms.

## Web (wasm)

| Technique | Where | Effect |
|---|---|---|
| `ControlFlow::Wait` on the web | `ume_shell::run` | winit's `Poll` spins a `postTask`/`AbortController` loop beside `requestAnimationFrame`; profiling showed about 75% of main-thread time (plus GC) burned there. Redraws are self-requested, so `Wait` is enough |
| Atlas zero-fill | `Renderer::new` | Atlas and palette textures are initialized once, so `texSubImage2D` no longer triggers lazy clears (Firefox warning) |
| Surface retry | `Shell::redraw` | A failed `get_current_texture` re-requests the frame instead of stalling the loop (black screen) |
| Loader + cache | `web/index.html`, `web/sw.js` | Download progress while the wasm loads; JS glue and wasm are fetched as `?v=<build>` (stamped by `build-web.sh`) and cached immutably, so a cache can never mix two builds, auto-reload on WebGL context loss |

Probe: `NODE_PATH=$(npm root -g) node scripts/web-profile.js Enter Enter Enter` (headless Chromium, needs `./scripts/build-web.sh` first).

## Browser stage loading

The browser WASM embeds 240×135 stage thumbnails, rather than the seven full PNGs (21,858,090 bytes). The selection grid decodes only requested thumbnails. Confirming a stage downloads `assets/stages/<id>.png` during the VS loading screen, with HTTP errors and a 30-second timeout returning to the menu. Only the most recently prepared full illustration is retained by the browser cache in Rust; a live match may still hold its own reference. Browser HTTP/service-worker caching can reuse previously downloaded files.

The native build keeps full images embedded, decoding each stage separately on first use. UMF definitions and the bundled Elecbyte archive still ship inside the WASM; this change only splits stage artwork. Fighter compilation and audio preparation remain on the main browser thread.

`scripts/build-web.sh` copies the external PNGs and `assets.js` into `web/dist/`. Deploy the entire directory, including `assets/stages/`. Offline play requires the selected stage to have been downloaded previously. When replacing stage art, regenerate its thumbnail under `assets/stages/thumbs/` as a 240×135 RGB PNG with `python scripts/generate-stage-thumbnails.py` (Pillow required).

Run `node scripts/web-smoke.js web/dist` after the web build (requires Playwright and Chromium). It rejects full stage downloads before confirmation and requires exactly one during the first match. This checks transfer scope, not a latency percentile or FPS improvement.
