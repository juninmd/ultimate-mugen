# Epilogue atlas reuse

Chromium captures of the candidate epilogue flow:

- First page: `after-epilogue-page-1.png`
- Home scene: `after-home-page.png`
- End card: `after-end-card.png`
- Story result: `after-story-rank.png`

The same 1280x720 software-WebGL browser probe advanced through epilogue pages 1–5. Baseline and candidate were built from the same `cfd9246` source snapshot, with only the atlas-reset call toggled. Both probes sampled 259 frames. Baseline reported 17 upload operations and 5,015 KB; candidate reported 10 operations and 3,058 KB. The candidate therefore had 7 fewer upload operations and 1,957 fewer reported KB (39.0%) in this matched probe. Atlas reset/generation counters rose from 2 to 6 in the baseline and stayed at 1 throughout the candidate transitions.

After rebasing, candidate-only probes on `9aa1080` (228 frames) and `132ba27` (262 frames) also reported 10 uploads and 3,058 KB. Those intervening commits do not change `src/app/story/epilogue.rs` or `crates/shell/src/quad.rs`.

The committed probe can be rerun with `node scripts/web-epilogue-atlas-bench.js web/dist <optional-shot-directory>` against each built bundle.

This measures the renderer's upload counters in Chromium/SwiftShader; it is not an FPS or wall-clock speedup claim. The captures document the epilogue and result screens after the change.

Validation: `scripts/build-web.sh`; `scripts/web-epilogue-smoke.js web/dist`; `node scripts/web-epilogue-atlas-bench.js web/dist`; `node scripts/file-size.mjs`; `node --check scripts/web-epilogue-smoke.js`; `git diff --check`.
