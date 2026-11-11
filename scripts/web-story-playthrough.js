const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const out = path.resolve(process.env.STORY_OUT || 'story-playthrough');
const god = process.env.STORY_GOD !== '0';
const pickSteps = Number(process.env.STORY_PICK_STEPS || 0);
const limitMs = Number(process.env.STORY_LIMIT_MIN || 120) * 60000;
const importZip = process.env.STORY_IMPORT_ZIP;
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip', '.png': 'image/png' };
const server = staticServer(dir, types);
const wait = ms => new Promise(r => setTimeout(r, ms));
(async () => {
  fs.mkdirSync(out, { recursive: true });
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html?e2e=${god ? 'god' : '1'}${process.env.STORY_STAGE ? `&e2e_stage=${process.env.STORY_STAGE}` : ''}`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader', '--autoplay-policy=no-user-gesture-required'] });
  const p = await b.newPage({ viewport: { width: 1280, height: 720 } });
  const problems = [], log = [], failedRequests = [];
  p.on('crash', () => problems.push('page crashed'));
  p.on('pageerror', e => problems.push(`pageerror: ${e.message}`));
  p.on('console', m => { if (/panicked|PanicHookInfo|error/i.test(m.text())) problems.push(`console ${m.type()}: ${m.text().slice(0, 200)}`); });
  p.on('requestfailed', r => failedRequests.push(r.url()));
  p.on('response', r => { if (r.status() >= 400) failedRequests.push(`${r.status()} ${r.url()}`); });
  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  await p.waitForTimeout(8000);
  await p.locator('#glcanvas').focus();
  const state = () => p.evaluate(() => { try { return JSON.parse(window.__ume_state); } catch { return null; } });
  const frames = n => p.evaluate(n => new Promise(done => { const s = i => (i ? requestAnimationFrame(() => s(i - 1)) : done()); s(n); }), n);
  const tap = async k => { await p.keyboard.down(k); await frames(2); await p.keyboard.up(k); await frames(3); };
  const shot = async name => p.screenshot({ path: path.join(out, `${name}.png`) }).catch(() => {});
  const note = msg => { const line = `[${((Date.now() - t0) / 1000).toFixed(0)}s] ${msg}`; log.push(line); console.log(line); };
  const t0 = Date.now();
  await tap('Enter'); await wait(1200);
  await shot('00-menu');
  await shot('01-menu-story'); await tap('Enter'); await wait(1500);
  for (let i = 0; i < pickSteps; i++) { await tap(process.env.STORY_PICK_KEY || 'ArrowRight'); await wait(200); }
  await shot('02-select');
  await tap('Enter'); await wait(2500);
  const held = new Set();
  const setKeys = async want => {
    for (const k of [...held]) if (!want.has(k)) { await p.keyboard.up(k); held.delete(k); }
    for (const k of want) if (!held.has(k)) { await p.keyboard.down(k); held.add(k); }
  };
  let last = '', fightStart = 0, shotFight = new Set(), mash = 0, stuckSince = Date.now(), lastSig = '';
  const events = [];
  for (;;) {
    if (Date.now() - t0 > limitMs) { note('TIME LIMIT'); break; }
    const s = await state();
    if (!s) { await wait(200); continue; }
    const st = s.story;
    const sig = `${s.screen}|${st ? `${st.stage}.${st.wave}.${st.phase}.${st.credits}` : '-'}`;
    if (sig !== last) {
      note(`${sig} score=${st ? st.score : '-'} fighters=${s.fighters.map(f => `s${f.side}:${f.life.toFixed(0)}/${f.max.toFixed(0)}`).join(' ')}`);
      events.push({ t: Date.now() - t0, screen: s.screen, story: st, fighters: s.fighters });
      last = sig; fightStart = Date.now();
      if (s.screen !== 'fight') { await setKeys(new Set()); await wait(900); await shot(`${String(events.length).padStart(3, '0')}-${s.screen}-${st ? `s${st.stage + 1}w${st.wave + 1}-${st.phase}` : 'none'}`); }
    }
    if (!s.story_mode && s.screen === 'menu' && events.length > 3) { note('RETURNED TO MENU'); break; }
    if (s.screen === 'fight' && s.paused) { await setKeys(new Set()); note('paused in fight, resuming'); await tap('Enter'); await wait(500); continue; }
    if (s.screen === 'fight') {
      const me = s.fighters.find(f => f.side === 0), foes = s.fighters.filter(f => f.side === 1 && f.alive);
      const key = `fight-${st.stage}-${st.wave}`;
      if (!shotFight.has(key) && Date.now() - fightStart > 4000) { shotFight.add(key); await shot(`fight-s${st.stage + 1}w${st.wave + 1}`); }
      if (me && foes.length && s.over < 0) {
        const t = foes.reduce((a, f) => (Math.abs(f.x - me.x) < Math.abs(a.x - me.x) ? f : a));
        const dx = t.x - me.x, want = new Set();
        const dirKey = dx > 0 ? 'ArrowRight' : 'ArrowLeft';
        if (Math.abs(dx) > 55) want.add(dirKey);
        if (t.lane - me.lane > 5) want.add('x'); else if (me.lane - t.lane > 5) want.add('z');
        mash++;
        if (Math.abs(dx) < 90) want.add(['q', 'w', 'a', 's', 'e', 'd'][(mash >> 1) % 6]);
        await setKeys(want);
      } else await setKeys(new Set());
      await frames(3);
    } else {
      await setKeys(new Set());
      if (s.screen === 'story' && st && (st.phase === 'Victory' || st.phase === 'Defeated')) { await wait(2500); await shot(`end-${st.phase}-${s.screen}-${Date.now() - t0}`); }
      if (['vs', 'story', 'cutscene'].includes(s.screen)) await tap('Enter');
      await wait(700);
    }
    if (Date.now() - stuckSince > 15 * 60000 && sig === lastSig) { problems.push(`stuck on ${sig} for 15 min`); break; }
    if (sig !== lastSig) { lastSig = sig; stuckSince = Date.now(); }
  }
  await shot('zz-final');
  fs.writeFileSync(path.join(out, 'events.json'), JSON.stringify({ events, problems, failedRequests }, null, 1));
  fs.writeFileSync(path.join(out, 'timeline.log'), log.join('\n'));
  await b.close(); server.close();
  console.log(problems.length ? `PROBLEMS:\n${problems.join('\n')}` : 'NO PAGE ERRORS');
  console.log(failedRequests.length ? `FAILED REQUESTS:\n${[...new Set(failedRequests)].join('\n')}` : 'NO FAILED REQUESTS');
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });
