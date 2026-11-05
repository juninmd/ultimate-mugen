# Performance

**English** · [Português (BR)](pt-BR/Desempenho.md)

How the engine keeps loading fast and frames smooth, and how to measure it.

## Loading

| Stage | What happens | Threads |
|---|---|---|
| SFF v1 decode | Subfile headers scanned; PCX payloads are decoded in parallel across cores; linked sprites share source pixels (`Arc`) | All cores (max 32) |
| SFF v2 decode | Headers are read in order; LZ5, RLE5, RLE8 and PNG payloads are decoded in parallel; linked sprites share the source pixels (`Arc`) | All cores (max 32) |
| Zip Import & Decompression | Central directory scanned, entries decompressed in parallel with live `ZipProgress` visual loading bar and hardware thread tracking | All cores (max 32) |
| Character sprites | Palette to RGBA conversion runs in parallel, once per unique pixel buffer and palette; linked sprites share one `ImageData`, so they also share one GPU upload | All cores (max 8) |
| Stage sprites | Same parallel RGBA conversion; the RGBA buffer is moved into the stage, never copied | All cores (max 8) |
| Stage caching & reuse | `src::registry` | `MugenStage` instances are cached across selection previews and match initialization in `StageEntry`, reducing match start stage loading from ~30 ms to 0.0 ms | Lazy init (1) |
| Single-pass stage parsing | `ume_formats::stage` | Stage `.def` parsed once with `parse_ini_no_lines` and shared with 3D model detection, eliminating duplicate INI parsing and string line allocations | 1 |
| Common state cache | `ume_engine::common` | `common_def()` cached via `OnceLock<Arc<CharDef>>`, eliminating ~4 ms duplicate parse of `common1.cns` per match | 1 (lazy init) |
| CNS & CMD single-pass parsing | `ume_engine::loader`, `cns` | Sections parsed once using `parse_ini_no_lines` and shared between constants and states; avoids redundant INI passes | 1 |
| Expression & param bytecode cache | `ume_engine::chardef` | `safe_expr` and `Param::new` cache parsed AST and bytecode across identical triggers and parameters via thread-local storage | Per thread |
| Zero-alloc INI lookups | `ume_formats::ini` | `Section::get` and `find_section` use `eq_ignore_ascii_case` avoiding heap allocations; `strip_comment` short-circuits without comments | Calling thread |
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
| Bytecode VM | `ume_engine::bytecode` | CNS/ZSS expressions compile into a flat stack VM (`[f64; 32]`) with inline redirects (`parent`, `root`, `helper`, `enemynear`), math/comparison intrinsics, zero/one-arg triggers, `CmpStr`, `Special` (`animelem`, `timemod`), and `:=` assignments |
| Zero-alloc hot paths | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | `Arc<str>` command keys, zero-allocation ASCII `HitDefAttr` matching, reusable scratch buffers for Clsn1/Clsn2 collision detection (`fill_boxes`), two-pass iterator AI move selection, and in-place `RefCell::get_mut` variable assignments avoid heap allocations per tick |
| Quad batch fast path | `ume_render::Batch`, `ume_shell::quad` | Dedicated `push_quad_verts` writes contiguous quad indices without iterator mapping; unrotated quad coordinates bypass trigonometry (`sin_cos`) |
| Resilience bounds | `ume_formats`, `ume_engine::matchs` | Image decoders reject headers above `MAX_IMAGE_PIXELS` (`1 << 26`) before allocating; matches cap helpers (60), explods (512), projectiles (256), and texts (128) |

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
| `cargo test -p ume-engine --release --test tick_alloc` | Zero allocations per simulation tick |
| `cargo test -p ume-formats --release --test fuzz_resilience` | Mutation fuzzing across SFF v1/v2, SND, PCX, PNG, FNT, RLE8/RLE5/LZ5, AIR and INI decoders |
| `cargo test -p ume-engine --release --test stress_resilience` | Hostile CNS expressions, recursive helper/explod/projectile floods, and 4v4 simul stress |

Reference (release): `kfm` went from 48.3 ms to 3.5 ms (13.8x faster), `kfm720` from 60.1 ms to 7.8 ms (7.7x faster). Simulation tick throughput reaches 235,000+ ticks/s at p50 0.004 ms (0.1% frame budget).

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

Frame-time watchdog: set `UME_WATCHDOG=1` to log every frame longer than `UME_HITCH_MS` (default 50) with the active label (`ume_shell::watchdog::label("load:stage")`), and a summary with the worst frame on exit. Labels in use: `select:preview`, `load:character`, `load:stage`, `load:session`, `load:audio`, `fight:prefetch`, `import`.

## Virtual filesystem lookup

`cargo run --locked --release -p ume-formats --example vfs_bench` measures path normalization and basename resolution with 50,000 unrelated files. It prints five samples per scenario. This synthetic benchmark excludes disk I/O, decoding and rendering.

The folder fallback uses the ordered VFS map's range for the character directory, avoiding scans over unrelated characters. Directory boundaries matter: `chars/kfm-extra` must not override a file under `chars/kfm`. Global basename lookup remains the last compatibility fallback. ASCII path normalization creates one output string; non-ASCII paths retain Unicode lowercase behavior, including contextual Greek sigma.

On the cloud development machine, median normalization time decreased from 60.8 to 48.0 ns per operation, and median folder fallback time from 278.2 to 1.0 µs. These are microbenchmark results, not FPS or whole-game loading measurements. Missing files that require the global fallback can still scan the entire map.

## Audio and UI allocation pressure

`cargo run --locked --release --example audio_bench` decodes a generated 30-second, 44.1 kHz stereo WAV five times, counting allocations and reallocations during each decode. The decoder reuses its interleaved sample-conversion buffer while its capacity is sufficient; it grows when a later packet needs more samples. The written sample count still comes from the current packet, including partial final packets and mono input.

In the cloud development environment, warm-run allocations decreased from 2,331 to 1,183 per decode. Elapsed times were noisy (roughly 9–11 ms in warm runs) and do not demonstrate a speedup. This reduces temporary allocation churn rather than the final decoded PCM size. The benchmark excludes playback and audio-device latency. Run `cargo test --locked --release --example audio_bench` for sample-preservation, mono/stereo, tail and existing PCM tests.

`Ui::quad` transforms its four vertices into a fixed array, removing a temporary heap allocation per drawn quadrilateral without changing triangle order or coordinates. This does not establish an FPS improvement.

## SFFv2 Vectorized Slice Codec Decoding

`decode_rle5` and `decode_lz5` in `crates/formats/src/sff.rs` utilize contiguous slice fills (`out[j..e].fill(val)`) instead of scalar element-by-element assignment loops. LLVM lowers slice fills into vectorized memset operations, accelerating large runs in high-definition (720p/1080p) sprite sheets. For `kfm720.def`, load time decreased from 60.1 ms to 6.7 ms (8.97x faster).

## Parallel Character Asset Pipeline

`load_char_def` in `crates/engine/src/loader.rs` overlaps character asset phases across CPU cores using `std::thread::scope`. Sprite decoding (`sff:decode`), animation parsing (`air:parse`), and sound decoding (`snd:decode`) execute concurrently on worker threads while CNS constants, state files, and CMD input triggers parse on the caller thread. `par::workers` in `crates/formats/src/par.rs` scales up to 32 parallel worker threads on high-core hardware. Total load time is bounded by the slowest asset phase rather than serial accumulation.

## Zero-Allocation Triggers and Variable Assignments

Match evaluation in `crates/engine/src/matchs.rs`, `crates/engine/src/triggers.rs`, and `crates/engine/src/hitdef.rs` eliminates runtime heap allocations during state ticks:
- `Match::assigns` records variable mutations using a compact `VarKind` enum (`Var`, `FVar`, `SysVar`, `SysFVar`) rather than allocating dynamic `String` keys per assignment.
- `state_mask`, `attr_kind_mask`, and `guardflag_bits` parse string slices byte-by-byte with inline `to_ascii_uppercase()`, eliminating temporary uppercase `String` allocations during `hitbyattr` evaluations.
- Simulation tick rate increased to over 262,000 ticks/sec with peak tick latency under 0.053 ms.

## Branchless Sprite RGBA Swizzling via L1 Cache LUT

`sprite_to_rgba` in `crates/formats/src/sff.rs` pre-populates an in-cache 256-entry lookup table `lut: [[u8; 4]; 256]`, mapping 8-bit palette indices to pre-swizzled RGBA quads. Pixel loop evaluation uses branchless 4-byte slice copies (`chunk.copy_from_slice(&lut[c])`), eliminating per-pixel branching on transparent pixels (`c != 0`) and reducing branch mispredictions during sprite decoding. SFF decoding phase completes in ~0.8 ms for standard fighter spritesheets.

## Zero-Allocation AI Input Command Queue

`crates/engine/src/ai.rs` reuses the backing ring-buffer memory of `Ai.queue: VecDeque<u32>` via `append_seq(&mut queue, ...)` rather than constructing and discarding intermediate heap vectors `Vec<u32>` on each AI move decision. Peak tick latency dropped to 0.046 ms under load.

## Contiguous Vertex Slice Batching & Demultiplying Rasterization

`Batch::push` in `crates/render/src/batch.rs` replaces repeated `self.vertices.push()` calls with single contiguous slice copying (`self.vertices.extend_from_slice(&[...])`), batching all four quad vertices in a single memory operation. In `crates/engine/src/raster.rs`, `pixmap_to_image` pre-sizes pixel buffers (`vec![0u8; pixels.len() * 4]`) and demultiplies straight-alpha channels via `chunks_exact_mut(4)` to enable auto-vectorization across procedurally rendered stage surfaces and effects.

## Bytecode Direct Trigger Dispatch

`Host` in `crates/engine/src/expr.rs` and the virtual machine in `crates/engine/src/bytecode.rs` introduce `trigger0` and `trigger1` direct dispatch hooks. For unary trigger evaluation (`Op::Trig1`), stack operands evaluate directly into trigger logic without wrapping into temporary `[Node::Num]` AST arrays or indirection through `eval()`. Hot path triggers (`var`, `fvar`, `sysvar`, `animelemno`, `animelemtime`, `animexist`, `selfstatenoexist`) execute as inline array indices and lookups, reducing worst-case simulation tick latency to 0.042 ms.
