# Development Roadmap: Ultimate MUGEN

[Português (BR)](ROADMAP.pt-BR.md)

Architectural priorities, compatibility improvements and planned features for **Ultimate MUGEN**.

---

## 1. Rollback Netcode (Determinism and Online Multiplayer)

The engine runs a strictly deterministic simulation loop at a fixed 60 ticks per second, with a controlled PRNG and no garbage collection, which is the ideal foundation for rollback netcode.

- [ ] **Match state serialization (`MatchStateSnapshot`)**:
  - Compact, fast in-memory serialization of `Match`, `Fighter`, `Projectile` and `Explod` state.
  - State pool for restore/rollback of up to 10-15 frames without per-tick allocations.
- [ ] **Cross-platform transport**:
  - **Web**: deterministic peer-to-peer over WebRTC DataChannels.
  - **Native**: plain UDP or an open rollback library such as GGRS.
- [ ] **Input synchronization and prediction window**:
  - Local and remote input buffering with prediction and instant rewind and replay on mismatch.
  - UI indicators for ping, jitter and rollback frame count.

---

## 2. Replays and Tournaments

- [ ] **Deterministic replay recorder**: initial `seed`, fighter and palette IDs, chosen stage and a bitfield-packed input stream. A multi-minute replay takes only a few kilobytes.
- [ ] **Replay player**: normal and fast playback (2x, 4x), pause, frame stepping and takeover of a character at any point.

---

## 3. Extended M.U.G.E.N Compatibility

Classic characters such as Kung Fu Man run faithfully, but advanced Elecbyte features and Ikemen GO extensions remain to be integrated:

- [ ] **Additional and rare state controllers**: full `Zoom`, advanced `PalFX` with sinusoidal interpolation and color inversion, `RemapPal` and dynamic palette swaps.
- [ ] **Team modes**: Simul (2v2 on the same screen), Tag (dynamic swaps with assists) and Turns (KOF style).
- [ ] **Audio and media**: MP3/OGG stage music, intro video and per-character ending screens (`intro.storyboard` / `ending.storyboard`).

---

## 4. UMF Tools and Ecosystem

UMF removes the need for dozens of legacy tools such as Fighter Factory.

- [ ] **Web-based visual editor**: WebGL/WASM editor for skeletons, poses and animation timing, with instant hitbox and hurtbox preview and direct `.json` import and export.
- [ ] **Particle effects and shaders**: declarative JSON particle effects (lightning, fire, shock waves) attached to skeleton limbs, plus custom shaders for auras, dynamic shadows and heat distortion.

---

## 5. New Game Modes and User Experience

- [ ] **Survival mode**: consecutive waves of fighters with persistent life or partial recovery per round.
- [ ] **Boss Rush and combo trials**: practice specific combos with real-time input and timing feedback.
- [ ] **Localization and accessibility**:
  - In-game language selection (English default, Portuguese, Spanish, Japanese) for all UI and menus. Documentation already has English and Portuguese.
  - In-game button remapping persisted in `LocalStorage` (web) or a local config file (desktop).

---

## 6. Performance and Memory

- [ ] **Asynchronous SFF sprite streaming and cache**: load non-essential sprite groups on demand to cut the initial memory peak for characters with SFFs above 100 MB.
- [ ] **Hardware-accelerated rasterization**: SIMD/AVX2 in the tiny-skia pipeline and direct GPU texture conversion.

---

## 7. Architecture Decisions and Phases (Nov 2026)

### Decisions

| Topic | Decision | Reason |
|---|---|---|
| Renderer | `wgpu` + `winit`, WGSL shaders, palette applied in the shader, atlas batching | One codebase for Vulkan, Metal, DX12, WebGPU and WebGL2 |
| Fighter scripting | CNS + ZSS (compatibility) and UMF JSON, compiled to the same bytecode | No Lua: smaller binary, simple WASM, determinism for rollback |
| Menus and modes | Rust + data (M.U.G.E.N screenpack or TOML) | Typed and testable, no Lua layer |
| Expressions | Stack bytecode with constants folded at load | No AST walk every tick |
| Audio | `kira` + `symphonia` (OGG, MP3, FLAC, WAV) | Same behavior in browser and native |
| Netplay | `ggrs` + `matchbox` (WebRTC) | Netplay in the browser and on phones too |
| Code style | No code comments; documentation lives in the wiki (`docs/wiki`) | Less text to keep in sync |
| Language | English by default, Portuguese translations alongside | Wider contributor base without losing the Portuguese-speaking community |

### Phases

| Phase | Deliverable | Acceptance |
|---|---|---|
| 0 | Conformance suite, logging of unknown parameters, syntax tolerance, `mugenversion` | Controller and trigger coverage report in `cargo test` |
| 1 | M.U.G.E.N 1.1 parity: missing controllers, full HitDef, win triggers, projectiles, helpers; AAA tweaks only for UMF | ≥ 95% per-tick match with M.U.G.E.N 1.1 recordings |
| 2 | Migration to `wgpu`; palettes, PalFX, AfterImage, `[BGCtrl]`, shadows, reflections, explods | Visual capture tests |
| 3 | `.fnt`, `system.def`, `select.def`, storyboards, `fight.def`, audio with loop and pan | Popular screenpacks open without errors |
| 4 | `Match` snapshot, rollback, CI sync-test, replays on disk | 50 fights at 100 ms without desync |
| 5 | Extensions: Map, RedLife, Dizzy, GuardBreak, Tag, Modify*, Clsn, Text, ZSS | Popular extended characters load |
| 6 | Bytecode, atlas, zero per-tick allocation, async loading | 10-fighter tick under 1 ms |
