const fs = require('fs'), path = require('path');
const { openMatch, wait } = require('./web-match-driver');
const dir = process.argv[2] || 'web/dist';
const out = process.argv[3] || 'atlas-probe';
const seconds = Number(process.env.PROBE_SECONDS || 60);
const query = process.env.PROBE_QUERY || 'e2e=1&p1=suzume&p2=shion&modo=cpu&luta=1';
const attackKeys = ['q', 'w', 'a', 's', 'e', 'd'];
const pct = (a, p) => (a.length ? a.slice().sort((x, y) => x - y)[Math.min(a.length - 1, Math.floor(a.length * p))] : 0);

function botKeys(s, mash) {
  const me = s.fighters.find(f => f.side === 0), foes = s.fighters.filter(f => f.side === 1 && f.alive);
  const want = new Set();
  if (!me || !foes.length) return want;
  const foe = foes.reduce((a, f) => (Math.abs(f.x - me.x) < Math.abs(a.x - me.x) ? f : a));
  const dx = foe.x - me.x;
  if (Math.abs(dx) > 55) want.add(dx > 0 ? 'ArrowRight' : 'ArrowLeft');
  if (Math.abs(dx) < 90) want.add(attackKeys[(mash >> 1) % attackKeys.length]);
  return want;
}

(async () => {
  fs.mkdirSync(out, { recursive: true });
  const session = await openMatch({ dir, query, settings: { mute: 1, rounds: 0, single_round: 1, round_time: 99, life: 8 }, shotDir: out, label: 'probe' });
  await session.page.evaluate(() => {
    window.__probe = [];
    let last = performance.now();
    const loop = t => {
      try { const g = JSON.parse(window.__ume_state).gpu; window.__probe.push([t - last, g.up, g.kb, g.rf, g.resets, g.tresets]); } catch (e) {}
      last = t;
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
  });
  let mash = 0, started = 0;
  const begin = Date.now();
  while (Date.now() - begin < 180000) {
    const s = await session.state();
    if (s && s.screen === 'fight' && s.fighters && s.fighters.length) {
      if (!started) { started = Date.now(); await session.page.evaluate(() => { window.__probe.length = 0; }); }
      if (Date.now() - started > seconds * 1000) break;
      await session.setKeys(botKeys(s, ++mash));
    }
    await session.frames(3);
  }
  await session.shot('end');
  const rows = await session.page.evaluate(() => window.__probe);
  const ft = rows.map(r => r[0]), kb = rows.map(r => r[2]), up = rows.map(r => r[1]);
  const resets = rows.length ? rows[rows.length - 1][4] - rows[0][4] : 0;
  const summary = {
    query, seconds, frames: rows.length,
    frame_ms: { p50: +pct(ft, 0.5).toFixed(1), p95: +pct(ft, 0.95).toFixed(1), p99: +pct(ft, 0.99).toFixed(1), max: +Math.max(0, ...ft).toFixed(1) },
    upload_kb: { total: kb.reduce((a, c) => a + c, 0), max_frame: Math.max(0, ...kb), p99: pct(kb, 0.99) },
    uploads: { total: up.reduce((a, c) => a + c, 0), max_frame: Math.max(0, ...up) },
    atlas_resets: resets, transient_resets: rows.length ? rows[rows.length - 1][5] - rows[0][5] : 0, rgba_fill_end: rows.length ? rows[rows.length - 1][3] : 0,
    problems: session.problems,
  };
  fs.writeFileSync(path.join(out, 'summary.json'), JSON.stringify(summary, null, 1));
  fs.writeFileSync(path.join(out, 'frames.json'), JSON.stringify(rows));
  console.log(JSON.stringify(summary));
  await session.close();
})().catch(e => { console.error(e); process.exit(1); });
