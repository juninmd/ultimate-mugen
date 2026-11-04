# Ultimate MUGEN

**English** · [Português (BR)](README.pt-BR.md)

A 2D fighting game engine **built from scratch in Rust**, inspired by M.U.G.E.N: modern, deterministic and cross-platform (Windows, Linux, macOS, **browser/WASM**, **Android/iOS**), **compatible with classic M.U.G.E.N characters and stages** and with the new native **UMF (Ultimate Mugen Fighter)** format, which is declarative and much simpler. It has native support for **DualShock 4 / DualSense with haptic feedback and colored lightbars**.

![Fight](screenshots/06-fight-dojo.jpg)

| | |
|---|---|
| ![Super](screenshots/08-fight-super.jpg) | ![Select](screenshots/03-select.jpg) |
| ![MUGEN KFM vs original](screenshots/12-mugen-kfm-vs-native.jpg) | ![MUGEN stage](screenshots/13-native-on-mugen-stage.jpg) |
| ![Kaname vs Valdric](screenshots/11b-fight-kaname-valdric.jpg) | ![Mobile](screenshots/15-mobile-fight.jpg) |
| ![Select with many fighters](screenshots/03b-select-many.jpg) | ![Loading on VS](screenshots/05a-vs-loading.jpg) |
| ![Video](screenshots/17-video.jpg) | ![4K](screenshots/19-fight-4k.jpg) |
| ![Cinematic KO](screenshots/20-fx-ko.jpg) | ![Super](screenshots/08-fight-super.jpg) |
| ![Neon impact](screenshots/21-fx-neon-hit.jpg) | ![Transition](screenshots/22-transition.jpg) |
| ![Team sequence](screenshots/30-team-sequence.jpg) | ![Team parallel](screenshots/31-team-parallel.jpg) |
| ![Team select](screenshots/32-team-select.jpg) | ![Impact frame](screenshots/23-impact-frame.jpg) |
| ![5v5 parallel](screenshots/33-team-5v5-parallel.jpg) | ![5v5 sequence](screenshots/34-team-5v5-sequence.jpg) |
| ![4K 5v5 HUD](screenshots/35-4k-5v5-hud.jpg) | ![4K super](screenshots/36-4k-super.jpg) |

---

## Contents
- [Why Rust + wgpu?](#why-rust--wgpu)
- [Highlights](#highlights)
- [Prerequisites](#prerequisites)
- [Build and Run](#build-and-run)
- [Controls](#controls)
- [Crate Architecture](#crate-architecture)
- [Asset Formats](#asset-formats)
- [Contributing](#contributing)
- [Releases and Auto-Update](#releases-and-auto-update)
- [Further Documentation](#further-documentation)
- [Acknowledgments](#acknowledgments)
- [Legal Notice](#legal-notice)

---

## Why Rust + wgpu?

| Requirement | Technical choice |
|---|---|
| **Single codebase** (desktop, web, mobile) | **wgpu + winit** (Vulkan, Metal, DX12, GLES, WebGL2; Windows, Linux, macOS, WASM, Android, iOS). iOS. |
| **Performance and determinism** (basis for rollback netcode and replays) | Rust with fixed-step simulation (60 ticks/s), deterministic PRNG and no garbage collector. |
| **Advanced controls and rumble** | **gilrs** for cross-platform input + direct HID reports for DualShock 4/DualSense (USB and Bluetooth, CRC32 checksum) + HTML5 Gamepad API with `vibrationActuator` on the web. |
| **Crisp procedural art** | **tiny-skia** (antialiased vector rasterization with gradients) for procedural skeletons at any scale. |
| **Lightweight binary** | Compact WASM (~3 MB uncompressed), single native binary, no heavy external runtimes. |

---

## Highlights
- **True 4K**: fullscreen (F11), windows up to 5K, and procedural art (UMF fighters and stages) rasterized at a density proportional to the screen. *Automatic*: 3× up to 900 px tall, 4× up to 1300, 4.5× up to 1700, **6× at 4K** (**Video** menu; flags `--res 3840x2160`, `--4k`, `--fullscreen`).
- **Many characters and stages**: scrolling select screen, filter (Tab: all/originals/M.U.G.E.N), PgUp/PgDn, mouse wheel, `R` for random. Only visible rows are drawn and only the item under the cursor is loaded (LRU cache bounds memory). Imported folders/zips are registered without decoding everything (hundreds in seconds).
- **Fast loading**: startup loading screen, staged preloading behind the VS screen (progress bar), lightweight previews (idle/victory) in menus, parallel fighter rasterization (≈3× faster) and synthesized music before the fight.
- **AAA visuals** ([`docs/AAA_FEATURES.md`](docs/AAA_FEATURES.md)): post-processing (bloom, chromatic aberration, radial blur, vignette, color grade), particles and shock rings on every hit, damage numbers, camera punch, slow-motion zoom KO, cinematic supers with speed lines and aura, floor reflections, animated HUD, screen transitions and living backgrounds. Accessibility option to reduce effects (Video → Impact effects).
- **Story and cinematics**: face-off with speech bubbles at round 1, Arcade cutscenes (opening, confrontation, ending), **transformation** (`T`), varied defeat animations, bigger supers, **zoom-out** camera for large teams and the **final boss Mikazuki Oboro** (Itachi × Aizen) at the end of Arcade.
- **Team modes**: *Sequence 3v3* (KOF style: entry order, winner keeps life, shared super meter) and *Parallel up to **5v5*** (10 fighters on screen; AI partners, `Tab` switches the controlled fighter; the round ends when a whole team falls).
- **Layered audio**: synthesized SFX with transient + body + crunch, 3 variations per hit, reverb, round/KO/victory stingers, combo pings and music *ducking*. **HUD** with real portraits, partner bars, team roster and alarms.
- **Replay** of the last fight (Results → Replay): the engine is deterministic and the replay re-simulates the recorded inputs, with a test.
- **Engine/gameplay**: combo damage scaling (native fighters), corner push, squash & stretch; **HUD v3** and effects built from polygons (per-vertex gradients, bevels, hexagons).

- **Complete M.U.G.E.N compatibility** (`crates/formats` + `crates/engine`):
  - Supports `.def`, `.sff` v1 and v2 (PCX 8-bit, RLE8, RLE5, LZ5, PNG8/24/32), `.air`, `.cmd`, `.cns`, `.snd` and `.act`.
  - Stages with multiple layers, parallax, tile repetition and dedicated animations.
  - M.U.G.E.N expression interpreter and evaluator with static typing (integer vs float division, `[a, b]` ranges, `p2`, `parent`, `root`, `helper` redirections, etc.).
  - 90+ state controllers, full `HitDef` system, throws (custom states), helpers, projectiles, explods, super pause, after images and HD `localcoord` scaling.
  - Tested and validated against reference characters such as Kung Fu Man (SD and 720p).
- **Native UMF format**:
  - Each fighter is defined in a single declarative JSON file.
  - Hitboxes and hurtboxes are generated from the skeleton and active parts (`hit: { part: "handR" }`).
  - Compiled at startup to the same state machine as the M.U.G.E.N VM, so UMF and M.U.G.E.N fighters fight under identical rules.
- **Roster and content**:
  - **15 original procedural fighters** + **10 compatible classic M.U.G.E.N characters** (Kung Fu Man SD/HD, Suave Dude and free community characters).
  - **14 stages** (6 original procedural + 8 M.U.G.E.N).
  - Synthesized procedural audio and embedded WAV effects.
- **Game modes**: Arcade, Versus CPU, 2 Players (local, same keyboard or gamepads), Training (collision box view) and CPU vs CPU. A universal AI can fight with any roster character.

---

## Prerequisites

1. **Rust toolchain**: stable (1.75+ recommended).
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **System dependencies (Linux / Debian / Ubuntu)**:
   ```bash
   sudo apt-get update && sudo apt-get install -y \
       pkg-config libudev-dev libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev
   ```
   On macOS and Windows, Cargo uses the native system APIs (Cocoa/CoreAudio and Win32/WASAPI) with no extra packages.
3. **WebAssembly target (browser)**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

---

## Build and Run

### Native (desktop)

```bash
cargo run --release
cargo test --workspace --release
```

Special launch modes:
```bash
cargo run --release -- --touch          # Force on-screen virtual touch controls
cargo run --release -- --mute           # Start without audio
cargo run --release -- --shots out/     # Automated tour that saves screenshots to out/
cargo run --release -- --4k             # 3840×2160 window (or --res 2560x1440, --fullscreen)
cargo run --release -p ume-engine --example load_bench   # Fighter load time (full vs. preview)
node scripts/web-smoke.js web/dist      # Web smoke test (headless Chromium; needs the playwright package)
```

### Web / PWA (WebAssembly)

`scripts/build-web.sh` builds the WASM binary and bundles the static files (HTML, assets, JS bridges and PWA manifest) into `web/dist`:

```bash
./scripts/build-web.sh
python3 -m http.server -d web/dist 8080
```
Open `http://localhost:8080`. On the web you can drag character `.zip` files in or click **"Import MUGEN (.zip)"**.

### Mobile (Android & iOS)

- **Android**:
  ```bash
  cargo install cargo-apk --version 0.9.6 --locked
  cargo apk build -p ultimate-mugen --lib --release --target aarch64-linux-android
  ```
- **iOS**: `cargo install cargo-bundle`, then `cargo bundle --release --target aarch64-apple-ios` on macOS with Xcode (sign the `.app` with your team in Xcode or `codesign`).

---

## Controls
Keyboard, multiple gamepads and touch work at the same time.

| Action | Keyboard (P1) | Keyboard (P2 / local versus) | DualShock 4 / DualSense / XInput | Touch (mobile / web) |
|---|---|---|---|---|
| **Move / Jump / Crouch** | `W` `A` `S` `D` or arrows | Arrow keys | D-Pad or left stick | Virtual stick |
| **Light Punch (LP)** | `U` (or `V`) | Numpad `4` | Square (`□`) / `X` (Xbox) | **LP** button |
| **Heavy Punch (HP)** | `I` (or `B`) | Numpad `5` | Triangle (`△`) / `Y` (Xbox) | **HP** button |
| **Extra Punch (Z)** | `O` (or `N`) | Numpad `6` | `R1` / `RB` | — |
| **Light Kick (LK)** | `J` (or `Z`) | Numpad `1` | Cross (`✕`) / `A` (Xbox) | **LK** button |
| **Heavy Kick (HK)** | `K` (or `X`) | Numpad `2` | Circle (`○`) / `B` (Xbox) | **HK** button |
| **Extra Kick (C)** | `L` (or `C`) | Numpad `3` | `R2` / `RT` | — |
| **Quick specials** | `1`, `2`, `3` | Numpad `7`, `8`, `9` | `L1`, `L2` | **SP1**, **SP2**, **SP3** |
| **Super** | `1` + `2` | Numpad `7` + `8` | `L1` + `L2` | **SUPER** button |
| **Awakening (transformation)** | `T` | Numpad `0` | Right stick click / `SELECT` | — |
| **Pause / Hitboxes** | `Esc` / `F2` | `Esc` / `F2` | `OPTIONS` / `START` | `❚❚` button (top corner) |
| **Fullscreen / FPS** | `F11` / `F3` | — | — | — |
| **Switch fighter (parallel team)** | `Tab` | — | — | — |
| **Fighter list** | `Tab` (filter), `PgUp`/`PgDn`, `R` (random) | — | — | — |

**Menus**: navigate with arrows / `WASD` / D-Pad; confirm with `Enter`, `Space`, `J`, `U` or `✕`/`A`; go back with `Esc`, `Backspace`, `K` or `○`/`B`; toggle hitboxes in training with `F2`.

**Haptics and lights (DualShock 4 / DualSense)**: native USB and Bluetooth HID reports; proportional rumble for light and heavy hits, blocks, Super and KO; lightbar blue for player 1, red for player 2, flashing white during Super.

---

## Crate Architecture

A Cargo workspace with strict separation between pure parsing, the deterministic engine and the presentation layer:

```
ultimate-mugen/
├── crates/
│   ├── formats/      # Pure M.U.G.E.N format parsers and decoders (no display dependencies)
│   ├── engine/       # Deterministic fight VM, physics, collisions, AI and UMF compiler
│   ├── render/       # wgpu renderer: atlas, batched sprites with palettes in WGSL, post-processing
│   └── shell/        # winit window, input, text, audio and the immediate-mode API used by the app
├── src/              # Application layer (screens, rendering, audio, HID/gamepads, HUD); wgpu + winit on every platform
├── assets/           # OFL fonts, UMF definitions (.json) and asset packs
├── docs/             # Detailed technical documentation, roadmap and research
└── web/              # HTML shell, PWA, service worker and JS glue code
```

- **`crates/formats` (`ume-formats`)**: `sff.rs` (SFF v1/v2), `air.rs` (animations and `Clsn1`/`Clsn2` boxes), `ini.rs` (tolerant M.U.G.E.N INI), `snd.rs` and `act.rs` (sounds and palettes), `stage.rs` (stages), `vfs.rs` (case-insensitive in-memory virtual file system over folders and ZIPs).
- **`crates/engine` (`ume-engine`)**: `expr.rs` (expression AST, parser and evaluator), `chardef.rs` and `cns.rs` (fighter data and states), `commands.rs` and `input.rs` (input buffers and command detection), `fighter.rs` (live fighter/helper state), `matchs.rs` (60 Hz match simulation), `hitdef.rs`, `controllers.rs` (90+ state controllers), `native/` (UMF compiler and procedural bone rasterizer via `tiny-skia`), `ai.rs`.
- **`src/` (`ultimate-mugen` binary)**: `app.rs` (screen lifecycle), `render.rs` (2D pipeline with additive blending, shadows, flash/pause effects), `gamepad.rs` (gilrs + raw HID with CRC32 for DualShock 4/DualSense), `controls.rs` (keyboard, joystick and virtual stick), `audio.rs` (procedural audio synthesizer and WAV playback).

---

## Asset Formats

### 1. Classic M.U.G.E.N
- **Desktop**: put the character folder or `.zip` in `mugen/chars/` (for example `mugen/chars/kfm/kfm.def`) or set the `UME_MUGEN_DIR` environment variable.
- **Stages**: put the `.def` and `.sff` files in `mugen/stages/`.
- **Web**: drag a `.zip` with the character or stage into the browser window.

### 2. Native UMF (JSON)
UMF fighters live in `assets/chars/<name>.json`. Instead of hundreds of hand-made sprites and complex text files, a fighter is described declaratively: metadata, base stats, hex color palettes, and moves and animations. The engine generates the `Clsn1` and `Clsn2` boxes from the skeleton limbs. See the full guide in [`docs/CHARACTER_FORMAT.md`](docs/CHARACTER_FORMAT.md) and the [wiki](docs/wiki/Home.md).

### 3. Audio and Fonts
- **Fonts**: open-source vector typefaces *Russo One* and *Rajdhani* under the SIL Open Font License (OFL), embedded in `assets/fonts/`.
- **Sound effects**: real-time synthesized effects and stereo/mono PCM `.wav` files (8-bit and 16-bit) inside `.snd` containers.

---

## Contributing

Contributions are very welcome. To keep the project stable:

1. **Tests**: `cargo test --workspace`
2. **WebAssembly compatibility**: `cargo check --target wasm32-unknown-unknown`
3. **Formatting and lints**: `cargo fmt --check` and `cargo clippy --workspace --tests`
4. **New fighters**: create `<name>.json` in `assets/chars/`, add it to the roster list in `src/registry.rs` and to the smoke test in `crates/engine/tests/roster.rs`.
5. **Conventional commits** (they drive releases): `feat: …`, `fix: …`, `perf: …`; `feat!:` or a `BREAKING CHANGE:` footer for breaking changes. `docs`, `chore`, `ci`, `test`, `refactor`, `style` and `build` do not trigger a release. The PR title (squash) **or** all commits must follow the format; the *Conventional commits* check validates it.
6. **Language**: code, commits, issues and documentation are written in English. Portuguese translations live in `*.pt-BR.md` files and `docs/wiki/pt-BR/`.

---

## Releases and Auto-Update

The **git tag is the source of truth for the version** (`version` in `Cargo.toml` is only the base before the first tag). On every push to `master`, the `Release` workflow (`.github/workflows/release.yml`):

1. **Plans** (`scripts/release-plan.mjs`): reads commits since the last `vX.Y.Z` tag. `feat` → minor, `fix`/`perf` → patch, `!`/`BREAKING CHANGE` → major (before 1.0, major becomes minor; 1.0.0 is an explicit decision). Only `docs/chore/ci/…` means no release.
2. **Builds** on Linux, Windows and macOS (`cargo build --release --locked`) with the version stamped through `UME_RELEASE_VERSION`, and checks `ultimate-mugen --version`.
3. **Publishes** as a *draft*: uploads the packages + `latest.json` (URLs and SHA-256) + `SHA256SUMS.txt`, **verifies** the draft against the manifest and only then publishes, which creates the `vX.Y.Z` tag on the planned commit. Re-running resumes the draft (idempotent); a published tag is never overwritten.
4. **Health check**: confirms `releases/latest/download/latest.json` already exposes the new version.

First release (old history without semantic commits) or forced release: *Actions → Release → Run workflow* and choose `bump`. **Rollback:** `gh release delete vX.Y.Z --cleanup-tag --yes`.

**In game (desktop):** on startup a background check reads the public `latest.json` (HTTPS only, 8 s timeout, silent when offline) and shows a notice on the title screen when a newer version exists. It is **notification only**: nothing is downloaded or run automatically; the package comes from the releases page and can be verified with the manifest SHA-256. Disable with `--no-update-check` or `UME_NO_UPDATE_CHECK=1`; local builds (`…-dev`) do not check. Headless diagnostics: `ultimate-mugen --version` and `ultimate-mugen --check-update`.

> Automatic installation would require signing the packages (minisign/certificate) and embedding the public key in the game; today the manifest guarantees integrity, not authorship.

---

## Further Documentation

- **[Wiki](docs/wiki/Home.md)**: install, configure, create UMF characters, CNS scripts, stages and customize the engine.
- [`docs/PESQUISA.md`](docs/PESQUISA.md): in-depth research on reverse engineering M.U.G.E.N and architectural challenges (Portuguese).
- [`docs/CHARACTER_FORMAT.md`](docs/CHARACTER_FORMAT.md): native UMF format manual (Portuguese).
- [`docs/ROSTER.md`](docs/ROSTER.md): roster guide and catalog of compatible characters (Portuguese).
- [`docs/ROADMAP.md`](docs/ROADMAP.md): development roadmap.
- [`docs/research-mugen-en.md`](docs/research-mugen-en.md): complete technical report on binary format specifications.

---

## Acknowledgments

Thanks to the [Ikemen GO](https://github.com/ikemen-engine/Ikemen-GO) team and to Elecbyte, creator of M.U.G.E.N, for the inspiration.

## Legal Notice

Copyrighted characters owned by third parties (franchises such as Dragon Ball, Naruto, Street Fighter, etc.) are **not included or distributed** with this engine's executable. Ultimate MUGEN is an open-source game engine interpreter able to load and render M.U.G.E.N content imported by users themselves.
