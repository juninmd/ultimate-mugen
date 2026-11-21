// Remote-pack homologation: loads a fighter straight from a GitHub repository the way the marketplace does, and
// reports the network and frame costs.  Usage: node scripts/web-remote-smoke.js [web/dist] [owner/repo] [zip path]
// Defaults to juninmd/mugen-assets and chars/Aquaman.zip.  Needs network; not part of CI.
const fs = require('fs'), path = require('path');
const { openMatch, wait } = require('./web-match-driver');

const dir = process.argv[2] || 'web/dist';
const repo = process.argv[3] || 'juninmd/mugen-assets';
const zipPath = process.argv[4] || 'chars/Aquaman.zip';
const api = `https://api.github.com/repos/${repo}/git/trees/main?recursive=1`;
const raw = `https://raw.githubusercontent.com/${repo}/main/${zipPath}`;
const shotDir = process.env.WEB_SMOKE_SHOTS;

const stats = gaps => {
  if (!gaps.length) return { frames: 0 };
  const sorted = [...gaps].sort((a, b) => a - b);
  const at = q => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * q))];
  return { frames: gaps.length, median: at(0.5), p95: at(0.95), max: sorted[sorted.length - 1], over100: gaps.filter(g => g > 100).length };
};

const recorder = () => {
  window.__gaps = [];
  let last = performance.now();
  const step = () => { const now = performance.now(); window.__gaps.push(Math.round(now - last)); last = now; requestAnimationFrame(step); };
  requestAnimationFrame(step);
};

(async () => {
  const session = await openMatch({ dir, query: 'e2e=1', settings: { mute: 1, gamespeed: 2 }, shotDir, label: 'remote' });
  const { page, problems } = session;
  await page.evaluate(recorder);
  let state = await session.state();
  for (let i = 0; i < 40 && (!state || !state.roster); i++) { await wait(500); state = await session.state(); }
  const before = state.roster.chars;

  const listing = await page.evaluate(async url => {
    const t0 = performance.now();
    const res = await fetch(url, { headers: { Accept: 'application/vnd.github+json' } });
    const body = res.ok ? await res.json() : null;
    return { ok: res.ok, status: res.status, ms: Math.round(performance.now() - t0), zips: body ? body.tree.filter(e => /\.zip$/i.test(e.path)).length : 0, size: body ? body.tree.find(e => /\.zip$/i.test(e.path))?.size : 0 };
  }, api);
  if (!listing.ok) problems.push(`GitHub listing failed: ${listing.status}`);

  await page.evaluate(() => { if (window.__gaps) window.__gaps.length = 0; });
  const fetched = await page.evaluate(async url => {
    const t0 = performance.now();
    const res = await fetch(url);
    if (!res.ok) return { ok: false, status: res.status };
    const bytes = new Uint8Array(await res.arrayBuffer());
    const name = url.split('/').pop();
    window.umeImport([new File([bytes], name, { type: 'application/zip' })]);
    return { ok: true, ms: Math.round(performance.now() - t0), mb: +(bytes.length / 1048576).toFixed(1) };
  }, raw);
  if (!fetched.ok) problems.push(`raw download failed: ${fetched.status}`);

  let imported = false;
  const started = Date.now();
  for (let i = 0; i < 240 && !imported; i++) {
    await wait(250);
    const s = await session.state().catch(() => null);
    if (s && !s.import.busy && !s.import.queued && s.roster.chars > before) imported = true;
  }
  const importMs = Date.now() - started;
  const importGaps = await page.evaluate(() => (window.__gaps || []).splice(0));
  state = await session.state();
  const tail = (state?.roster?.tail || [])[0] || '';
  const needle = tail.split(':').pop().split('/').pop().replace(/\.def$/i, '');

  const url = new URL(page.url());
  url.search = `?e2e=1&p1=${encodeURIComponent(needle)}&p2=${encodeURIComponent(needle)}&modo=watch&luta=1`;
  const fightAt = Date.now();
  await page.addInitScript(recorder);
  await page.goto(url.href);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  const bootMs = Date.now() - fightAt;
  await page.locator('#glcanvas').focus();
  let screen = '';
  for (let i = 0; i < 120; i++) {
    state = await session.state();
    screen = state?.screen || '';
    if (screen === 'fight') break;
    await wait(250);
  }
  const fightMs = Date.now() - fightAt;
  await wait(4000);
  const fightGaps = await page.evaluate(() => (window.__gaps || []).splice(0));
  if (!imported) problems.push('the remote pack added no character');
  if (screen !== 'fight') problems.push(`never reached the fight (stuck at ${screen})`);
  const tick = state?.clock?.tick ?? -1;
  if (tick <= 0) problems.push('the match never ticked');

  console.log(JSON.stringify({
    repo, zip: zipPath,
    listing, download: fetched, imported, importMs, import: stats(importGaps),
    bootMs, fightMs, fight: stats(fightGaps), screen, tick, wasmMb: state?.wasm_mb, problems,
  }, null, 2));
  await session.close();
  process.exit(problems.length ? 1 : 0);
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });
