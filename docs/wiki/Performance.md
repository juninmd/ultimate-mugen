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
| VFS resolution | Short-circuiting single-candidate lookups, zero-alloc ASCII path normalization, and clone-free `BTreeMap::range` queries | Calling thread |

The helper is `ume_formats::par::map`. On the web (`wasm32`) it runs on the calling thread, since the browser build has no threads.

## Smooth frames

| Technique | Where | Effect |
|---|---|---|
| Frame pacing | `ume_shell::Pacer` | Frame times within 4% of a common refresh rate (30 to 240 Hz) become the exact period; the leftover is paid back once it adds up to a full period, so game time stays real time. With vsync this gives exactly one 60 Hz tick per frame instead of 0/2 jitter |
| Fixed step | `Session::update` | 60 Hz simulation with a tiny tolerance and at most 5 ticks per frame (no spiral of death) |
| Texture prefetch | `Gfx::prefetch_step`, `App::warm_step` | Fighter sprites are uploaded to the atlas one 512 KiB chunk per frame across the loading steps (VS screen) or between yielded frames (`start_fight`), so no single frame pays for a whole fighter; a rematch with the same roster copies nothing |
| Sound prewarm | `Audio::preload_match` | Every fighter sound is decoded in parallel during the VS screen; sounds of characters no longer in the match are freed |
| Bytecode VM | `ume_engine::bytecode` | CNS/ZSS expressions compile into a flat stack VM (`[f64; 32]`) with inline redirects (`parent`, `root`, `helper`, `enemynear`), math/comparison intrinsics, zero/one-arg triggers, `CmpStr`, `Special` (`animelem`, `timemod`), and `:=` assignments |
| Zero-alloc hot paths | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | `Arc<str>` command keys, zero-allocation ASCII `HitDefAttr` matching, branchless `abs` direction translation, O(1) close-range AI selection, in-place `append_seq`, zero-alloc stage background tile iteration, and in-place `RefCell::get_mut` variable assignments avoid heap allocations per tick |

## Resilience bounds | `ume_formats`, `ume_engine::matchs` | Image decoders reject headers above `MAX_IMAGE_PIXELS` (`1 << 26`) before allocating; matches cap helpers (60), explods (512), projectiles (256), and texts (128) |

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
| `cargo run --release -p ume-engine --example tick_bench -- [zip_or_dir] [seconds] [per_side]` | Simulation throughput (`ticks/s`) and latency percentiles (`p50`, `p99`, `max`, 16.6 ms frame budget %) |
| `cargo run --release -p ume-engine --example sprite_mem -- <char_dir>...` | Sprite payload per fighter: sprites, index/RGBA atlas MB, largest sprite (sizes the warm-up budget) |
| `cargo test -p ume-engine --release --test tick_alloc` | Zero allocations per simulation tick |
| `cargo test -p ume-formats --release --test fuzz_resilience` | Mutation fuzzing across SFF v1/v2, SND, PCX, PNG, FNT, RLE8/RLE5/LZ5, AIR and INI decoders |
| `cargo test -p ume-engine --release --test stress_resilience` | Hostile CNS expressions, recursive helper/explod/projectile floods, and 4v4 simul stress. |

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

## Main-thread loading budget

Rule: the render thread never waits on disk, decoding, parsing or synthesis. On native, that work runs on worker threads and the UI polls for completion; on wasm (single thread) it stays sliced into one step per frame.

| Work | Where it runs now |
|---|---|
| Select-screen MUGEN preview and portrait | `CharEntry::stream_preview` / `stream_portrait` (max 2 workers, collected by `Registry::reap_streams`). Call cost ~0.1 ms versus 28 ms (KFM) and 84 ms (KFM HD) synchronously |
| `.zip` import (hash + decompression) | `registry::read_zip` on a worker; only `finish_import` (~4 ms) runs on the main thread |
| Fight sample decoding | `Audio::preload_match` on a worker, the VS loading bar waits for it |
| Stage music synthesis (17-31 ms per track) | `Audio::music_ready` on a worker |
| Winner screen | uses the cached definition (`peek`), never loads |

Frame-time watchdog: set `UME_WATCHDOG=1` to log every frame longer than `UME_HITCH_MS` (default 50) with the active label (`ume_shell::watchdog::label("load:stage")`), and a summary with the worst frame on exit. Labels in use: `select:preview`, `load:character`, `load:stage`, `load:session`, `load:audio`, `load:sprites`, `import`. The label sticks to the first phase named in a frame, so the trailing bookkeeping blocks do not relabel a slow loading step.
## Virtual filesystem lookup

`cargo run --locked --release -p ume-formats --example vfs_bench` measures path normalization and basename resolution with 50,000 unrelated files. It prints five samples per scenario. This synthetic benchmark excludes disk I/O, decoding and rendering.

The folder fallback uses the ordered VFS map's range for the character directory, avoiding scans over unrelated characters. Directory boundaries matter: `chars/kfm-extra` must not override a file under `chars/kfm`. Global basename lookup remains the last compatibility fallback. ASCII path normalization creates one output string; non-ASCII paths retain Unicode lowercase behavior, including contextual Greek sigma.

On the cloud development machine, median normalization time decreased from 60.8 to 48.0 ns per operation, and median folder fallback time from 278.2 to 1.0 µs. These are microbenchmark results, not FPS or whole-game loading measurements. Missing files that require the global fallback can still scan the entire map.

## Audio and UI allocation pressure

`cargo run --locked --release --example audio_bench` decodes a generated 30-second, 44.1 kHz stereo WAV five times, counting allocations and reallocations during each decode. The decoder reuses its interleaved sample-conversion buffer while its capacity is sufficient; it grows when a later packet needs more samples. The written sample count still comes from the current packet, including partial final packets and mono input.

In the cloud development environment, warm-run allocations decreased from 2,331 to 1,183 per decode. Elapsed times were noisy (roughly 9–11 ms in warm runs) and do not demonstrate a speedup. This reduces temporary allocation churn rather than the final decoded PCM size. The benchmark excludes playback and audio-device latency. Run `cargo test --locked --release --example audio_bench` for sample-preservation, mono/stereo, tail and existing PCM tests.

`Ui::quad` transforms its four vertices into a fixed array, removing a temporary heap allocation per drawn quadrilateral without changing triangle order or coordinates. This does not establish an FPS improvement.

- **Wave 29 - Chunked Raster Pixmap Conversion & In-Order SFF RLE8 Decoding**:
  - Replaces per-pixel `extend_from_slice` calls in `raster::pixmap_to_image` with direct 4-byte chunk slice writes (`chunks_exact_mut(4)`).
  - Streamlines `decode_rle8` read sequence without extra index decrementing.

- **Wave 30 - Branchless Input Relative Directions & Byte Jump Table**:
  - Replaces nested branching in `input::rel_dirs` with branchless bitwise shift `ud | (4 << (is_back ^ 1))`.
  - Replaces string matching in `button_bit` with byte-level slice matching, supporting case-insensitivity without allocation.
  - Implements branchless facing assignment in `InputHistory::push`.

## HitDef zero-alloc shake parameters and in-place type matching

- Zero-alloc shake parameter resolution: `hitdef::build` uses direct static parameter string lookups (`parse_shake`) instead of formatting 10 dynamic heap `String` keys (`{prefix}.freq`, `{prefix}.time`, `{prefix}.ampl`, etc.) per `HitDef` instantiation.
- In-place `animtype` and `groundtype` parsing: `hitdef::animtype` and `hitdef::groundtype` match standard MUGEN animation/ground tokens using `eq_ignore_ascii_case` directly on borrowed `&str` without lowercase string allocations.
