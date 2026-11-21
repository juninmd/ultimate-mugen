# Smoke test (debug mode)

One in-engine runner checks every main feature on every platform and leaves evidence: a screenshot per step and a report that says which feature broke and why.

## Run it

| Target | Command | Output |
|---|---|---|
| Desktop (Linux, macOS, Windows) | `ultimate-mugen --smoke <dir>` (Linux headless: `xvfb-run -a ultimate-mugen --smoke <dir> --res 1280x720`) | `<dir>/report.json`, `<dir>/report.md`, `<dir>/shots/*.png` |
| Web | `node scripts/smoke-web.js web/dist <dir>` | same files |
| Web mobile | `SMOKE_MOBILE=1 node scripts/smoke-web.js web/dist <dir>` (Pixel 7 viewport, touch) | same files |

The process exits with 1 when any check fails. `UME_SMOKE_ONLY=boot,fight_1v1` (desktop env var, also honoured by the web driver) runs only those features; the web page accepts `?smoke=1&smoke_only=boot`. `UME_SMOKE_FRAME_BUDGET_MS` overrides the average frame budget (500 ms).

## What it covers

| Feature id | Checks |
|---|---|
| `boot` | fighters and stages registered, nothing quarantined, no import errors, title renders |
| `screens` | menu, settings, controls, MUGEN import, market, replays, player card and tuning render |
| `select` | fighter select (1P and 2P) and stage select render |
| `fight_1v1` | fight loads, round intro ends, combat deals damage, KO ends the match, results screen follows |
| `training` | training mode loads and the simulation advances |
| `team_5v5` | ten fighters load and render |
| `story` | story intro and the first wave load and render |

Every feature also checks the average frame time and every screenshot is checked for a blank or flat frame.

## Reading a report

`report.md` lists one row per feature. A failed feature expands to the failing checks, each linking its screenshot. A panic is written to the report with the feature that was running, so the failing area is known even when the game dies.

## Adding a check

Add a function in `src/smoke/features/` (new file when the module nears 250 lines), list it in `src/smoke/features.rs` and use `ctx.check(name, ok, detail)` or `ctx.shot(app, name, warm_frames)`. Drive the game through `App` state like the screenshot tours do; keep a feature under a few seconds.

## Not covered yet

Netplay, auto update, zip import and the on-screen touch controls are not exercised by the runner.

## Nightly sweeps

Full-roster simulations that take 10 to 30 s each (`native_roster_lab`, `native_air_abilities`, `native_match_invariants`, `native_input_moves`, `native_normal_reach`, `native_no_stuck_states`, `native_attack_keyframes`, `pixel_states_test` and the air-normals sweep) are marked `#[ignore = "nightly: full-roster sweep"]`. The PR gate skips them; `.github/workflows/nightly.yml` runs them every day (`cargo test --workspace --release -- --ignored`), and locally you can run them the same way.
