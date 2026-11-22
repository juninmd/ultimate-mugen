// Team size smoke: mode chip -> TEAM · PARALLEL, size chip up to the cap, pick the squad and reach a fight.
// Usage: node scripts/web-team-smoke.js [web/dist]   (TEAM_MOBILE=1 taps in a phone viewport, cap 7 either way on the web build)
const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const server = staticServer(dir, types);
const mobile = !!process.env.TEAM_MOBILE;
const PICKS = 7;
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html?e2e=1`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const p = await b.newPage(mobile ? { viewport: { width: 844, height: 390 }, hasTouch: true, isMobile: true, deviceScaleFactor: 2 } : { viewport: { width: 1280, height: 720 } });
  const problems = [];
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => { if (/panicked|PanicHookInfo/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`); });
  const shotDir = process.env.WEB_SMOKE_SHOTS; if (shotDir) fs.mkdirSync(shotDir, { recursive: true });
  const snap = async name => { if (shotDir) await p.screenshot({ path: path.join(shotDir, `${name}.png`) }).catch(() => {}); };
  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await p.waitForTimeout(12000);
  await p.locator('#glcanvas').focus();
  const frames = n => p.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const key = async (k, wait = 900) => { await p.keyboard.down(k); await frames(2); await p.keyboard.up(k); await frames(5); await p.waitForTimeout(wait); };
  const state = () => p.evaluate(() => JSON.parse(window.__ume_state || 'null'));
  const box = async () => p.evaluate(() => { const c = document.querySelector('#glcanvas').getBoundingClientRect(); const s = Math.min(c.width / 1280, c.height / 720); return { x: c.left + (c.width - 1280 * s) / 2, y: c.top + (c.height - 720 * s) / 2, s }; });
  const click = async (vx, vy, wait = 900) => {
    const g = await box(), x = g.x + vx * g.s, y = g.y + vy * g.s;
    if (mobile) await p.touchscreen.tap(x, y); else { await p.mouse.move(x, y); await frames(2); await p.mouse.down(); await frames(2); await p.mouse.up(); }
    await frames(4); await p.waitForTimeout(wait);
  };
  await key('Enter', 1500);
  for (let i = 0; i < 3; i++) await key('ArrowDown'); await key('Enter', 1500);
  await snap('01-select-versus-cpu');
  const MODE_NEXT = [1221, 20], SIZE_PLUS = [1221, 49];
  for (let i = 0; i < 4; i++) await click(...MODE_NEXT);
  await snap('02-mode-chip-team');
  for (let i = 0; i < 4; i++) await click(...SIZE_PLUS, 500);
  await snap('03-size-7v7');
  for (let i = 0; i < PICKS; i++) { await key('Enter', 700); await key('ArrowRight', 300); }
  await snap('04-squad-picked');
  await key('Enter', 1500);
  await p.waitForFunction(() => { const s = JSON.parse(window.__ume_state || 'null'); return s?.screen === 'fight' && s.clock?.round_state === 2; }, null, { timeout: 120000 }).catch(() => problems.push('fight did not start'));
  await p.waitForTimeout(1500);
  const s = await state();
  const roots = s?.fighters?.length || 0;
  if (roots !== PICKS * 2) problems.push(`expected ${PICKS * 2} fighters, got ${roots}`);
  const xs = (s?.fighters || []).map(f => Math.round(f.x ?? f.pos?.[0] ?? NaN));
  console.log('fighters', roots, 'x', JSON.stringify(xs));
  await snap('05-fight-7v7');
  await b.close(); server.close();
  if (problems.length) { console.error(problems.join('\n')); console.log('WEB TEAM SMOKE FAILED'); process.exit(1); }
  console.log('WEB TEAM SMOKE OK');
})().catch(e => { console.error(e); process.exit(1); });
