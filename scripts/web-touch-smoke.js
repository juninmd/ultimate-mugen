// Mobile touch smoke: drives title → fight with the keyboard, then taps every touch button in a phone-sized viewport
// and fails on page errors or Rust panics. Usage: node scripts/web-touch-smoke.js [web/dist] (build with ./scripts/build-web.sh).
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
  p.on('crash', () => problems.push('Chromium page crashed'));
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => { if (/panicked|PanicHookInfo/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`); });
  const shotDir = process.env.WEB_SMOKE_SHOTS; if (shotDir) fs.mkdirSync(shotDir, { recursive: true });
  const snap = async name => { if (shotDir) await p.screenshot({ path: path.join(shotDir, `${name}.png`) }).catch(() => {}); };
  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await p.waitForTimeout(12000);
  await p.locator('#glcanvas').focus();
  const frames = n => p.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const key = async (k, wait = 1000) => { await p.keyboard.down(k); await frames(2); await p.keyboard.up(k); await frames(5); await p.waitForTimeout(wait); };
  await key('Enter');
  for (let i = 0; i < 3; i++) await key('ArrowDown');
  await key('Enter', 1500);
  await key('Enter', 1500);
  await key('Enter', 1500);
  for (let attempt = 0; attempt < 4; attempt++) await key('Enter', 500);
  await p.waitForTimeout(8000);
  await snap('00-fight');
  const layout = await p.evaluate(() => {
    const w = innerWidth, h = innerHeight;
    const r = Math.min(44, Math.max(20, h * 0.075)), gap = r * 2.1, small = r * 0.72, sg = small * 2.3;
    const bx = w - r * 1.4 - gap * 2, by = h - r * 2.3;
    const extra = n => [bx - gap * 0.85 - (3 - (n % 4)) * sg, by - Math.floor(n / 4) * sg];
    const k = Math.min(w / 1280, h / 720);
    return {
      soco: [bx, by - gap], especial: [bx + 2 * gap, by - gap], chute: [bx, by], super: [bx + 2 * gap, by],
      toggle: [bx + 2 * gap, by - 2 * gap + r * 0.2],
      carregar: extra(1), troca: extra(6), defesa: extra(10), soco3: extra(8), chute3: extra(9), especial3: extra(7),
      quick: [2, 3, 4, 5].map(extra), pause: [w / 2, (h - 720 * k) / 2 + 116 * k],
    };
  });
  const tap = async ([x, y]) => { await p.touchscreen.tap(x, y); await frames(4); await p.waitForTimeout(250); };
  await tap(layout.soco); await tap(layout.chute); await tap(layout.especial); await tap(layout.super); await snap('01-core-buttons');
  await tap(layout.toggle); await snap('02-extras-open');
  await tap(layout.carregar);
  await tap(layout.troca); await tap(layout.defesa); await tap(layout.soco3); await tap(layout.chute3); await tap(layout.especial3);
  for (const q of layout.quick) await tap(q);
  await snap('03-after-extras');
  await tap(layout.toggle); await snap('04-extras-closed');
  await tap(layout.pause); await p.waitForTimeout(1500); await snap('05-paused');
  if (problems.length) console.error(problems.join('\n'));
  console.log(problems.length ? 'WEB TOUCH SMOKE FAILED' : 'WEB TOUCH SMOKE OK');
  await b.close(); server.close();
  process.exit(problems.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
