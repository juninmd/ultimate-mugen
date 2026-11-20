const assert = require('node:assert/strict');
const { openMatch } = require('./web-match-driver');

const dir = process.argv[2] || 'web/dist';
const shotDir = process.env.WEB_SMOKE_SHOTS;

async function check(fighter, forms) {
  const match = await openMatch({
    dir,
    query: `e2e=1&p1=${fighter}&p2=shion&modo=treino&luta=1`,
    settings: { mute: 1 },
    shotDir,
    label: fighter,
  });
  const waitFor = (form, recovered) => match.page.waitForFunction(({ form, recovered }) => {
    const state = JSON.parse(window.__ume_state || 'null');
    const player = state?.fighters?.find(fighter => fighter.side === 0);
    return state?.screen === 'fight' && state.clock?.round_state === 2 &&
      player?.form === form && (!recovered || player.state !== 19902);
  }, { form, recovered }, { timeout: 90000 });

  try {
    await waitFor(0, true);
    await match.shot('base');
    for (let level = 1; level <= forms; level++) {
      await match.tap('t');
      await waitFor(level, false);
      await match.shot(`awakening-${level}`);
      await waitFor(level, true);
      await match.shot(`form-${level}`);
    }
    assert.deepEqual(match.problems, []);
    console.log(`AWAKENING OK ${fighter} forms=0..${forms}`);
  } finally {
    await match.close();
  }
}

(async () => {
  await check('kaito', 2);
  await check('kfm', 1);
})().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
