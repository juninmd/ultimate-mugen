# E2E ArcadeRun initializer

The current `master` web build failed because the cutscene preview hook assigned the removed `(ladder, index)` tuple to `App::arcade`, which now stores `Option<ArcadeRun>`. Constructing the run through the existing `ArcadeRun::at` constructor restores the hook's intended one-fight boss preview.

Validation on this branch:

- `scripts/build-web.sh` — passed.
- `node scripts/web-smoke.js web/dist` — `WEB SMOKE OK`.
- Full fifth-stage browser playthrough with mercy and helper choices — 272 seconds, returned to menu, no page errors or failed requests.

The story screenshots show that the compiled web app remains playable through the final boss, epilogue and return to menu after the type fix.
