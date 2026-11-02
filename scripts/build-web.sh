#!/usr/bin/env bash
# Build the browser/PWA version into web/dist (serve it with any static HTTP server).
set -e
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown
mkdir -p web/dist
cp target/wasm32-unknown-unknown/release/ultimate-mugen.wasm web/dist/
cp web/index.html web/gamepad.js web/mq_js_bundle.js web/manifest.webmanifest web/icon.svg web/sw.js web/dist/
echo "OK: web/dist  (ex.: python3 -m http.server -d web/dist 8080)"
