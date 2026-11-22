# Combo Trials

Training mode shows a ladder of seven combo goals above the arena. They work for every fighter, native or M.U.G.E.N, because they only read the combo counters the engine already keeps.

| # | Goal |
|---|---|
| 1 | 3 hits |
| 2 | 5 hits |
| 3 | 8 hits |
| 4 | 25% of the rival's life in one combo |
| 5 | 6 or more hits finishing with a super (at least 1000 power spent) |
| 6 | 10 or more hits without spending any meter |
| 7 | 45% of the rival's life in one combo |

A combo is judged when it ends. Goals are cleared in order, a flash says TRIAL CLEAR and progress is saved per fighter in `trials.json` (next to the config, or in browser storage on the web build). Finishing the ladder shows ALL TRIALS CLEARED.

## How it works

- `src/trials.rs`: pure `Tracker`, fed once per frame with the player's `combo_count`, `combo_damage`, `power` and the rival's `life_max`. It tracks power at the start and the lowest power during the combo to know whether meter was spent.
- `src/app/trials_hud.rs`: reads the match after each tick (never changes it) and draws the panel; hidden while paused, in replays and outside training.
- `Tracker::demo` replaces the live sample and exists only for the screenshot tour: `UME_SHOTS_ONLY=trials`.
- Cost: one struct update per frame in training only; no allocation.
