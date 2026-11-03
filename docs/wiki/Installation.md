# Installation

## Download a ready build

Download the file for your platform from [Releases](https://github.com/juninmd/ultimate-mugen/releases), extract it and run `ultimate-mugen` (Windows: `ultimate-mugen.exe` or `run.bat`). The engine checks for updates by itself; turn that off with `--no-update-check` or `UME_NO_UPDATE_CHECK=1`.

## Build from source

Requires stable Rust ([rustup.rs](https://rustup.rs)).

Linux (Debian/Ubuntu):

```bash
sudo apt-get install -y pkg-config libudev-dev libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev
```

| Target | Command |
|---|---|
| Desktop | `cargo run --release` |
| Tests | `cargo test --workspace --release` |
| Web (PWA) | `rustup target add wasm32-unknown-unknown`, then `./scripts/build-web.sh` and `python3 -m http.server -d web/dist 8080` |
| Android | `cargo install cargo-quad-apk`, then `cargo quad-apk build --release` |
| iOS | `aarch64-apple-ios` target, guide at [macroquad.rs/articles/ios](https://macroquad.rs/articles/ios/) |

## Folder layout next to the executable

```
ultimate-mugen(.exe)
ultimate-mugen.cfg        settings saved by the menu
mugen/
  chars/<name>/<name>.def M.U.G.E.N characters
  stages/<name>.def       M.U.G.E.N stages
  <anything>.zip          zipped packs (characters or stages)
```

The `mugen/` folder is searched in this order: `UME_MUGEN_DIR`, the current directory, next to the executable, and two levels above it.
