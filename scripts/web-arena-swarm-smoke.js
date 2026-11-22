const assert = require('node:assert/strict');
const { openMatch } = require('./web-match-driver');

async function run() {
  const mobile = process.env.ARENA_MOBILE === '1';
  const match = await openMatch({
    dir: process.argv[2] || 'web/dist',
    query: 'e2e=god&p1=kaito&modo=arena&luta=1&inimigos=100&time=3',
    settings: { difficulty: 1, mute: 1 },
    shotDir: process.env.WEB_SMOKE_SHOTS,
    label: mobile ? 'arena-swarm-mobile' : 'arena-swarm',
  });
  if (mobile) await match.page.setViewportSize({ width: 844, height: 390 });
  const until = (check, timeout = 60000) => match.page.waitForFunction(check, null, { timeout });
  try {
    await until(() => { const s = JSON.parse(window.__ume_state || 'null'); return s?.screen === 'fight' && s.clock?.round_state === 2; });
    const first = await match.state();
    assert.equal(first.clock.foes_total, 100);
    assert.ok(first.clock.arena_half > 1108, `map should be wider than the base arena, got ${first.clock.arena_half}`);
    const squad = first.fighters.filter(f => f.side === 0).length;
    const foes = first.fighters.filter(f => f.side === 1).length;
    assert.equal(squad, 3);
    assert.ok(foes >= 8 && foes <= 12, `active enemies ${foes}`);
    assert.equal(first.clock.elixir_y, -414, 'elixir waits on the highest platform');
    assert.equal(first.clock.elixir_taken, false);
    await match.shot('start');
    await match.page.keyboard.down('ArrowRight');
    await until(() => { const s = JSON.parse(window.__ume_state || 'null'); return s?.clock?.tick >= 900; }, 90000);
    await match.page.keyboard.up('ArrowRight');
    const later = await match.state();
    await match.shot('fight');
    const alive = later.fighters.filter(f => f.side === 1 && f.alive).length;
    assert.ok(alive <= 12, `enemies on the field stay capped, got ${alive}`);
    assert.deepEqual(match.problems, []);
    console.log(`ARENA SWARM OK ${squad} vs ${later.clock.foes_total}, tick ${later.clock.tick}, map ${later.clock.arena_half * 2}px, wasm ${later.wasm_mb} MB`);
  } finally {
    await match.close();
  }
}

run().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
