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
  const consoleTail = [];
  p.on('crash', () => problems.push('Chromium page crashed'));
  const stageRequests = [];
  const timeline = [];
  p.on('request', r => { if (/\/assets\/stages\/[^/]+\.png$/.test(new URL(r.url()).pathname)) stageRequests.push(r.url()); });
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => {
    consoleTail.push(`${m.type()}: ${m.text().slice(0, 160)}`); if (consoleTail.length > 40) consoleTail.shift();
    if (/panicked|PanicHookInfo|renderbufferStorageMultisample/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`);
  });
  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await p.waitForTimeout(12000);
  await p.locator('#glcanvas').focus();
  const key = async (k, wait = 1000) => { await p.keyboard.down(k); await p.waitForTimeout(200); await p.keyboard.up(k); await p.waitForTimeout(wait); };
  await key('Enter');                       // title → menu
  await key('ArrowDown'); await key('ArrowDown'); await key('Enter', 1500);  // Arcade, Survival → CPU (Versus CPU) → select
  await key('Enter', 1500); await key('Enter', 1500); // P1, P2 → stages
  if (stageRequests.length) problems.push(`Full stage images fetched before confirmation: ${stageRequests.length}`);
  // A slow runner can stall the wasm main thread for seconds and swallow a tap, so the confirm key is retried until the
  // stage request shows up instead of trusting one fixed 1.5 s step.
  const startedAt = Date.now();
  for (let attempt = 0; attempt < 4 && !stageRequests.length; attempt++) {
    await key('Enter', 500);                // stage → VS / loading
    await p.waitForRequest(r => /\/assets\/stages\/[^/]+\.png$/.test(new URL(r.url()).pathname), { timeout: 8000 }).catch(() => null);
    timeline.push(`confirm attempt ${attempt + 1} at +${Date.now() - startedAt} ms, stage requests ${stageRequests.length}`);
  }
  await p.waitForTimeout(6000);             // intro + fight start
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
  if (problems.length) { console.error('WEB SMOKE FAILED\n' + problems.join('\n') + '\ntimeline:\n' + timeline.join('\n') + '\nlast console messages:\n' + consoleTail.join('\n')); process.exit(1); }
  console.log('WEB SMOKE OK', lum ? JSON.stringify(lum) : '(no pngjs: luminance check skipped)');
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });
