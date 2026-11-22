# Web QA pipeline

Every pull request plays the real wasm build in headless Chromium, in parallel shards, and fails on any WebAssembly error. A failure leaves screenshots and logs as CI artifacts.

## What runs

`scripts/web-qa.json` lists the scenarios; `scripts/web-qa.js` runs one shard of them. CI job `web-qa` (matrix, `fail-fast: false`):

| Shard | Scenarios |
|---|---|
| `ui` | title to fight flow, Arcade click, story intro, epilogue |
| `mobile` | touch menu, touch controls in a fight, in-engine smoke on a Pixel 7 viewport |
| `engine` | in-engine smoke (`?smoke=1`): boot, menus, select, training, 5v5, story; plus the guard self-test |
| `loading` | M.U.G.E.N import (kfm), native fighters with awakenings, power arena, audio unlock, fatal-trap screen (`unreachable`) |
| `match-1`, `match-2` | full matches in two halves ([Web match smoke](Web-Match-Smoke.md)) |

## The guard

Every scenario runs with `scripts/web-guard.js` preloaded. It watches every page and fails the run, even when the script itself forgot to, on:

- `panicked`, `PanicHookInfo`, `unreachable executed` or `renderbufferStorageMultisample` in the console;
- `gpu error:` (the wgpu error handler in `ume-shell`);
- an uncaught `pageerror` or a Chromium crash.

On the first problems it saves `problem-N.png` immediately, and a `final-N.png` before the browser closes. It also writes `console.log` (every console line with the time and page) and `network.log` (failed requests and HTTP 400+). `fatal-trap` sets `expectFatal` because it throws a trap on purpose, so there the guard only records.

## Artifacts

Each shard uploads `web-qa-<shard>` (always, kept 7 days): `summary.md`, `summary.json` and one folder per scenario with `output.log`, `console.log`, `network.log`, `guard.json`, `shots/` and the `.png` files. The summary is also written to the job summary page with the failing problems and the last output lines.

## Run it locally

```
./scripts/build-web.sh
npm install --no-save playwright@1.56.1 && npx playwright install chromium
node scripts/web-qa.js --dist web/dist --out web-qa-report --shard ui
node scripts/web-qa.js --only fatal-trap,engine-smoke --jobs 2
```

Software GL is slow: keep `--jobs` at 1 or 2 locally. For a one-off page add `?debug=1` and read the console.

## Adding a scenario

1. Write `scripts/web-<name>-smoke.js` that takes the dist folder as its first argument and exits non-zero on failure.
2. Add it to `scripts/web-qa.json` with a shard, or list it under `excluded` with the reason (network, manual tool).

`scripts/web-qa.test.mjs` fails when a `web-*-smoke.js` is neither covered nor excluded, or when the shards in the manifest and in `ci.yml` differ.
