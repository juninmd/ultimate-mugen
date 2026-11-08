const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.env.WEB_DIST || path.join(__dirname, '..', 'web', 'dist'));
const out = path.resolve(process.env.CAPTURE_OUT || 'capture');
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const plan = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const server = staticServer(dir, types);
(async () => {
  fs.mkdirSync(out, { recursive: true });
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, recordVideo: plan.video ? { dir: out, size: { width: 960, height: 540 } } : undefined });
  const page = await ctx.newPage();
  page.on('console', m => { if (process.env.CAPTURE_LOG) console.log(m.type(), m.text().slice(0, 200)); });
  page.on('pageerror', e => console.log('pageerror', e.message.slice(0, 300)));
  await page.goto(url);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  await page.waitForTimeout(12000);
  await page.locator('#glcanvas').focus();
  const frames = n => page.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const press = async (k, hold = 2) => { await page.keyboard.down(k); await frames(hold); await page.keyboard.up(k); await frames(3); };
  for (const step of plan.steps) {
    if (step.key) { await press(step.key, step.hold || 2); await page.waitForTimeout(step.wait ?? 900); }
    else if (step.hold) { await page.keyboard.down(step.hold); await page.waitForTimeout(step.ms); await page.keyboard.up(step.hold); }
    else if (step.wait) await page.waitForTimeout(step.wait);
    if (step.shot) await page.screenshot({ path: path.join(out, step.shot + '.png') });
    if (step.mark) console.log('mark', step.mark, Date.now());
  }
  const video = plan.video ? page.video() : null;
  await ctx.close();
  if (video) fs.renameSync(await video.path(), path.join(out, plan.video));
  await browser.close(); server.close();
})().catch(e => { console.error(e); process.exit(2); });
