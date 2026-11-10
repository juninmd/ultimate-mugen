// Arcade click smoke: boots the web build, opens the menu, clicks "Arcade" with the mouse and checks that the game keeps
// running (frames advance, no page error or panic, the screen changes). Regression guard for "clicking Arcade freezes".
// Usage:  node scripts/web-arcade-smoke.js [web/dist]   (needs `playwright`; build first with ./scripts/build-web.sh)
const fs = require('fs'), path = require('path'), os = require('os');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const ARCADE_POINT = { x: 200, y: 216 };
const server = staticServer(dir, types);
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const p = await b.newPage({ viewport: { width: 1280, height: 720 } });
  const problems = [], consoleTail = [];
  p.on('crash', () => problems.push('Chromium page crashed'));
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => {
    consoleTail.push(`${m.type()}: ${m.text().slice(0, 200)}`); if (consoleTail.length > 40) consoleTail.shift();
    if (/panicked|PanicHookInfo|renderbufferStorageMultisample/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`);
  });
  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await p.waitForTimeout(12000);
  const shotDir = process.env.WEB_SMOKE_SHOTS; if (shotDir) fs.mkdirSync(shotDir, { recursive: true });
  const snap = async name => { if (shotDir) await p.screenshot({ path: path.join(shotDir, `${name}.png`) }).catch(() => {}); };
  const frameCount = () => p.evaluate(() => new Promise(done => { let n = 0; const t0 = performance.now(); const step = () => { n++; if (performance.now() - t0 < 1000) requestAnimationFrame(step); else done(n); }; requestAnimationFrame(step); }));
  const frames = n => p.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  await p.locator('#glcanvas').focus();
  await p.keyboard.down('Enter'); await frames(2); await p.keyboard.up('Enter'); await frames(30); await p.waitForTimeout(1500);
  await snap('01-menu');
  const fpsMenu = await frameCount();
  await p.mouse.move(ARCADE_POINT.x, ARCADE_POINT.y); await frames(5);
  await p.mouse.down(); await frames(2); await p.mouse.up();
  await snap('02-after-arcade-click');
  // the game must leave the menu and keep drawing: sample frame rate after the click
  await p.waitForTimeout(1500);
  const fpsAfter = await frameCount();
  await snap('03-after-1s');
  await p.waitForTimeout(3000);
  const fpsLater = await frameCount();
  await snap('04-after-4s');
  const shot = await p.screenshot();
  await b.close(); server.close();
  if (fpsMenu < 5) problems.push(`menu not animating before click (frames/s ${fpsMenu})`);
  if (fpsAfter < 5) problems.push(`game frozen after clicking Arcade: frames/s ${fpsAfter}, later ${fpsLater}`);
  if (fpsLater < 5) problems.push(`game frozen 4 s after clicking Arcade: frames/s ${fpsLater}`);
  fs.writeFileSync(path.join(os.tmpdir(), 'web-arcade-smoke.png'), shot);
  const info = `frames/s menu=${fpsMenu} afterClick=${fpsAfter} later=${fpsLater}`;
  if (problems.length) { console.error('WEB ARCADE SMOKE FAILED ' + info + '\n' + problems.join('\n') + '\nlast console messages:\n' + consoleTail.join('\n')); process.exit(1); }
  console.log('WEB ARCADE SMOKE OK ' + info);
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });
