const fs = require('node:fs');
const path = require('node:path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');

const dist = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const output = process.env.WEB_RESULTS_TOUCH_SHOTS;
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.json': 'application/json' };
const server = staticServer(dist, types);

(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, hasTouch: true });
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (/panicked|PanicHookInfo/.test(message.text())) errors.push(message.text()); });
  await page.goto(`http://127.0.0.1:${server.address().port}/index.html?e2e=1&e2e_results=1`);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await page.waitForTimeout(12000);
  await page.locator('#glcanvas').focus();
  await page.keyboard.down('Enter');
  await page.waitForTimeout(100);
  await page.keyboard.up('Enter');
  await page.waitForFunction(() => JSON.parse(window.__ume_state || 'null')?.screen === 'results', null, { timeout: 45000 }).catch(async error => {
    const state = await page.evaluate(() => window.__ume_state || 'unpublished');
    throw new Error(`Results screen did not open: ${state}; ${error.message}`);
  });
  await page.waitForTimeout(1200);
  if (output) {
    fs.mkdirSync(output, { recursive: true });
    await page.screenshot({ path: path.join(output, 'results-touch-before.png') });
  }
  await page.touchscreen.tap(1008, 408);
  await page.waitForFunction(() => JSON.parse(window.__ume_state || 'null')?.screen === 'menu', null, { timeout: 5000 });
  await page.waitForTimeout(1200);
  if (output) await page.screenshot({ path: path.join(output, 'results-touch-after.png') });
  await browser.close();
  server.close();
  if (errors.length) throw new Error(errors.join('\n'));
  console.log('WEB RESULTS TOUCH OK menu option tapped');
})().catch(error => {
  console.error('WEB RESULTS TOUCH FAILED', error);
  server.close();
  process.exit(1);
});
