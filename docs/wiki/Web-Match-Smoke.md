# Web Match Smoke

`node scripts/web-match-smoke.js web/dist` plays complete matches in headless Chromium (CI job `web-smoke`, after the web build) and fails when a match freezes.

Each scenario opens a deep link with `?e2e=1`, which publishes `window.__ume_state` every frame (screen, `clock.tick`, round, wins, fighters). Settings are injected through `localStorage` (30 s rounds, 1.15x speed, muted).

| Scenario | What it covers |
|---|---|
| `cpu-bot-best-of-3` | A scripted player closes in and mashes attacks against the CPU for at least two rounds |
| `watch-cpu-vs-cpu` | No input: the match must still run to the results screen |

A scenario fails on any of:

- page error, Rust panic, `RuntimeError` or an unexpected `console.error`;
- the simulation tick not advancing for `MATCH_SMOKE_STALL_MS` (default 20000 ms) outside pause and match end;
- no animation frame, or an unresponsive main thread, for the same period;
- the rendered image never changing during the fight;
- the results screen not being reached within `MATCH_SMOKE_LIMIT_MS` (default 360000 ms), or not being leavable with Enter.

`MATCH_SMOKE_SCENARIOS=<name,...>` runs a subset and `WEB_SMOKE_SHOTS=<dir>` stores screenshots (uploaded by CI on failure).
