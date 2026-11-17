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
    const r = Math.min(52, Math.max(24, h * 0.09)), gap = r * 2.25;
    const bx = w - r * 1.3 - gap * 2, by = h - r * 2.4;
    const quick = [0, 1, 2, 3].map(j => [w * 0.57 + (j - 1.5) * r * 1.8, h - r * 0.9]);
    return {
      soco: [bx, by - gap], socoPlus: [bx + gap, by - gap], especial: [bx + 2 * gap, by - gap],
      chute: [bx, by], chutePlus: [bx + gap, by], super: [bx + 2 * gap, by],
      despertar: [bx + gap, by - 2 * gap], carregar: [bx + 2 * gap, by - 2 * gap],
      troca: [bx, by - 2 * gap], especial3: [bx - gap, by - 2 * gap], soco3: [bx - gap, by - gap],
      chute3: [bx - 2 * gap, by - 2 * gap], defesa: [bx - 2 * gap, by - gap],
      quick, pause: [w / 2, (h - 720 * Math.min(w / 1280, h / 720)) / 2 + 116 * Math.min(w / 1280, h / 720)],
    };
  });
  const tap = async ([x, y]) => { await p.touchscreen.tap(x, y); await frames(4); await p.waitForTimeout(250); };
  await tap(layout.carregar); await snap('01-after-carregar');
  await tap(layout.troca); await tap(layout.defesa); await snap('02-after-troca-defesa');
  await tap(layout.soco3); await tap(layout.chute3); await tap(layout.especial3); await snap('03-after-3o-ataque');
  for (const q of layout.quick) await tap(q);
  await snap('04-after-quick-skills');
  await snap('05a-before-pause');
  await tap(layout.pause); await p.waitForTimeout(1500); await snap('05-paused');
  if (problems.length) console.error(problems.join('\n'));
  console.log(problems.length ? 'WEB TOUCH SMOKE FAILED' : 'WEB TOUCH SMOKE OK');
  await b.close(); server.close();
  process.exit(problems.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
