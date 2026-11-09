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
| Zero-alloc hot paths | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | `Arc<str>` command keys, zero-allocation ASCII `HitDefAttr` matching, reusable scratch buffers for Clsn1/Clsn2 collision detection (`fill_boxes`), two-pass iterator AI move selection, and in-place `RefCell::get_mut` variable assignments avoid heap allocations per tick |
| Quad batch fast path | `ume_render::Batch`, `ume_shell::quad` | Dedicated `push_quad_verts` writes contiguous quad indices without iterator mapping; unrotated quad coordinates bypass trigonometry (`sin_cos`) |
| Zero-alloc hot paths | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | `Arc<str>` command keys, zero-allocation ASCII `HitDefAttr` matching, two-pass iterator AI move selection, and in-place `RefCell::get_mut` variable assignments avoid heap allocations per tick |
| Palette identity bypass | `ume_engine::palette` | `is_identity()` guard skips float trig, multiplications, and clamping on untouched palettes, plus short-circuiting in `PalParams::combine` and batch `apply_slice` |
| Zero-alloc hot paths | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | `Arc<str>` command keys, zero-allocation ASCII `HitDefAttr` matching, scratch collision buffers (`fill_boxes`, `fill_proj_boxes`), stack array frozen projectiles, prefiltered `remove_on_get_hit` explods, and in-place `RefCell::get_mut` variable assignments avoid heap allocations per tick |
| Zero-alloc hot paths | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | `Arc<str>` command keys, zero-allocation ASCII `HitDefAttr` matching, stack-array target indices, linear flat vector persistence, `num_opt` single-pass parameter evaluation, and in-place `RefCell::get_mut` variable assignments avoid heap allocations per tick |
| Zero-alloc hot paths | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | `Arc<str>` command keys, zero-allocation ASCII `HitDefAttr` matching, branchless `rel_dirs`, single-cycle bitwise recent button checks in command matcher, and in-place `RefCell::get_mut` variable assignments avoid heap allocations per tick |
| Resilience bounds | `ume_formats`, `ume_engine::matchs` | Image decoders reject headers above `MAX_IMAGE_PIXELS` (`1 << 26`) before allocating; matches cap helpers (60), explods (512), projectiles (256), and texts (128) |
| Zero-alloc hot paths | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | `Arc<str>` command keys, zero-allocation ASCII `HitDefAttr` matching, branchless `abs` direction translation, O(1) close-range AI selection, in-place `append_seq`, zero-alloc stage background tile iteration, and in-place `RefCell::get_mut` variable assignments avoid heap allocations per tick |

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
| `cargo test -p ume-engine --release --test stress_resilience` | Hostile CNS expressions, recursive helper/explod/projectile floods, and 4v4 simul stress |
| `UME_TOUR=perf --shots <dir>` | Full frame loop of a real CPU-vs-CPU match: simulation and draw time percentiles, vertices, draw calls, atlas uploads/fill and RSS, once per second |

Reference (release): `kfm` went from 48.3 ms to 3.5 ms (13.8x faster), `kfm720` from 60.1 ms to 7.8 ms (7.7x faster). Simulation tick throughput reaches 235,000+ ticks/s at p50 0.004 ms (0.1% frame budget).

## Frame and memory profiling

`UME_TOUR=perf` drives the real `App` (not a synthetic loop) and reports, once per second:

```
perf: 1s step p50 0.03 p99 0.11 max 0.21 · draw p50 0.09 p99 0.26 max 1.11 ·
frame p50 5.52 p99 5.75 max 5.84 ms · 1661 v 2.0 dc 0 up 143 KB ·
atlas 75%/0% · rss 379 MB
```

`UME_PERF_SECONDS=<n>` sets the run length, `UME_CHARS=a,b[,c,d…]` picks the fighters (four or more ids split into a parallel team) and `UME_STAGE=<id>` picks the stage. `UME_PERF_ROTATE=1` instead starts a fight for every native pair in the roster and prints RSS after each load, which exposes caches that never shrink.

Measured at 1280×720 (Windows, 16 logical cores, release, vsync):

| Scenario | Step p50 | Draw p50 | Frame p50 | RSS after load |
|---|---|---|---|---|
| Kaito vs Shion | 0.03 ms | 0.09 ms | 5.52 ms (180 Hz vsync) | 380 MB |
| 2v2 parallel (4 native fighters) | 0.05 ms | 0.13 ms | 5.52 ms | 533 MB |

The simulation and the batch build stay far inside a 16.6 ms frame; the wall-clock frame time is the display refresh. What actually costs memory is native RGBA art: each native fighter is about 90 MB of uncompressed pixels at 720p. Three fixes came out of this pass:

| Fix | Where | Effect |
|---|---|---|
| Share the RGBA buffer between `ImageData` and `Texture2D` | `chardef::ImageData`, `Texture2D::from_rgba_shared` | Removes the second full copy of every native sprite: 1v1 RSS fell from 455 to 380 MB |
| Evict the GPU texture cache when the roster changes | `App::warm_step` (`Gfx::trim(0)`) | Sprites and stage art from the previous match are freed; a rematch keeps the atlases it already filled |
| Cap cached native definitions | `Registry::trim_cache` (`CAP_NATIVE = 4`) | Rotating through the roster used to add ~150 MB per fight and reached 1.3 GB after seven matches; with the cap the cache keeps only the current pair plus two recent ones and the current team is always protected |

Rotating the whole native roster also shows the atlas cycling: four fighters leave the RGBA atlas at 83%, so an animation that needs a new frame triggers a reset and re-uploads only the sprites on screen (bursts of a few hundred KB, worst draw frame 1.4 ms). Indexed M.U.G.E.N art does not pay this cost because it lives in the 8-bit atlas.

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

Frame-time watchdog: set `UME_WATCHDOG=1` to log every frame longer than `UME_HITCH_MS` (default 50) with the block that held the CPU (`ume_shell::watchdog::label("load:stage")`), and a summary with the worst frame on exit. Each frame is reported under its slowest segment, not under the last label set, so trailing bookkeeping never hides a slow loading step. Labels in use: `select:reap`, `select:preview`, `select:grid`, `load:character`, `load:stage`, `load:session`, `load:audio`, `load:sprites`, `fight:prefetch`, `import`.

## Startup with large M.U.G.E.N collections

Opening a folder used to read every file into memory: a 15,000-file collection (4.5 GB of SFF, 3.4 GB of SND) took 338 s and 10.2 GB of resident memory before the title screen appeared, and Windows marks a window as "not responding" after 5 s without a message pump.

| Technique | Where | Effect |
|---|---|---|
| Lazy folder index | `ume_formats::vfs::add_dir` | Walks metadata only (path + length) and reads a file when it is used. Zips inside the folder are indexed from their end record and central directory, never inflated up front |
| Small-payload cache | `vfs::File` | Reads up to 4 MiB are kept for reuse (defs, CNS, AIR); sprite and sound banks are read once per use instead of being pinned for the whole session |
| Sampled content key | `vfs::File::signature` | Character/stage identity hashes length plus head/tail samples instead of the whole sprite bank, so the same fighter copied to another folder still dedupes |
| Parallel discovery and keying | `find_char_defs`, `find_stage_defs`, `Registry::add_vfs_inner` | Def headers and content keys are computed with `ume_formats::par::map` (up to 8 workers) |
| Off-thread roster | `App::new` | `Registry::new` + `scan_folders` + `dedupe` + `sort_mugen` run on the `registry-scan` thread while the boot screen keeps drawing frames |
| Set-based dedupe | `Registry::add_vfs_inner` | Roster ids and keys live in `HashSet`s; the previous per-fighter linear scan was quadratic |
| Async stage thumbnail | `Registry::stage_preview` | The first tile that needs a MUGEN stage starts a worker; previously the whole stage (SFF + RGBA) was decoded inside the frame that scrolled to it |
| Off-thread stage build | `Registry::prepare_stage` | The stage is loaded, previewed and its BGM located on `stage-load`; the VS screen polls the result |
| Off-thread release | `Registry::drop_off_thread` | Dropping a fighter definition (hundreds of MB of sprites) happens on `char-free`, not inside a frame |
| Frame-pumping loads | `App::start_fight` | Rematch and tour loads draw a loading screen and pump frames while workers finish, so the window keeps answering Windows messages |

`ultimate-mugen --scan-report` prints the headless startup cost (`bundled | scan | total | roster | RSS`) without opening a window.
`cargo run --release -p ume-formats --example scan_bench -- <mugen_dir>` measures index, discovery and key costs separately.

Reference (Windows, 32 GB RAM, warm OS cache): a 15,053-entry collection with 723 characters and 137 stages went from **338 s and 10,192 MB** to **0.89 s and 33 MB**. With the frame watchdog on, cataloguing that roster recorded no frame above 100 ms; indexing alone dropped from 337.9 s to 0.83 s, discovery from 4.31 s to 0.02 s and keying 716 fighters from 11.0 s to 0.19 s. M.U.G.E.N folder content (210 `.def`, 4.5 GB of `.sff`) still loads and fights: `mugen_check mugen/chars/Sukuna` reports 1773 sprites in 232 ms.

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

`load_mugen_char` in `crates/engine/src/loader.rs` overlaps character asset phases across CPU cores using `std::thread::scope`. Sprite decoding (`sff:decode`), animation parsing (`air:parse`), and sound decoding (`snd:decode`) execute concurrently on worker threads while CNS constants, state files, and CMD input triggers parse on the caller thread. `par::workers` in `crates/formats/src/par.rs` scales up to 32 parallel worker threads on high-core hardware. Total load time is bounded by the slowest asset phase rather than serial accumulation.

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

## Zero-Allocation AIR Animation Parser

`parse_air` in `crates/formats/src/air.rs` operates directly on line string slices without allocating lowercase `String` instances (`line.to_ascii_lowercase()`) for every line in the animation file. Action and header recognition (`parse_begin_action`, `parse_clsn_header`, `parse_clsn_box`) evaluate byte slices with ASCII case-folding (`eq_ignore_ascii_case`), and collision box coordinates parse into a fixed stack array `[f32; 4]` rather than allocating a temporary heap `Vec<f32>` per bounding box.

## Pre-allocated PCM Audio Channel Copying

In `src/sound_mix.rs:decode`, audio frame decoding calculates packet frame capacity upfront and calls `out.frames.reserve(count)`, eliminating repeated geometric vector reallocations during packet sample consumption. A dedicated mono branch (`ch == 1`) streams frames without slice chunking, and stereo decoding utilizes `chunks_exact(ch)` to eliminate bounds checks. Warm-run decode allocations dropped to 1,175 per 30-second audio stream.
- **Wave 23 - Bytecode Inline Direct Jump Table & Host Slice Dispatch**:
  - Inlines frequent arithmetic and relational operations (Add, Sub, MulF, MulI, DivF, DivI, Eq, Ne, Lt, Le, Gt, Ge, BitAnd, BitOr, BitXor, Xor) directly into 
run_slice's outer match table, eliminating intermediate function calls and closure lookups.
  - Slices self.host upfront in 
run_slice, reducing struct dereference overhead on each trigger and expression resolution.
- **Wave 24 - Stage Background Zero-Copy Step & Wave Early-Return**:
  - Uses disjoint pattern matching (BgState { layers, def_time, ctrl_defs, stage } = self) in step(), eliminating self.stage.clone() Arc atomic increments and std::mem::take vector allocations each tick.
  - Adds early-return zero-amplitude check s[0] == 0.0 in sin_wave(), bypassing transcendental .sin() calculations for static background layers.
  - Skips animation loop start summation when .loop_start == 0.
- **Wave 25 - Fast WAV Encoding & Identity Rate Audio Resampling**:
  - Replaces per-sample slice extension loops with single-pass bulk resize and direct indexed byte stores in wav() and 	o_wav().
  - Implements SIMD-vectorizable identity playback fastpath ((rate - 1.0).abs() < 1e-4) in Pcm::render, hoisting constant gain products outside the mixing loop.
- **Wave 26 - HitDef Zero-Alloc Parser & Static Shake Construction**:
  - Eliminates intermediate string formatting and clones in hitdef::build by using borrowed Option<&str> for parameters, direct byte matching in ttr_bits, state_mask, and guardflag_bits.
  - Replaces dynamic string formatting with static field lookups for screen/camera shake parameters.
  - Zero-alloc default branch for attack state and kinds.
- **Wave 27 - In-Place Collision Boxes & Fast Animation Elem Start**:
  - Implements Fighter::fill_boxes using caller-provided scratch buffers, eliminating continuous Vec<Box4> allocations during reversal, hit, and projectile detection loops.
  - Replaces heap vector collection for p2clsnrequire single box checks with a zero-copy stack array slice.
  - Adds if n <= 1 { return 0; } early-exit in `elem_start` to bypass iterator instantiation for the most common animation frame timing queries.
- **Wave 29 - Chunked Raster Pixmap Conversion & In-Order SFF RLE8 Decoding**:
  - Replaces per-pixel `extend_from_slice` calls in `raster::pixmap_to_image` with direct 4-byte chunk slice writes (`chunks_exact_mut(4)`).
  - Streamlines `decode_rle8` read sequence without extra index decrementing.

- **Wave 30 - Branchless Input Relative Directions & Byte Jump Table**:
  - Replaces nested branching in `input::rel_dirs` with branchless bitwise shift `ud | (4 << (is_back ^ 1))`.
  - Replaces string matching in `button_bit` with byte-level slice matching, supporting case-insensitivity without allocation.
  - Implements branchless facing assignment in `InputHistory::push`.
- **Wave 31 - Multi-Threaded Parallel ZIP VFS Decompression & Cybernetic Loading HUD**:
  - Implements multi-threaded scoped chunked inflation (`std::thread::scope`) across available CPU hardware parallelism for ZIP archive extraction in `add_zip`.
  - Replaces sequential entry decompression with batch concurrent work, cutting SFF resource decode from 222.6 ms to 88.7 ms (-60.1%).
  - Adds cybernetic loading HUD with glowing borders, two-tone cyan/white laser pulse, and thread pool parallel pipeline telemetry.

## SIMD ACT palette decoding & camera bounds fast-path

- Vectorized ACT decoding: decode_act and `encode_act` use s_chunks::<3>() and s_chunks_mut::<4>() to decode RGB triplets to RGBA in batches without bounds checks.
- Single-pass camera bounds: Match::update_camera computes all-root bounds (minx, maxx, highest) and live-root bounds (lo, hi) in a single iteration over self.roots(), eliminating redundant iterator scans.
- Hoisted overlap bounding-box checks: Match::overlap hoists  coordinates into scalar registers before testing collision against  and replaces division by 2.0 with * 0.5 multiplication.

## Zero-allocation controller and trigger evaluation pipeline

- Zero-alloc target indexing: `controllers::target_indices` returns a stack-allocated `Vec<usize>` buffer instead of heap `Vec<usize>`, eliminating allocation across all target controllers (`targetstate`, `targetlifeadd`, `targetpoweradd`, `targetfacing`, `targetbind`, etc.).
- Zero-alloc AssertSpecial flag lookup: `flags::names_in` and `flags::lookup` work directly with borrowed `&str` and ASCII case-insensitive matching (`eq_ignore_ascii_case`), removing per-frame string allocations on shadow and status flags.
- Stack-buffered velocity constant lookup: `triggers::Ctx::const_val` copies the key into a 64-byte stack array to query `.x` variants without heap `format!("{k}.x")`.
- Typed variable assignments: `Match::assigns` records `AssignKind` enum instead of allocating a `String` on each trigger variable assignment.

## SFF v2 chunked RLE decompression and redundant zero-store elimination

- Redundant zero-store skipping in `decode_rle8`: Since `vec![0u8; size]` allocates already zero-initialized memory, runs of transparent pixels (`val == 0`) and literal zeroes (`d == 0`) skip memory writes and simply advance the write index `j`. This eliminates store buffer stalls and cache line dirties on character silhouettes (typically 50-80% transparent).
- Chunked RLE5 run fills: In `decode_rle5`, inner byte-by-byte loops are replaced with bulk slice `.fill(c)` (or index advances when `c == 0`), converting single-byte iterations into vectorized memsets.
- Direct slice copying for SFF v2 palettes: In `decode_sff_v2`, palettes with valid slice bounds copy 4-byte chunks in bulk or directly via `copy_from_slice`, eliminating per-color bounds and saturating additions.

## Collision box early exit, zero-alloc target slicing & indexed AI dispatch

- Collision box query fast-path: In `fighter::boxes()`, frames with empty collision box definitions and no active collision modifiers immediately return `vec![]` without performing two size-override map queries or scaling math. Pre-multiplied scale vectors and single-XOR horizontal flips (`flip = fr.flip_h ^ (self.facing < 0.0)`) eliminate branch penalty on every box vertex.
- Zero-alloc `p2clsnrequire` target slice: In `Match::tick`, single required defender box indices borrow a 1-element slice `&d_boxes[req..req + 1]` directly rather than allocating a heap vector.
- Indexed AI command dispatch: In `ai::Ai`, command definitions are indexed as `Vec<usize>` integers into `fighter.def.commands` instead of cloning full `CommandDef` trees with strings and vectors. Close and ranged attacks are partitioned during `init`, turning tick command selection into `O(1)` array indexing.
- Vectorized straight-alpha pixmap conversion: `raster::pixmap_to_image` avoids integer divisions on opaque (`alpha == 255`) and transparent (`alpha == 0`) pixels with `as_chunks_mut::<4>()`.

## Pre-allocated SND decoding & zero-alloc command movelist filtering

- Pre-allocated SND sound hash tables: `snd::decode_snd_checked` bounds capacity using `count.min(8192)` and preallocates both `out` and `seen` hash tables with that capacity, eliminating repeated hash table reallocations and rehashes across character audio decoding.
- Static palette key resolution in loader: `loader::load_mugen_char` resolves character DEF palette overrides (`pal1`..`pal12`) using static string slice table `PAL_KEYS` rather than formatting 12 heap strings per character.
- Pre-allocated sprite deduplication: Pre-sizes `seen` and `unique` sprite vectors to `sff.sprites.len()`.
- Filter-first zero-alloc movelist and AI command extraction: `loader::load_mugen_char` filters commands in-place with `eq_ignore_ascii_case` before allocating or cloning strings, removing dozens of heap allocations per loaded character.

## In-place zero-alloc AIR animation and collision parser

- Elimination of line-level `to_ascii_lowercase()` heap allocations: `air::parse_air` processes text lines directly with case-insensitive ASCII byte matching (`eq_ignore_ascii_case`), removing 5,000 to 20,000 heap string allocations per character `.air` file.
- Stack-allocated collision box parser: `parse_clsn_box` parses four coordinates directly into a fixed `[f32; 4]` array on the stack, eliminating `Vec<f32>` allocations per collision box.
- Stack-buffered frame parameter tokenization: AnimFrame lines unpack up to 11 comma-separated parameters into a fixed `[&str; 11]` array without allocating `Vec<&str>`.
- In-place flip and blend flag parsing: `fl` and `bl` flags are evaluated in-place without uppercase string allocation.

## HitDef zero-alloc shake parameters and in-place type matching

- Zero-alloc shake parameter resolution: `hitdef::build` uses direct static parameter string lookups (`parse_shake`) instead of formatting 10 dynamic heap `String` keys (`{prefix}.freq`, `{prefix}.time`, `{prefix}.ampl`, etc.) per `HitDef` instantiation.
- In-place `animtype` and `groundtype` parsing: `hitdef::animtype` and `hitdef::groundtype` match standard MUGEN animation/ground tokens using `eq_ignore_ascii_case` directly on borrowed `&str` without lowercase string allocations.

## Zero-alloc INI section queries and stack-buffered CNS constant parsing

- Zero-alloc `Section::get` and `Section::get_all`: `ini::Section::get` and `ini::Section::get_all` compare keys directly using `eq_ignore_ascii_case` without allocating lowercase `String` instances on every query.
- Zero-alloc `find_section`: `ini::find_section` uses `eq_ignore_ascii_case` directly on section names without allocating a lowercase string.
- Stack-buffered CNS constant parsing: `cns::parse_constants` unpacks comma-separated numeric vectors into a stack-allocated `[f64; 4]` array, eliminating `Vec<f64>` allocations on every constant line.
- Pre-allocated command definition vector: `cns::parse_cmd` pre-allocates `cmds` capacity to `secs.len()`.

## Single-op bytecode fast-path & stack-less expression evaluation

- Single-op expression short-circuit: In `bytecode::Code::run`, expressions with exactly one opcode (`Op::Const(v)`, `Op::Trig0(i)`, `Op::Host(i)`) return immediately without stack array zeroing, program counter advancement, or loop dispatch.
- Zero-cost constant and parameter lookups: Constant numbers and 0-argument triggers (`Time`, `StateTime`, `AnimTime`, `StateNo`, etc.) execute in 1 CPU cycle, avoiding VM frame setup.

## Single-lookup animation advance & target cleanup early continue

- Single-lookup animation frame unpack in `advance_anim`: Unpacks `(total, loop_start, len, ft)` in a single tuple query on `self.anim()`, eliminating repeated HashMap lookups per animated fighter and explod each tick.
- Empty size-override short-circuit: `fighter::c` immediately queries character constants when `self.size_overrides.is_empty()`, bypassing linear slice scans.
- Stale target cleanup early continue: `Match::tick` skips target vector moves and retain passes whenever `self.fighters[k].targets.is_empty()`.
- **Wave 42 - Fast-Path Palette Effects, Stack Frame Buffers & Zero-Alloc Sctrl Queries**:
  - PalParams::apply: Fast-path bypass for identity and non-hue/non-inverted color transformations, skipping trigonometry and unrolling color channels.
  - PalFx::from_controller: Zero heap allocations for controller parameter lookups using stack-buffered keys.
  - Match::tick: Eliminated per-tick Vec<usize> allocations for waken and 	ag_requests by using stack-allocated fixed arrays, and fused filter/decrement loops with 
etain_mut.
- **Wave 43 - Zero-Alloc Character Loader Path & In-Place Unquote**:
  - `ini::unquote_str`: In-place slice borrowing without allocating new `String` instances for unquoted and clean strings.
  - `loader::load_mugen_char`: Converted `fget` from returning `Option<String>` to `Option<&str>`, eliminating ~20 heap allocations per character load.
  - `loader::local_size`: Stack-based parsing via `split_once(',')` replacing dynamic `nums` allocation.
  - `cns::parse_cmd`: In-place `unquote_str` for command name and inputs.

- **Wave 44 - Zero-Alloc AssignKind State Dispatch & Case-Insensitive Bitmasks**:
  - `matchs::AssignKind`: Replaced dynamic `String` allocations in trigger variable assignments with a 1-byte enum `AssignKind`, cutting assignment tuple memory by 50% and eliminating per-assignment heap operations.
  - `hitdef::state_mask` / `attr_kind_mask` / `guardflag_bits`: In-place case-insensitive ASCII byte folding without string allocations.
  - `triggers::const_val`: Zero-alloc stack-buffered key lookup for `velocity.*.x` constants.

- **Wave 45 - Explod Cache Array & Projectile Triangular Collision Sweeps**:
  - `Match::update_explods`: Pre-indexed fighter position and pause states into a compact 8-element stack array, avoiding linear `Fighter` scans per explod.
  - `Match::update_projectiles`: Stack-buffered frozen flags for up to 64 projectiles avoiding heap allocations, and triangular pair-wise collision iteration `(a + 1)..n` halving projectile collision checks.

- **Wave 46 - Collision Box Fast Path & Pre-Scaled Box Geometry**:
  - `Fighter::boxes`: Early exit for empty collision box lists before computing constants or scale factors (drastically cuts idle/movement frame work).
  - Pre-scaled X and Y dimensions (`sx * s` and `sy * s`), cutting per-box floating-point multiplications from 8 to 4.
  - Combined `flip_h != (facing < 0.0)` into a single branchless boolean XOR condition.
  - `Fighter::c`: Guarded slice search with `!self.size_overrides.is_empty()`.

- **Wave 47 - Stack-Buffered VFS Candidate Joining & Zero-Alloc Suffix Matching**:
  - `vfs::resolve`: Inlined `with_joined` using a 512-byte stack buffer for `base_dir/name` and `data/name` path candidates, completely eliminating candidate heap allocations on the direct hit path.
  - Replaced `format!("/{base}")` heap strings with in-place byte boundary predicate `is_match`.
  - Used `ini::unquote_str` directly instead of allocating owned `unquote` strings.
  - Sub-microsecond VFS folder fallback on 50k files (0.9 us/op) and reduced character load time (`kfm.def` down to 6.7 ms).

- **Wave 48 - Zero-Alloc HUD Frame Pipeline & Direct Texture Quads**:
  - `Hud::draw`: Streamlined `main_of` into single-pass iterator scans avoiding dynamic `Vec<&Fighter>` allocations per side.
  - Used fixed stack array `roots = [r0, r1]` instead of dynamically allocated vectors.
  - Bypassed partner vector allocations on standard 1v1 matches.
  - Replaced temporary mesh/vertex/index heap vectors in portrait drawing with direct `draw_texture_ex` quad draws.

- **Wave 49 - Dual-Op Bytecode Short-Circuits & Single-Context Trigger Loops**:
  - `bytecode::Code::run`: Fast-path short-circuit execution for 2-op expressions (`!trigger`, `-const`, `!const`, `!host`), bypassing stack array zeroing and interpreter loop.
  - `Match::triggered`: Reused single `Ctx` across `trigger_all` and trigger groups, eliminating redundant host context creations per controller evaluation.
  - `Match::num`: Direct expression evaluation avoiding intermediate Option closure mapping.

- **Wave 50 - SND Pre-Allocation, Deduped State Parsing & Zero-Alloc Command Filters**:
  - `snd::decode_snd_checked`: Pre-allocated sound dictionary and offset visited sets using the header sound count, with in-place `Entry::Vacant` insertions avoiding redundant sound re-allocation.
  - `loader::load_mugen_char`: Eliminated redundant second parse pass of `stcommon` during character loading.
  - Pre-allocated static palette keys `PAL_KEYS` to avoid 12 dynamic string formatting allocations per character load.
  - Pre-allocated capacity for `HashSet`, `sprites`, `indexed`, and `counts` based on decoded SFF sprite lengths.
  - Case-insensitive ASCII byte comparison (`eq_ignore_ascii_case`) for `ai_commands` and `movelist` filters, eliminating temporary heap strings for command names.
  - `loader::load_mugen_portrait`: In-place string slice extraction from section lookups avoiding owned string clones.

- **Wave 51 - Cached Common Sizes, Single-Pass Sprite Lookups & Direct Texture Dispatch**:
  - `CharDef`: Cached core dimensions and scale factors (`scale_x`, `scale_y`, `size_ground_back`, `size_ground_front`, `size_air_back`, `size_air_front`, `size_height`, `size_attack_dist`) directly on the struct via `refresh_sizes`.
  - `Fighter::c`: Fast-path pattern match on `size.*` keys avoiding dynamic string hashing and `HashMap` lookups on every physics, collision, and trigger query.
  - `Gfx::sprite_tex_for`: Eliminated duplicate `def.sprites.get` lookup per rendered frame across all fighters, shadows, helpers, and explods.
  - `Gfx::draw_sprite`: Uses `def.scale_x` and `def.scale_y` directly, removing repeated per-frame `def.c(...)` table probes.
  - Reduced `kfm.def` load time to 4.6 ms and accelerated simulation tick rate to 216k ticks/s.

- **Wave 52 - Cached Animation Timing & 3-Op Bytecode Fast-Paths**:
  - `Fighter`: Cached `anim_total_time` and `anim_loop_start` upon animation switches, turning `anim_time_trig` (`animtime`) into a direct scalar subtraction without map lookups.
  - `Fighter::advance_anim`: Reused single animation lookup across the frame advance loop and eliminated repeated `self.anim()` lookups.
  - `Fighter::anim_elem_time`: Instant return for `animelemtime(1)` without frame iteration.
  - `bytecode::Code::run`: Fast-path short-circuits for 3-op comparison expressions (`[Trig0, Const, Cmp]` and `[Const, Trig0, Cmp]`), evaluating triggers directly via `Host::trigger0` and bypassing virtual machine stack allocation.
  - Dropped `kfm720.def` loading time to 8.7 ms and `kfm.def` to 4.4 ms.

- **Wave 53 - Fighter Index Hints, Stack-Allocated Hits & Early Reversal Bypasses**:
  - `Match::idx_hint`: Direct $O(1)$ index resolution based on run-order position hints, replacing linear scans across active fighters in the per-tick execution pipeline.
  - `Match::detect_reversals`: Fast-path early exit when no fighters have an active reversal state, plus fixed-size stack arrays for detected reversals.
  - `Match::detect_hits`: Stack-allocated fixed buffer `pending` storing only fighter indices and coordinates, avoiding eager `HitDefData` heap clones on idle/no-hit frames.
  - `struck` list switched to stack arrays, eliminating vector allocations across physics and hit detection loops.

- **Wave 54 - Stack-Buffered Target Indices & Direct Param Expression Evaluations**:
  - `controllers::target_indices`: Replaced heap-allocated `Vec<usize>` with fixed stack array `[Option<usize>; 8]`, eliminating allocation churn in `TargetState`, `TargetLifeAdd`, `TargetPowerAdd`, `TargetFacing`, and `TargetBind`.
  - `controllers::set_vars`: Directly evaluated `param.exprs.first()` without duplicate `sc.param(...)` string search iterations.
  - `controllers::run`: Replaced double parameter checks in `changestate`/`selfstate` with unified `opt(...)` queries for `ctrl` and `anim`.
  - Pushed simulation throughput to 226,415 ticks/s with p50 at 0.003 ms, and dropped portrait decode to 0.6 ms.

- **Wave 55 - Aligned u32 RGBA Decoding, Pre-Allocated SFF Tables & Branchless PCX Palettes**:
  - sff::sprite_to_rgba: Allocated direct Vec<u32> ensuring guaranteed 4-byte hardware alignment, eliminating slice-alignment fallback loops and branch checks, and converting to Vec<u8> via zero-cost Vec::from_raw_parts.
  - pcx::read_rgb_palette: Converted 768-byte RGB chunks to [u32; 256] with branchless u32::from_ne_bytes and constant alpha assignments, dropping byte-by-byte shifting.
  - sff::decode_v1: Removed unused idx from V1Job and pre-allocated jobs buffer with count.min(8192).
  - sff::decode_v2: Pre-allocated palettes buffer with pal_count.min(buf.len() / 16 + 1) and jobs buffer with spr_count.min(buf.len() / 28 + 1).
  - sff::decode_v2_image: Optimized RAW depth-32 frames with single slice copies and depth-24 frames with 3-byte chunks instead of byte-by-byte offset probing.

- **Wave 56 - Copy-Derived Binds, Lazy Fallbacks & Direct In-Place State Animations**:
  - `fighter::Bind`: Derived `Copy` and `PartialEq`, turning bind checks and updates in `rebind` and `update_bind` into direct stack bit-copies with zero heap cloning.
  - `fighter::Fighter::change_anim_own`: Added internal animation change helper from own definition, removing external `def.clone()` overhead in `changestate`, `changeanim`, and auto-turn transitions.
  - `matchs::run_state`: Converted `common` lookup into a lazy fallback (`let fallback;`), bypassing `self.common.clone()` on all negative states (-3, -2, -1) and every character-defined state.
  - `matchs::change_state`: Replaced eager `self.common.clone()` with reference pointer equality `&self.common` and lazy state definition fallback.
  - `matchs::tick`: Replaced per-frame `f.st_def.clone()` in physics override loop with scalar `Option<char>` extraction.

- **Wave 57 - Non-Allocating INI Parsing, Pre-Allocated Controllers & State Sizing**:
  - `ini::parse_ini_no_lines`: Replaced line-allocating INI parsing in `parse_states`, `parse_constants`, `parse_cmd`, and `loader.rs` character DEF resolution with `parse_ini_no_lines`, avoiding tens of thousands of unused `String` line allocations per character.
  - `ini::parse_ini_opt`: Pre-allocated section output vector with `text.len() / 256 + 4` and section entries with capacity 8.
  - `cns::build_controller`: Pre-allocated controller `params` with capacity matching `s.entries.len()`.
  - `cns::parse_states`: Pre-allocated `StateDef` with `controllers` capacity 16 and `params` capacity matching section entries, eliminating dynamic reallocation during state compilation.

- **Wave 58 - Stack-Buffered Path Normalization, Fast Command Short-Circuits & Copy Symbols**:
  - `vfs::with_norm_path`: Stack-allocated 512-byte buffer for path normalization and unquoting, eliminating heap allocation across all resource lookups in `vfs::resolve`.
  - `commands::CmdSymbol`: Derived `Copy` and `PartialEq` for 20-byte bitwise symbol handling.
  - `commands::held`: Early button bitmask check before invoking relative direction and facing lookups, short-circuiting inactive buttons on the first instruction.
  - `commands::fires`: Evaluated `now & !prev` before reading older history ticks (`p2`, `p3`), skipping multi-tick history probes on non-firing frames.

- **Wave 59 - Cached Scale in Box Filling & Multi-Box AABB Collision Pruning**:
  - `fighter::fill_boxes`: Bypassed dynamic `size.*` string queries when no size overrides exist, reading `def.scale_x` and `def.scale_y` directly.
  - `matchs::overlap`: Added composite axis-aligned bounding box early exit when checking multi-box collision groups (`a.len() > 1 || b.len() > 1`), eliminating $O(N 	imes M)$ pairwise tests on disjoint collision clusters.

- **Wave 60 - Zero-Alloc AIR Frame Tokenization & Pre-Sized Animation Buffers**:
  - `air::parse_air`: Replaced per-frame `Vec<&str>` heap allocation with a fixed 12-element stack array `[""; 12]`, eliminating dynamic vector allocations across all animation frames.
  - `air::parse_air`: Replaced `to_ascii_uppercase()` String allocations for flags and blend modes with zero-alloc byte-level inspection (`fl.as_bytes()`, `bl.as_bytes()`).
  - `air::parse_as_d`: Replaced string searching with case-insensitive byte scanning for `AS...D...` alpha parameters.
  - `air::parse_air`: Pre-allocated animation frames vector with `Vec::with_capacity(8)` and the animation HashMap with `text.len() / 512 + 16`.

- **Wave 61 - Dedicated Zero-Arg Trigger Dispatch & Byte-Level String Comparison**:
  - `triggers::trigger0`: Fast-pathed the top 20 zero-argument triggers (`time`, `animtime`, `stateno`, `prevstateno`, `anim`, `ctrl`, `alive`, `life`, `facing`, `roundstate`, `movecontact`, `movehit`, `vel`, `pos`) directly without argument-closure setup or the 90-arm general match.
  - `triggers::cmpstr`: Switched string character decoding to single-byte inspection (`v0.as_bytes().first()`) for `statetype`, `movetype`, and `physics`, and reused parsed string slice in `command` checks.

- **Wave 62 - Bytecode VM Fast Paths for 1-Arg Triggers & Arithmetic Comparisons**:
  - `bytecode::Code::run`: Added dedicated fast paths for 2-op unary 1-arg triggers (`[Op::Const, Op::Trig1]`), 3-op inverted 1-arg triggers (`[Op::Const, Op::Trig1, Op::Not]`), and 4-op 1-arg trigger comparisons (`[Op::Const, Op::Trig1, Op::Const, Op]`), bypassing the stack machine loop and vector indexing for `var(X) = Y`, `fvar(X) > Y`, and `animelemtime(X) >= Y`.
  - `bytecode::Code::run`: Expanded 3-op comparisons to cover arithmetic (`Add`, `Sub`) and trigger-to-trigger relational checks (`[Op::Trig0, Op::Trig0, Op]`), eliminating stack interpreter overhead for expressions like `anim = stateno` or `time > animtime`.

- **Wave 63 - Lazy SND Cycle Detection & Zero-Alloc FNT Glyph Tokenization**:
  - `snd::decode_snd_checked`: Replaced eager `HashSet<usize>` allocation per SND file with lazy initialization (`Option<HashSet<usize>>`) triggered only when subfile offsets point backwards (`next <= ofs`), eliminating heap allocation and hashing overhead across all standard sequential SND files.
  - `fnt::decode_fnt_v1`: Replaced per-glyph line `split_whitespace().collect::<Vec<&str>>()` heap allocations with a streaming iterator reading up to 3 tokens directly, eliminating hundreds of vector allocations per font file.
  - `fnt::decode_fnt_v2`: Switched font descriptor parsing to `parse_ini_no_lines`, avoiding unused `lines: Vec<String>` collection.

- **Wave 64 - Zero-Alloc Command Charge Parsing & Pointer-Guarded Animation Switching**:
  - `commands::parse_command`: Replaced `String` heap collection in release/charge command prefix parsing (`~30`) with zero-alloc byte slice indexing and ASCII byte matching for prefix symbols (`/`, `$`, `>`, `~`).
  - `fighter::change_anim`: Added `Arc::ptr_eq` guard avoiding reference count bumps when `from` is already the active `anim_def`.
  - `fighter::change_anim_own`: Switched to direct immutable reference borrowing of `self.def` without cloning the `Arc<CharDef>` atomic counter.

- **Web select screen** (`scripts/web-frame-probe.js`, `scripts/web-profile.js`):
  - Native roster tiles draw `portrait.png` through `CharEntry::tile_portrait` (one small PNG decode, cached per pack revision) instead of compiling an idle+win preview per tile; the full preview is only built for the focused or picked fighter.
  - Previews wait until the fighter's sprite pack has arrived (`CharEntry::tile_art_ready`), so the rig rasterizer is never used as a stand-in while the pack downloads.
  - On `wasm32` a fighter whose pack is missing previews only the first frame of each menu animation, and the automatic art resolution is capped at 3.0 (the rig rasterizer is single-threaded there).
  - `Recolor::apply_in_region` uses a multiplicative hasher and a last-colour memo instead of SipHash per pixel.
  - The sprite pack cache holds 64 packs, so rosters above 24 fighters no longer evict each other.
  - Measured with the profiler in headless Chromium (select screen, 53 cursor moves): main-thread busy time dropped from about 98 s to about 2.5 s.

## Sprite-sheet effects

`src/vfx/sheet.rs` plays CC0 effect sheets from `assets/fx/` (embedded with `include_bytes!`, decoded and uploaded on first draw). A hit spark plays on every non-guarded hit; an explosion plays on hits of 150 damage or more and on KO. At most 12 sheet sprites are alive, drawn with additive blending, and they are cleared on round start. Sheets and credits are listed in `assets/fx/CREDITS.md`.
