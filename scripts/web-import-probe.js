// Web import latency probe: imports a pack through the real file input and reports how long the main thread stopped
// painting (requestAnimationFrame gaps), then deep-links into a fight with the imported fighter and reports the gap
// while it is loaded.  Usage: node scripts/web-import-probe.js [web/dist] <pack.zip>
const fs = require('fs'), path = require('path');
const { openMatch, wait } = require('./web-match-driver');

const dir = process.argv[2] || 'web/dist';
const pack = path.resolve(process.argv[3] || 'assets/mugen/elecbyte.zip');

const recorder = () => {
  window.__gaps = [];
  let last = performance.now();
  const step = () => {
    const now = performance.now();
    window.__gaps.push(Math.round(now - last));
    last = now;
    requestAnimationFrame(step);
  };
  requestAnimationFrame(step);
};

const stats = gaps => {
  if (!gaps.length) return { frames: 0 };
  const sorted = [...gaps].sort((a, b) => a - b);
  const at = q => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * q))];
  return {
    frames: gaps.length,
    median: at(0.5),
    p95: at(0.95),
    max: sorted[sorted.length - 1],
    over100: gaps.filter(g => g > 100).length,
    over250: gaps.filter(g => g > 250).length,
    over1000: gaps.filter(g => g > 1000).length,
  };
};

(async () => {
  const session = await openMatch({ dir, query: 'e2e=1&perf=1', settings: { mute: 1, gamespeed: 2 }, shotDir: process.env.WEB_SMOKE_SHOTS, label: 'probe' });
  const { page, problems } = session;
  const perf = [];
  page.on('console', m => { const t = m.text(); if (t.includes('[perf]')) perf.push(t.replace('[perf] ', '')); });
  await page.evaluate(recorder);
  let state = await session.state();
  for (let i = 0; i < 40 && (!state || !state.roster); i++) { await wait(500); state = await session.state(); }
  const before = state.roster.chars;
  await page.evaluate(() => { if (window.__gaps) window.__gaps.length = 0; });
  const started = Date.now();
  await page.setInputFiles('#ume-file', pack);
  let imported = false;
  for (let i = 0; i < 240 && !imported; i++) {
    await wait(250);
    const s = await session.state().catch(() => null);
    if (s && !s.import.busy && !s.import.queued && s.roster.chars > before) imported = true;
  }
  const importMs = Date.now() - started;
  const importGaps = await page.evaluate(() => (window.__gaps || []).splice(0));
  state = await session.state();
  const tail = (state?.roster?.tail || [])[0] || '';
  const needle = process.env.MUGEN_SMOKE_CHAR || (tail.split(':').pop() || '').split('/').pop().replace(/\.def$/i, '');
  const url = new URL(page.url());
  url.search = `?e2e=1&perf=1&p1=${encodeURIComponent(needle)}&p2=${encodeURIComponent(needle)}&modo=watch&luta=1`;
  const fightStarted = Date.now();
  await page.addInitScript(recorder);
  await page.goto(url.href);
  const navStarted = Date.now();
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  const bootMs = Date.now() - navStarted;
  await page.locator('#glcanvas').focus();
  let screen = '';
  for (let i = 0; i < 120; i++) {
    state = await session.state();
    screen = state?.screen || '';
    if (screen === 'fight') break;
    await wait(250);
  }
  const fightMs = Date.now() - fightStarted;
  await wait(4000);
  const fightGaps = await page.evaluate(() => (window.__gaps || []).splice(0));
  console.log(JSON.stringify({
    pack: path.basename(pack),
    packMb: +(fs.statSync(pack).size / 1048576).toFixed(1),
    imported,
    importMs,
    import: stats(importGaps),
    charLoadMs: fightMs,
    bootMs,
    screen,
    fight: stats(fightGaps),
    wasmMb: state?.wasm_mb,
    perf: perf.slice(0, 18),
    perfTail: perf.slice(-10),
    problems,
  }, null, 2));
  await session.close();
  process.exit(problems.length ? 1 : 0);
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });
