#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
node scripts/wiki-book.mjs target/wiki-book/src
mkdir -p web/dist
mdbook build docs/book --dest-dir "$PWD/web/dist/wiki"
echo "OK: web/dist/wiki"
