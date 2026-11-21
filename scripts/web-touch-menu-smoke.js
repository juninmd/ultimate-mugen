// Mobile menu smoke: with only taps (no keyboard) title -> menu -> d-pad/OK/BACK pad, in a phone viewport. Usage: node scripts/web-touch-menu-smoke.js [web/dist]
const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const server = staticServer(dir, types);
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const p = await b.newPage({ viewport: { width: 844, height: 390 }, hasTouch: true, isMobile: true, deviceScaleFactor: 2 });
  const problems = [];
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => { if (/panicked|PanicHookInfo/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`); });
  const shotDir = process.env.WEB_SMOKE_SHOTS; if (shotDir) fs.mkdirSync(shotDir, { recursive: true });
  const snap = async name => { if (shotDir) await p.screenshot({ path: path.join(shotDir, `${name}.png`) }).catch(() => {}); };
  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await p.waitForTimeout(12000);
  const frames = n => p.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const tap = async ([x, y], wait = 1200) => { await p.touchscreen.tap(x, y); await frames(4); await p.waitForTimeout(wait); };
  const L = await p.evaluate(() => {
    const w = innerWidth, hh = innerHeight, r = Math.min(44, Math.max(20, hh * 0.075)), gap = r * 2.1;
    const bx = w - r * 1.4 - gap * 2, by = hh - r * 2.3, sx = r * 2.8, sy = hh - r * 3.1, sr = r * 2 * 0.5 * 1.7;
    return { ok: [bx + 2 * gap, by], back: [bx, by], down: [sx, sy + sr], up: [sx, sy - sr], center: [w / 2, hh / 2] };
  });
  await snap('00-title-touch');
  await tap(L.center, 1500); await snap('01-menu-after-tap-title');
  await tap(L.down, 400); await tap(L.down, 400); await snap('02-menu-dpad-down');
  await tap(L.ok, 1800); await snap('03-screen-after-ok');
  await tap(L.back, 1800); await snap('04-back-to-menu');
  if (problems.length) console.error(problems.join('\n'));
  console.log(problems.length ? 'WEB TOUCH MENU SMOKE FAILED' : 'WEB TOUCH MENU SMOKE OK');
  await b.close(); server.close();
  process.exit(problems.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
