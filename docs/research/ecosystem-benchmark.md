# Ultimate MUGEN — Engine Baseline & Third-Party Ecosystem Benchmark

Date: `2026-11-20` · Platform: Windows `x86_64` · Profile: `--release`

## 1. Live Engine Baseline Benchmarks

### Zero-Allocation Tick Gate (`tick_alloc`)
- `cargo test -p ume-engine --release --test tick_alloc`: **PASS** (`2 passed, 0 failed`, `0.05s`)
- Confirms `Match::tick` performs **zero heap allocations** and maintains deterministic state for `ggrs` rollback and replays.

### Simulation Throughput (`native_tick_bench`)
- Command: `cargo run --release -p ume-engine --example native_tick_bench -- assets/chars`
- Result: **21 fighters · 252,000 ticks · 62,280 ticks/s**
- Mean tick time: **0.0161 ms** (1,038× faster than the 16.67 ms 60 Hz frame budget)
- Worst tick time: **5.310 ms**

### Fighter Load & Sprite Rasterization (`load_bench`)

| Fighter | Full Load (ms) | Sprites | Preview (ms) | Notes |
|---|---:|---:|---:|---|
| `jin` | 118 | 280 | 47 | Fastest full roster fighter |
| `haruki` | 119 | 270 | 36 | — |
| `garan` | 137 | 278 | 43 | — |
| `bruno` | 141 | 278 | 36 | — |
| `akio` | 146 | 281 | 40 | — |
| `daichi` | 153 | 287 | 46 | — |
| `volta` | 153 | 291 | 42 | — |
| `marlow` | 155 | 284 | 40 | — |
| `lian` | 162 | 280 | 41 | — |
| `shion` | 166 | 295 | 47 | — |
| `valdric` | 173 | 277 | 55 | — |
| `kaname` | 185 | 311 | 45 | — |
| `suzume` | 191 | 416 | 63 | — |
| `npc_spitfire` | 197 | 144 | 11 | — |
| `sylene` | 204 | 270 | 59 | — |
| `kaito` | 212 | 422 | 52 | — |
| `oboro` | 281 | 559 | 68 | Largest sprite pack (boss) |
| `npc_razor` | 302 | 142 | 11 | **Anomaly: 2× slower per sprite** |
| `npc_phantom` | 436 | 140 | 11 | **Anomaly: 3× slower per sprite** |
| `npc_colossus` | 584 | 148 | 10 | **Anomaly: 4× slower per sprite** |
| `npc_brute` | 748 | 136 | 10 | **Anomaly: 5× slower per sprite** |
| **Total (21 fighters)** | **4,962 ms** | — | — | — |

> **Actionable Finding:** `npc_brute`, `npc_colossus`, `npc_phantom`, and `npc_razor` take 302–748 ms to load despite having only ~140 sprites (~half of regular fighters). Optimizing their PNG compression / dimensions in `assets/chars/sprites/npc_*/` can cut total roster load time by **~35% (~1.7 s)** immediately.

---

## 2. Third-Party Rust Crates Evaluation

| Domain | Current Stack | Candidate Crate (Latest Stable) | WASM Safe | Determinism Impact | Verdict & Recommendation |
|---|---|---|---|---|---|
| **Online Netplay (Web)** | `ggrs 0.13.0` (desktop UDP only) | **`matchbox_socket` `0.4.0`** (WebRTC P2P data channels) | Yes (first-class) | None (transport only) | **ADOPT (P0)** — Connects existing `ggrs` rollback engine to browser PvP with zero install. |
| **UI Layout** | Manual virtual coordinates (`1280×720`) | **`taffy` `0.15.1`** (Flexbox / CSS Grid) | Yes (`no_std` capable) | None | **ADOPT SELECTIVELY (P1)** — Use for responsive Menus, Settings, Tower, and Select grids while keeping custom `ume_shell::quad` rendering. |
| **Debug / Devtools UI** | Custom immediate quads | **`egui` `0.37.0`** | Yes | None | **DEV-ONLY (P2)** — Too heavy/opinionated for player-facing fighting UI, but ideal behind a `dev-ui` feature flag for live hitbox/frame-data editors. |
| **Audio & Dynamic BGM** | `symphonia 0.5.5` + `quad-snd` | **`kira` `0.12.6`** | Partial (custom backend needed on web) | None | **EVALUATE (P2)** — Offers beat-synced transitions, low-HP stems, and smooth crossfades; keep `symphonia` on WASM unless `kira` web backend is unified. |
| **Campaign / Mode Scripting** | `toml 1.2.1` (declarative Story) | **`rhai` `1.23.4`** (pure Rust, sandboxed) | Yes (pure Rust) | Safe if outside `Match::tick` | **ADOPT FOR MODS (P1)** — Pure-Rust sandboxed scripting for custom Story cutscenes, Tower rules, and Arcade triggers without C FFI. |
| **Legacy IKEMEN Lua** | Custom ZSS parser (`crates/engine/src/zss`) | **`mlua` `0.12.x`** (Lua 5.4 / Luau) | Complex (C toolchain on WASM) | High risk inside tick | **DEFER (P3)** — Prefer declarative TOML + `rhai` for native modes; keep ZSS parser in pure Rust for WASM portability. |

---

## 3. Fighting-Game Ecosystem Tools (Import & Adaptation)

| Tool / Project | License / Ecosystem | What Can Be Imported or Adapted | Integration Path |
|---|---|---|---|
| **IKEMEN GO** | MIT (Go) | Frame-data HUD bar, ZSS state patterns, Tag/Simul turn conventions, Training dummy record/playback | Reference spec for `crates/engine/src/zss` and Training Mode frame-advantage bar. |
| **Fighter Factory Studio (FF3/FFS)** | Community standard IDE | `.def` / `.sff` / `.air` / `.cns` / `.snd` authoring workflow | Add `ume-devtools` CLI validator (`mugen_check --lint`) hints tailored to Fighter Factory exports. |
| **Aseprite JSON Export** | Ubiquitous pixel-art tool | Frame tags, durations, pivot points, and slices | Add an `ume-devtools import-aseprite` subcommand to emit UMF `meta.json` + animation tables directly. |

---

## 4. Recommended Implementation Order

1. **P0 — Browser Rollback Netplay (`matchbox_socket` + `ggrs`):** Unlocks cross-platform WebRTC PvP directly on `juninmd.github.io/ultimate-mugen/`.
2. **P0 — NPC Sprite Asset Optimization:** Investigate and re-pack `npc_brute`/`npc_colossus`/`npc_phantom`/`npc_razor` PNGs to save ~1.7 s on full load.
3. **P1 — Responsive Layout (`taffy`) for Settings/Tower/Select:** Eliminates manual coordinate math and adapts cleanly to mobile/ultrawide screens.
4. **P1 — Sandboxed Mod Scripting (`rhai` feature-gated):** Extends the new TOML Story & Daily Tower systems with scripted boss phases and custom triggers.
