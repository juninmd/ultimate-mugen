#!/usr/bin/env bash
# Build the browser/PWA version into web/dist (serve it with any static HTTP server).
set -e
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown
mkdir -p web/dist
wasm-bindgen --target web --no-typescript --out-dir web/dist target/wasm32-unknown-unknown/release/ultimate-mugen.wasm
cp web/index.html web/gamepad.js web/manifest.webmanifest web/icon.svg web/sw.js web/dist/
mkdir -p web/dist/assets/mugen && cp assets/mugen/community.zip web/dist/assets/mugen/
echo "OK: web/dist  (ex.: python3 -m http.server -d web/dist 8080)"
