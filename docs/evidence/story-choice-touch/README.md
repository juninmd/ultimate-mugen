# Final boss choice touch evidence

The first screenshot shows the final boss mercy choice. The smoke taps the visible `DERRUBAR / FINISH` option at (896, 510); the second screenshot shows the next helper selection, which proves the choice was recorded and the story flow advanced.

`node scripts/web-prefight-smoke.js web/dist` passed. The E2E state asserted `mercy === false` and `helper_pending === true` after the touch, then continued to the fight. The same controls remain available through Left/Right.
