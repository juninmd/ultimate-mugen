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
  const wait = (step, height = 0) => match.page.waitForFunction(({ step, height }) => {
    const state = JSON.parse(window.__ume_state || 'null');
    const clock = state?.clock;
    const player = state?.fighters?.find(f => f.side === 0);
    if (state?.screen !== 'fight' || !clock?.arena || !player) return false;
    switch (step) {
      case 'start': return clock.round_state === 2;
      case 'approach': return player.x < -218;
      case 'land': return player.ground <= height && player.y === player.ground && player.state === 0;
      case 'edge': return player.x < -510 && clock.round_state === 2;
      case 'fall': return player.state === 19903 && player.alive && player.y > 12;
      case 'result': return clock.winner === 1 && !player.alive && player.x < -554 && player.y >= 184;
      default: return false;
    }
  }, { step, height }, { timeout: 35000 });

  try {
    await wait('start');
    await match.shot('start');
    await match.page.keyboard.down('ArrowLeft');
    await wait('approach');
    await match.page.keyboard.up('ArrowLeft');
    for (const [height, shot] of [[-50, 'platform'], [-102, 'second-tier']]) {
      await match.page.keyboard.down('ArrowUp');
      await match.page.waitForTimeout(120);
      await match.page.keyboard.up('ArrowUp');
      await wait('land', height);
      await match.shot(shot);
    }
    await match.page.keyboard.down('ArrowLeft');
    await wait('edge');
    await match.shot('edge');
    await wait('fall');
    await match.shot('fall');
    await wait('result');
    await match.page.keyboard.up('ArrowLeft');
    await match.shot('ring-out');
    assert.deepEqual(match.problems, []);
    console.log('POWER ARENA OK platform ascent, arena edge, ring-out');
  } finally {
    await match.close();
  }
}

run().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
