# Story playthrough decisions

Browser evidence captured from the current story scenes on base `589f657` with only the E2E cutscene initializer corrected so the web build could run. The intervening `master` update changes formats code and does not alter these story flows. The 272-second run completed all five waves, spared the boss, recruited Shion, won the final fight, played the epilogue, and returned to the main menu. The runner reported no page errors or failed requests.

`timeline.log` and `events.json` record the decisions and transitions. Screenshots capture the optional helper choice, recruited helper, final boss, epilogue, and completed return to menu.

Reproduce with `STORY_STAGE=4 STORY_FINAL_STAGE=4 STORY_LIMIT_MIN=8 STORY_MERCY=spare STORY_HELPER=choose STORY_OUT=<output-dir> node scripts/web-story-playthrough.js web/dist`.

