// Web smoke test: drives the real flow in headless Chromium (title → menu → select → stage → VS → fight) and fails on any
// page error / Rust panic or a black screen.   Usage:  node scripts/web-smoke.js [web/dist]
// (needs `playwright`; build first with ./scripts/build-web.sh). Regression guard for "black screen after choosing fighters".
const fs = require('fs'), path = require('path'), os = require('os');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const server = staticServer(dir, types);
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const p = await b.newPage({ viewport: { width: 1280, height: 720 } });
  const problems = [];
  p.on('crash', () => problems.push('Chromium page crashed'));
  const stageRequests = [];
  p.on('request', r => { if (/\/assets\/stages\/[^/]+\.png$/.test(new URL(r.url()).pathname)) stageRequests.push(r.url()); });
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => {
    if (/panicked|PanicHookInfo|renderbufferStorageMultisample/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`);
  });
  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await p.waitForTimeout(1500);
  await p.locator('#glcanvas').click();
  const key = async (k, wait = 900) => { await p.keyboard.down(k); await p.waitForTimeout(120); await p.keyboard.up(k); await p.waitForTimeout(wait); };
  await key('Enter');                       // title → menu
  await key('ArrowDown'); await key('ArrowDown'); await key('Enter', 1500);  // Arcade, Survival → CPU (Versus CPU) → select
  await key('Enter', 1500); await key('Enter', 1500); // P1, P2 → stages
  if (stageRequests.length) problems.push(`Full stage images fetched before confirmation: ${stageRequests.length}`);
  await key('Enter', 1500);                 // stage → VS / loading
  await p.waitForTimeout(10000);            // loading + VS + intro
  if (stageRequests.length !== 1) problems.push(`Expected one selected stage download, got ${stageRequests.length}`);
  // releaseAll (blur, pagehide, contextmenu) synthesises a pointerup: it must be a real PointerEvent, or the
  // wasm glue throws on event.pointerType and the input stays stuck.
  await p.locator('#glcanvas').click({ button: 'right' });
  await p.evaluate(() => window.dispatchEvent(new Event('blur')));
  await p.waitForTimeout(400);
  const shot = await p.screenshot();
  await b.close(); server.close();
  // black-screen check: the fight must have visible content (mean luminance well above 0 and some colour variance)
  const { PNG } = (() => { try { return require('pngjs'); } catch { return {}; } })();
  let lum = null;
  if (PNG) { const img = PNG.sync.read(shot); let s = 0, s2 = 0, n = img.width * img.height; for (let i = 0; i < n; i++) { const l = (img.data[i * 4] + img.data[i * 4 + 1] + img.data[i * 4 + 2]) / 3; s += l; s2 += l * l; } const m = s / n; lum = { mean: m, sd: Math.sqrt(s2 / n - m * m) }; if (m < 8 || lum.sd < 6) problems.push(`black/flat screen: ${JSON.stringify(lum)}`); }
  fs.writeFileSync(path.join(os.tmpdir(), 'web-smoke-fight.png'), shot);
  if (problems.length) { console.error('WEB SMOKE FAILED\n' + problems.join('\n')); process.exit(1); }
  console.log('WEB SMOKE OK', lum ? JSON.stringify(lum) : '(no pngjs: luminance check skipped)');
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });
