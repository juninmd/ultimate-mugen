const assert = require('node:assert/strict');
const { openMatch } = require('./web-match-driver');

async function run() {
  const match = await openMatch({
    dir: process.argv[2] || 'web/dist',
    query: 'e2e=1&p1=kaito&p2=shion&modo=arena&luta=1',
    settings: { difficulty: 1, mute: 1 },
    shotDir: process.env.WEB_SMOKE_SHOTS,
    label: 'power-arena',
  });
  const wait = (step) => match.page.waitForFunction(step => {
    const state = JSON.parse(window.__ume_state || 'null');
    const clock = state?.clock;
    const player = state?.fighters?.find(f => f.side === 0);
    if (state?.screen !== 'fight' || !clock?.arena || !player) return false;
    switch (step) {
      case 'start': return clock.round_state === 2;
      case 'approach': return player.x < -178;
      case 'land': return player.ground === -50 && player.y === player.ground && player.state !== 0;
      case 'edge': return player.x < -350 && clock.round_state === 2;
      case 'result': return clock.winner === 1 && !player.alive && player.x < -426;
      default: return false;
    }
  }, step, { timeout: 35000 });

  try {
    await wait('start');
    await match.shot('start');
    await match.page.keyboard.down('ArrowLeft');
    await wait('approach');
    await match.page.keyboard.up('ArrowLeft');
    await match.tap('ArrowUp');
    await wait('land');
    await match.shot('platform');
    await match.page.keyboard.down('ArrowLeft');
    await wait('edge');
    await match.shot('edge');
    await wait('result');
    await match.page.keyboard.up('ArrowLeft');
    await match.shot('ring-out');
    assert.deepEqual(match.problems, []);
    console.log('POWER ARENA OK platform, arena edge, ring-out');
  } finally {
    await match.close();
  }
}

run().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
