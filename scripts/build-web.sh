#!/usr/bin/env bash
# Build the browser/PWA version into web/dist (serve it with any static HTTP server).
set -e
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown
mkdir -p web/dist
wasm-bindgen --target web --no-typescript --out-dir web/dist target/wasm32-unknown-unknown/release/ultimate-mugen.wasm
cp web/gamepad.js web/market.js web/manifest.webmanifest web/icon.svg web/sw.js web/assets.js web/dist/
# the JS glue and the wasm are a matched pair: both are fetched as ?v=<build>, so a cache can never mix two builds
BUILD="${UME_BUILD_ID:-$(git rev-parse --short HEAD 2>/dev/null || echo dev)-$(date +%s)}"
sed "s/__BUILD__/$BUILD/g" web/index.html > web/dist/index.html
mkdir -p web/dist/assets/stages
cp assets/stages/*.png web/dist/assets/stages/
mkdir -p web/dist/assets/stages/story
cp assets/stages/story/*.png web/dist/assets/stages/story/
cargo run --release -p ume-devtools --bin ume-tools -- pack-sprites web/dist
echo "OK: web/dist  (ex.: python3 -m http.server -d web/dist 8080)"
