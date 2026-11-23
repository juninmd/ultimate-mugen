# Story checkpoint flow in the web QA gate

The checkpoint smoke now runs in the `ui` shard immediately after the story smoke. It clears stage 1, persists a checkpoint, resumes it through the checkpoint menu, then chooses a new campaign and confirms the previous checkpoint remains intact.

The assertion includes the current story decision snapshot (`mercy`, `helper_pending`, `helper_cursor`, `helper_chosen`). These are default resume state, not saved progress fields. The smoke uses the runner's `WEB_SMOKE_SHOTS` path by default so CI stores the screens as an artifact.

Captured on Chromium/SwiftShader at 1280x720:

- Stage clear: `01-stage-cleared.png`
- Resume/new campaign choice: `02-checkpoint-menu.png`
- Resumed stage: `03-resumed-stage.png`

Validation: `node scripts/web-qa.js --dist <web-dist> --only story-checkpoint`; `node --test scripts/web-qa.test.mjs`.
