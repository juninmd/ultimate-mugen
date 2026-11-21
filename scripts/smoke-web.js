// Drives the in-engine smoke runner (`?smoke=1`) in headless Chromium, saves a screenshot for every request the game makes
// and writes report.json + report.md.   Usage: node scripts/smoke-web.js [web/dist] [out-dir]   (SMOKE_MOBILE=1 for a phone)
const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium, devices } = require('playwright');
const { analyzeShot, markdown } = require('./smoke-web-report');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const out = path.resolve(process.argv[3] || 'smoke-report');
const limitMs = Number(process.env.SMOKE_LIMIT_MS || 900000);
const only = process.env.UME_SMOKE_ONLY ? `&smoke_only=${process.env.UME_SMOKE_ONLY}` : '';
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip', '.png': 'image/png' };
const ignoredConsole = /favicon|sw\.js|GPU stall|GroupMarker|WebGL: INVALID_OPERATION: texSubImage|\[perf\]|\[smoke\]/;
const sleep = ms => new Promise(r => setTimeout(r, ms));

(async () => {
  fs.mkdirSync(path.join(out, 'shots'), { recursive: true });
  const server = staticServer(dir, types);
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader', '--autoplay-policy=no-user-gesture-required'] });
  const mobile = process.env.SMOKE_MOBILE === '1';
  const context = await browser.newContext(mobile ? { ...devices['Pixel 7'] } : { viewport: { width: 1280, height: 720 } });
  const page = await context.newPage();
  const problems = [];
  page.on('crash', () => problems.push('Chromium page crashed'));
  page.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message.slice(0, 300)}`); });
  page.on('console', m => {
    const text = m.text();
    if (/panicked|PanicHookInfo|unreachable executed|RuntimeError/.test(text)) problems.push(`panic: ${text.slice(0, 400)}`);
    else if (m.type() === 'error' && !ignoredConsole.test(text)) problems.push(`console error: ${text.slice(0, 200)}`);
  });
  await page.goto(`http://127.0.0.1:${server.address().port}/index.html?smoke=1${only}`);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  const blank = {};
  let lastSeq = null, report = null;
  const started = Date.now();
  while (Date.now() - started < limitMs && !problems.some(p => /panic|crashed|pageerror/.test(p))) {
    const request = await page.evaluate(() => window.__ume_smoke_shot || null).catch(() => null);
    if (request && JSON.parse(request).seq !== lastSeq) {
      const { seq, file } = JSON.parse(request);
      lastSeq = seq;
      const png = await page.screenshot();
      fs.writeFileSync(path.join(out, file), png);
      blank[file] = await analyzeShot(page, png);
      await page.evaluate(s => { window.__ume_smoke_ack = s; }, seq);
    }
    const raw = await page.evaluate(() => window.__ume_smoke_report || null).catch(() => null);
    if (raw) { report = JSON.parse(raw); if (report.done) break; }
    await sleep(100);
  }
  await browser.close(); server.close();
  if (!report) report = { schema: 1, platform: 'web', done: false, features: [] };
  report.platform = mobile ? 'web-mobile' : 'web';
  for (const feature of report.features) for (const check of feature.checks) {
    const stats = check.shot && blank[check.shot];
    if (!stats) continue;
    check.detail = stats.describe;
    if (stats.blank) { check.ok = false; check.detail = `blank or flat frame: ${stats.describe}`; }
    feature.ok = feature.checks.every(c => c.ok);
  }
  if (!report.done) problems.push(`runner did not finish (last feature: ${report.running || 'none started'})`);
  if (problems.length) report.panic = problems.join(' | ');
  report.passed = report.done && !problems.length && report.features.every(f => f.ok);
  fs.writeFileSync(path.join(out, 'report.json'), JSON.stringify(report, null, 2));
  fs.writeFileSync(path.join(out, 'report.md'), markdown(report));
  console.log(markdown(report));
  process.exit(report.passed ? 0 : 1);
})().catch(e => { console.error('SMOKE DRIVER FAIL', e); process.exit(2); });
