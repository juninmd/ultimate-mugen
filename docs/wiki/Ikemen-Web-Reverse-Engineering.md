# Reverse engineering a browser M.U.G.E.N port

**English** · [Português (BR)](pt-BR/Engenharia-Reversa-Ikemen-Web.md)

What <https://w8.snokido.com/games/html5/dbz-vs-marvel-vs-capcom/> does with a 78-fighter M.U.G.E.N roster, how its animation system works, why it holds 60 fps, and what of all that we should copy — or not — in Ultimate MUGEN.

Everything below was measured or read from the shipped artifacts. The scripts that produced it live in [`docs/research/ikemen-web/`](../../research/ikemen-web/), so every number can be re-run.

## 1. What the page is

The public page is a 11 KB shell: menus, character select, versus splash and the options screens are plain DOM (that part is the snokido/`runtime-ui.js` UI layer). The fight itself is a separate document, `frame.html`, inside an iframe, and the engine there is **not** JavaScript: it is a Go program compiled to WebAssembly.

| Artifact | Size | Role |
|---|---|---|
| `index.html` | 11 KB | shell: boot, menu, select, options |
| `local-runtime.js` | 342 KB | obfuscated bootstrap: holds the Go `wasm_exec` glue and the frame module **as strings**, plus the content catalog |
| `runtime-ui.js` | 420 KB | the DOM UI (obfuscated, digest-gated) |
| `frame.html` | 2.2 KB | engine host: one `<canvas id="ikemen-canvas">` |
| `runtime-engine.bin` | 5,887,387 B | gzip of a **25,343,508 B** WebAssembly module |
| `content/catalog.shvd.json` | 265 KB | 78 characters, 74 stages, pack URLs, sha256, sizes |
| `content/packs/*.zip` | 1–24 MB each | unmodified M.U.G.E.N files, one zip per character/stage |

The bootstrap posts `{frameJs, wasmExec, descriptor, gzipBytes}` to `frame.html`, which installs `wasm_exec.js` from a string, imports the engine module from a Blob URL and gates it on a SHA-256 of the values it was handed. That is copy protection, not engine design.

```console
$ node docs/research/ikemen-web/wasm-artifacts.js runtime-engine.bin
module bytes: 25343508
[imports "gojs"] 25
  runtime.scheduleTimeoutEvent:function
  runtime.wasmExit:function
  syscall/js.valueGet:function
  syscall/js.copyBytesToJS:function
  mem:memory
[exports] run:function resume:function getsp:function mem:memory
[build info] ikemen-GO v0.0.0-20260927200105-cae6cd63ba4a+dirty
[go source files named by the symbol table] ... render_webgl.go render_webgl_cmdbuf.go system_js.go audio_js.go input_js.go loadyield_js.go ...
```

`gojs` imports plus `run`/`resume`/`getsp` exports are the Go runtime's WebAssembly ABI, and the Go symbol table keeps every source path. The engine is **Ikemen-GO** (the open Go reimplementation of M.U.G.E.N) with a WebGL2 backend added for the browser: `render_webgl.go`, `render_webgl_cmdbuf.go`, `system_js.go`, `audio_js.go`, `input_js.go`, `pause_js.go`, `platform_js.go`, `loadyield_js.go`, `font_webgl.go`, `netplay_js.go`.

The catalog names its own provenance, which is how the backend source was located and cross-checked against the binary:

```json
"source": { "components": [
  { "name": "ikemen-go-web",      "url": "https://github.com/energyjp/ikemen-go-web",             "revision": "54305ec917c3e995f433004e2e8f7c8902bff96a" },
  { "name": "Ikemen-GO-Screenpack","url": "https://github.com/ikemen-engine/Ikemen-GO-Screenpack", "revision": "87f348a5f6e01e878a62ae865c6ee7d3281f8c98" } ] }
```

Upstream `ikemen-engine/Ikemen-GO` has no WebGL backend; `energyjp/ikemen-go-web` does. The checked-out source matches the binary exactly (opcode table, GLSL, symbol names, canvas id), so the "how" below is verified against both.

## 2. Content: no re-encoding, just zips

The packs contain the original M.U.G.E.N files — `.def`, `.sff`, `.snd`, `.air`, `.cns`, `.zss`, `.act` — untouched. Verified on the Goku pack (24,319,368 B, sha256 matches the catalog): `files/goku.sff` 9.6 MB, `files/goku-jap-en-bl.snd` 20.7 MB, `files/goku.air` 287 KB, 21 palette `.png`/`.act` files, plus the character's own movelist HTML.

So the browser downloads **raw M.U.G.E.N content** and the WASM engine parses SFF/SND by itself. Nothing is pre-baked into spritesheets, and only the *front-end* media (portraits, icons, stage thumbnails) is pre-encoded to WebP with content-addressed names (`collections/shvd/media/<sha256>.webp`).

`core` (7.0 MB zip → 9.7 MB) is the screenpack and system data, and every other pack declares `"dependencies": ["core"]`. A match loads: `core` + one pack per fighter + one pack per stage.

## 3. Animation: M.U.G.E.N semantics, unchanged

Animations are `.air` files parsed into the same structures M.U.G.E.N uses (`src/anim.go`):

| Structure | Fields that matter |
|---|---|
| `Animation` | `frames`, `loopstart`, `curtime`, `curelem`, `totaltime`, `looptime`, `interpolate_offset/scale/angle/blend`, `mask`, `transType`, `srcAlpha`/`dstAlpha` |
| `AnimFrame` | `Group`, `Number`, `Xoffset`, `Yoffset`, `Time` (ticks), `Xscale`/`Yscale`, `Angle`, `TransType`, `Clsn1`/`Clsn2` |
| `Anim` (instance) | `anim`, `x`/`y`, `xscl`/`yscl`, `window`, `xshear`, `rot`, `palfx`, `layerno`, `localcoord` |

The per-tick path is deliberately cheap:

1. `Animation.Action()` advances `curtime`; at `totaltime` it jumps to `loopstart` and sets `newframe`.
2. `Animation.UpdateSprite()` runs every tick but only does work when `newframe` is set — that is where the sprite is resolved, with one map lookup: `a.spr = a.sff.GetSprite(uint16(group), uint16(number))` (`anim.go:702`), after applying the character's `Remap` presets.
3. Rendering reads `frames[drawidx]` for the transform and the cached `spr` for the texture. Nothing is re-looked-up per tick, and nothing is interpolated.

So there is **no tweening and no sub-tick interpolation**: frames change on the 60 Hz tick, exactly like M.U.G.E.N. The perceived smoothness comes from the tick rate, from `interpolate offset/scale/angle/blend` (which animate transforms *between* AIR frames), and from the shader doing all colour work in one pass.

### Colour is a shader feature, not a bake

Two things that would normally cost uploads are pushed into the fragment shader (`src/shaders/sprite.frag.glsl`, which is also embedded in the WASM as a string):

```glsl
c = COMPAT_TEXTURE(tex, uv);
if (isRgba) { ... } else {
    c = COMPAT_TEXTURE(pal, vec2(c.r*0.9966, 0.5));   // palette lookup from the red channel
}
if (hue  != 0.0) c.rgb = hue_shift(c.rgb, hue);
if (neg)         c.rgb = neg_base - c.rgb;
c.rgb = mix(vec3((c.r+g.c+b)/3.0), c.rgb, 1.0 - gray);
c.rgb += final_add;
c *= final_mul;
if (!isFlat) c.rgb = mix(c.rgb, tint.rgb * c.a, tint.a);
```

* **Palettes**: an indexed sprite keeps its 8-bit pixels as the texture and is coloured through a `256×1` palette texture (`newPaletteTexture()` → `newTexture(256, 1, 32, false)`). Changing `pal1…pal12` or `RemapPal` re-uploads **1 KB**, not the sprite. `Sprite::CachePalTex` (`image.go:1336`) compares the 256 colour words and only re-uploads on change, so a palette swap costs one `texSubImage2D` per sprite that actually changes.
* **`palFX`** (`add`, `mul`, `color`, `hue`, `invertall`) is per-sprite uniforms on the same draw — no second pass, no `RenderMode` shader switching.
* **Trapezoid distortion** (floor shadows / `xscale` skew) is computed in the same shader from `gl_FragCoord`, not by re-projecting vertices.

One shader covers flat quads, RGBA sprites, indexed sprites, trapezoids and the whole `palFX` chain. M.U.G.E.N's "one more shader per effect" problem simply does not exist there.

## 4. How it stays fast

The interesting part. Measured on the shipped build (headless Chromium, ANGLE/SwiftShader so the GPU numbers are software-rendered, 1280×720 CSS, two runs on a loaded machine):

| Metric | Value |
|---|---|
| Canvas backing store | **640×480** (CSS 1280×720, `object-fit: contain`) |
| `drawArrays` per rAF | p50 **82–105**, p95 113–163, max 172–224 |
| `bufferData` per rAF | 82–125 (one small upload per sprite; runs of text share one) |
| Uniform calls per rAF | 322–397 |
| `useProgram` per second | 30–76 (a handful per frame) |
| `bindTexture` per second | 1.4k–6.2k |
| Engine frames per rAF | **0.78–0.89** (fixed 60 Hz, gated) |
| JS heap in a match | 9–74 MB |
| Boot → menu | 1.9 s |
| Menu → select | 4.0 s |
| Select → first fight | **37 s** (core + 2 fighter packs + stage over the network, then SFF decode) |
| Long tasks during a match | 50–90 ms, recurring (asset decode / GC on the main thread) |

### 4.1 The scene renders at 640×480 into an FBO, then gets blitted

`Init()` creates an RGBA texture + a `DEPTH_COMPONENT24` renderbuffer at `sys.scrrect` (640×480) and draws the whole scene there; `EndFrame()` runs the external post shaders through two ping-pong FBOs and only then does **one** `drawArrays(TRIANGLE_STRIP, 0, 4)` to the default framebuffer with a letterboxed viewport and `NEAREST` (or `LINEAR` if the user picked smoothed scaling). Every sprite is rasterized at 640×480 regardless of window size.

### 4.2 One JS crossing per frame instead of one per GL call

This is the headline trick (`render_webgl_cmdbuf.go`). GL calls are not issued from Go; they are **encoded into a byte stream**:

```go
// GL command buffer: batches the browser build's per-frame WebGL commands into a shared ArrayBuffer
// that a JS-side interpreter drains in ONE syscall/js crossing per flush, instead of one crossing
// per GL command.
//
// Why: profiling the shipped (v0.9) build showed hundreds of `gl.Call` crossings per frame; each
// crossing boxes its arguments through syscall/js.makeValue (~450MB/30s of JS garbage) and pays the
// wasm->JS context switch.
```

* 256 KB staging buffer (`wglCmdBufBytes = 1 << 18`), 23 opcodes (`useProgram`, `uniform*`, `activeTexture+bindTexture`, `bufferData`, `drawArrays`, `enable/disable`, `blendFunc`, `scissor`, `depthMask`, `bindVertexArray`, `texParameteri`, …).
* GL objects (textures, programs, uniform locations, VAOs, buffers) are registered once into a JS array and referenced by integer handle.
* `flushCmds()` does one `js.CopyBytesToJS` + one `Invoke` of a JS interpreter built with the `Function` constructor.
* Every direct GL call goes through `glc()`, which flushes first, so command order is preserved by construction.

### 4.3 Uniform and state deduplication on the Go side

`SetUniformISub`/`SetUniformFSub` keep per-program caches keyed by `(programID << 16) | uniformIndex` and emit nothing when the value is unchanged. `EnableBlending`, `SetDepthTest`, `SetDepthMask`, `EnableScissor` and `ChangeProgram` do the same for GL state. That is why ~14 uniforms per sprite collapse to ~4 on average, and why `useProgram` is only 30–50 calls *per second*.

### 4.4 Zero allocation in the draw path

The same file documents the measured cost of the naive version: passing a local `[16]float32` to an interface method made it escape, allocating vertices + uniform + modelview **per quad**, and `drawQuads` alone allocated **>1 GB per 30 s** of heap. The fix is package-level scratch arrays reused per call (the render path is single-threaded and each buffer is consumed synchronously).

### 4.5 rAF gating to the game's framerate

`system_js.go` does not run a frame per `requestAnimationFrame`; it runs at most one per `1000/Video.Framerate`:

```go
// Browsers fire rAF at the DISPLAY's refresh rate - 120/144/240Hz on high-refresh screens. ...
// Releasing a frame on every rAF therefore runs the whole loop 2-4x more often than the game is
// authored for, multiplying allocation and forcing far more frequent GC pauses (the 200-500ms
// mark stalls high-refresh users saw) for zero visual benefit.
```

`SwapBuffers()` then parks the main goroutine on a channel until the next released frame — which also yields the JS event loop so input, network and audio callbacks run.

### 4.6 Sprites are not atlased; fonts are

Every sprite keeps its own texture. There is no sprite atlas and no eviction policy, so a match can never hit an atlas-full cliff. The 16 texture units are used as an **LRU cache** (`texCacheTexSerial`/`texCacheLastUsed`) so consecutive sprites from the same sheet do not re-bind.

Only **text** uses a `TextureAtlas`: 256×256, skyline packing with edge extrusion, and glyph quads batched per atlas so a run of text is one draw.

### 4.7 One draw call per sprite, by design

`SetVertexData(...16 floats...)` + `RenderQuad()` → `bufferData(STATIC_DRAW)` followed by `drawArrays(TRIANGLE_STRIP, 0, 4)`. Per sprite: 1 `bufferData`, 1 `drawArrays`, ~4 uniforms, ≤1 texture bind — all inside the command buffer, so the Go side pays only a few `emitI`/`emitF` per sprite. At 82–172 sprites per frame that is 5–10 k GL calls/s, which is exactly what a software rasterizer in headless Chromium can absorb.

## 5. What we already do better

Ultimate MUGEN does not need the command buffer: our shell talks to wgpu, which already packs a frame into a few submissions. And our sprite path is architecturally ahead of theirs:

| Concern | Ikemen web build | Ultimate MUGEN |
|---|---|---|
| Draw calls | 1 per sprite (82–172/frame) | batched: quads merged per blend state + scissor into one vertex/index buffer |
| Bind state | per-sprite texture binds, 16-unit LRU | one bind group; atlases + palette texture bound once |
| Palette | `256×1` texture per sprite, per-sprite upload on change | 1024 palette rows in one texture, row index per vertex, `RemapPal` swaps the row |
| Atlas | none for sprites (per-sprite textures) | RGBA atlas + 8-bit index atlas, reset independently |
| Simulation | whole fighter state machine in WASM | `hotpath_bench` (below) |
| Frame pacing | rAF gate at `Video.Framerate` | `ume_shell::Pacer` + fixed 60 Hz step (≤5 ticks/frame) |

Our simulation cost, so the frame budget is not the issue:

```console
$ cargo run --release -p ume-engine --example hotpath_bench
sff decode                   mean      829.6 us
air parse                    mean      493.5 us
cns parse+compile            mean     3680.8 us
char load (no images)        mean     7388.2 us
match tick (2 fighters)      mean        4.2 us   p99  9.0
match tick (6 fighters)      mean       11.3 us   p99 17.6
match tick (10 fighters)     mean       20.1 us   p99 33.5
```

4–20 µs per tick against a 16.7 ms frame: 0.03–0.12 %. The fight is spent in the renderer and the loader, which is exactly where the browser port spent its effort too.

## 6. What we changed

The port's own bug reports (`>1 GB/30 s` of GC from per-quad allocations, `450 MB/30 s` from per-call boxing, 200–500 ms stalls from oversimulating at 144 Hz) all have the same shape: **a whole match's work paid inside one frame**. We had the same shape in one place: the sprite warm-up.

`App::finish_fight` uploaded fighter sprites in the single frame that switched to the fight:

```rust
self.g.trim(4000);
for f in &sess.m.fighters { if !self.g.prefetch(&f.def, 0.5) { break; } }
```

How much art a fighter carries (new `sprite_mem` example):

```console
$ cargo run --release -p ume-engine --example sprite_mem -- mugen/chars/BuuZ2 mugen/chars/cell mugen/chars/frieza
mugen:c/buuz2.def   sprites=2510  indexed=2510 | index atlas  33.71 MB | total  33.86 MB | largest spr 5202 KB
mugen:c/cell.def    sprites=631   indexed=631  | index atlas   9.91 MB | total   9.99 MB | largest spr  818 KB
mugen:c/frieza.def  sprites=673   indexed=673  | index atlas   8.67 MB | total   8.74 MB | largest spr  682 KB
```

A DBZ pair is 43.6 MB of index sheets across 3,141 sprites, and the largest single sprite is 1.3 MB.

### 6.1 Pay the warm-up per frame, not per match

`App::warm_step` + `Gfx::prefetch_step`: a load step of its own that copies at most `WARM_BUDGET` = 512 KiB per call. Behind the VS screen it is one chunk per frame (the splash lasts at least 1.8 s), and the blocking `start_fight` path (rematch from the pause menu or the results screen, netplay start, the scripted tours) now yields a frame between chunks instead of freezing.

Measured on the same machine, same tour, same fighters, with `UME_WATCHDOG=1`:

| | Damaging frame |
|---|---|
| before | `212.8 ms in 'fight:prefetch'` (one frame) |
| after | `2.3 / 2.4 / 2.5 / 2.1 / 8.8 ms in 'load:sprites'` (one chunk per frame) |

A rematch with the same roster no longer flushes the texture cache at all (see 6.3), so its warm-up copies nothing.

### 6.2 Palette rows are shared by content

`ume_render::PaletteRows` keeps one row per *distinct* 256-colour palette instead of one row per palette object: a mirror match, a `RemapPal` that lands on an existing palette, and a re-warm after an atlas reset all reuse the row and skip the upload. Unit-tested in `crates/render/src/palette.rs` (50 lookups of the same colours → one row; a full atlas reports failure instead of overwriting a row in use).

### 6.3 A rematch keeps the atlases it already filled

The warm-up keys on the roster (`def.id` of both fighters). If the key matches the last warm-up, `Gfx::trim` is skipped, so the atlas generation does not change and `prefetch_step` finds every sprite already resident: the second fight of a marathon starts with zero uploads.

### 6.4 A draw-call budget that fails the build

`ume_render::DRAW_CALL_BUDGET = 24` with `batch::tests::a_fight_screen_stays_within_the_draw_call_budget`: a synthetic fight frame (stage layers, a subtractive reflection pass, two fighters, additive hit sparks, a two-region HUD overlay) must stay under the ceiling, and `a_flat_run_of_quads_is_one_call` pins the 300-quads-one-call property. Break the batching and CI says so.

### 6.5 Hitch labels that point at the right phase

`ume_shell::watchdog::label` now sticks to the first phase named in a frame. Before, the trailing `label("import")` at the end of `App::frame` overwrote everything, so a slow loading step was reported as an import; that is why the `load:sprites` numbers above are visible at all. Unit-tested.

### 6.6 What the numbers also told us

`prefetch_step` logs where it stops: for a native (RGBA, high-resolution) fighter the 50 % atlas cap fits **8 of 116 sprites**, after which the warm-up gives up and the rest stream lazily during the fight — one upload at draw time, which is what the browser port does for every sprite. For indexed DBZ art the same cap fits a few hundred. So the bulk warm-up is bounded by the atlas, not by the roster, and the next step (backlog 5) is to give the *lazy* path a per-frame upload budget instead of trying to pre-load more.

## 7. Backlog, in priority order

| # | Idea from the port | State here | How to verify |
|---|---|---|---|
| 1 | Never pay a whole match inside one frame | done: `App::warm_step` + `Gfx::prefetch_step`, 512 KiB per frame, VS step and `start_fight` both spread | `UME_WATCHDOG=1` label `load:sprites`; F3 `up`/`KB` |
| 2 | Keep the transition frame free in the paths with no VS screen | done: `start_fight` yields a frame per chunk | `UME_WATCHDOG=1` worst frame at the transition |
| 3 | Draw-call budget is a first-class number | done: `DRAW_CALL_BUDGET` + batch test | `cargo test -p ume-render` |
| 4 | Palette rows as a scarce resource | done for duplicates: `PaletteRows` shares rows by content | `cargo test -p ume-render` |
| 5 | Sprites are never atlased on the web build | **not copied** (our atlas is why we draw in a handful of calls). Follow-up: per-frame upload budget in the lazy path, because a 50 % cap only holds 8 native sprites | `atlas {}` fill in the F3 overlay |
| 6 | One shader for every sprite effect | already true (`sprite.wgsl`: flat/RGBA/indexed/palFX/tint in one pass) | `cargo test -p ume-render --test shader` |
| 7 | Yield while decoding | `load:character` already runs one character per step; keep new decoders inside that budget | `UME_WATCHDOG=1` worst frame during load |

## 7b. Are we better than them?

On the things this investigation measured, yes — and on the things it did not measure, no. Being precise matters more than the headline.

Better:

| | Browser port | Ultimate MUGEN |
|---|---|---|
| Draw calls for the same kind of scene | one per sprite, 82–105 per frame | merged by blend + scissor, budget 24, test-enforced |
| Palette changes | 1 KB upload per sprite that changes palette | palette rows deduplicated by content, shared across fighters |
| Simulation cost | whole state machine inside one WASM thread | 4–20 µs per tick (0.03–0.12 % of a frame) |
| Warm-up / transition | uploads spread by luck (per-sprite textures) | explicitly budgeted per frame, rematch copies nothing |
| Hitch observability | none | `UME_WATCHDOG=1` with per-phase labels, F3 with uploads/KB and atlas fill |
| Content licensing/portability | bundles third-party M.U.G.E.N characters | loads the player's own `mugen/` folder, UMF, or the marketplace |

Not better (yet):

| | Browser port | Ultimate MUGEN |
|---|---|---|
| Shipped in a browser | yes, 78 fighters, 74 stages, netplay via GGPO rollback | wasm build exists but the roster ships as packs; no equivalent catalogue |
| Content volume | 78 fighters, 74 stages, per-match lazy packs | whatever the player installs |
| Cross-device reach | phones, tablets, desktop | desktop, Android build, browser build |

So: a better *engine* on the measured axes, and a smaller *product* than theirs. The PR that follows implements every item of section 7 that was still open.

## 8. Reproducing

```console
# artifacts, request order, sizes, frame list
NODE_PATH="$(npm root -g)" node docs/research/ikemen-web/capture.js capture

# what the engine module needs, exports, and which Go files it was built from
NODE_PATH="$(npm root -g)" node docs/research/ikemen-web/wasm-artifacts.js capture/runtime-engine.bin

# per-frame WebGL counters from inside frame.html
NODE_PATH="$(npm root -g)" node docs/research/ikemen-web/frame-stats.js
```

`capture/` then holds the gzipped engine, the UI bundles, the catalog and the content packs used above. Playwright and Chromium are required; nothing is pulled from a third-party package registry beyond them.

Two measurement caveats: the numbers come from **headless Chromium with SwiftShader** (a software rasterizer), so treat the frame times as a floor, not a GPU result; and the 37 s select→fight time is dominated by the download of ~50 MB of content in that sandbox, not by the engine's own parsing.
