#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
bindgen_version="$(awk '/name = "wasm-bindgen"/{getline; gsub(/[^0-9.]/,""); print; exit}' "$root/Cargo.lock")"

if [ ! -w "${RUSTUP_HOME:-/nonexistent}" ] && [ -d /usr/local/rustup ]; then
  export RUSTUP_HOME="$HOME/.rustup"
  [ -d "$RUSTUP_HOME" ] || cp -r /usr/local/rustup "$RUSTUP_HOME"
fi
export PATH="$HOME/.cargo/bin:$PATH"

rustup target add wasm32-unknown-unknown
if ! command -v wasm-bindgen >/dev/null || [ "$(wasm-bindgen --version | awk '{print $2}')" != "$bindgen_version" ]; then
  cargo install wasm-bindgen-cli --locked --version "$bindgen_version"
fi

npm install --no-save --no-package-lock --prefix "$root" playwright@1.56.1
npx --prefix "$root" playwright install chromium

echo "export RUSTUP_HOME=$RUSTUP_HOME PATH=$HOME/.cargo/bin:\$PATH"
echo "then: ./scripts/build-web.sh && NODE_PATH=$root/node_modules node scripts/web-smoke.js web/dist"
