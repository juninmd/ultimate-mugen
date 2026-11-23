const fs = require('node:fs');
const path = require('node:path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');

const dist = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.json': 'application/json' };
const server = staticServer(dist, types);

(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  const errors = [];
  const consoleTail = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => {
    consoleTail.push(`${message.type()}: ${message.text().slice(0, 180)}`);
    if (consoleTail.length > 20) consoleTail.shift();
    if (/panicked|PanicHookInfo/.test(message.text())) errors.push(message.text());
  });
  const url = `http://127.0.0.1:${server.address().port}/index.html?e2e=1&e2e_arcade_route=1`;
  await page.goto(url);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await page.waitForTimeout(5000);
  await page.locator('#glcanvas').focus();
  const frames = count => page.evaluate(count => new Promise(resolve => {
    const step = remaining => remaining ? requestAnimationFrame(() => step(remaining - 1)) : resolve();
    step(count);
  }), count);
  await page.keyboard.down('Enter');
  await frames(2);
  await page.keyboard.up('Enter');
  await frames(5);
  const arrived = await page.waitForFunction(() => {
    const state = JSON.parse(window.__ume_state || 'null');
    return state?.screen === 'vs' && state.arcade?.stage === 'native:brazil' && state.arcade.fight === 1;
  }, null, { timeout: 30000 }).then(() => true).catch(() => false);
  if (!arrived) {
    const state = await page.evaluate(() => window.__ume_state || 'unpublished');
    const debug = process.env.WEB_ROUTE_STAGE_SHOTS;
    if (debug) {
      fs.mkdirSync(debug, { recursive: true });
      await page.screenshot({ path: path.join(debug, 'arcade-route-failure.png') });
    }
    throw new Error(`Expected route VS, got ${state}; console: ${consoleTail.join(' | ')}`);
  }
  await page.waitForFunction(() => {
    const state = JSON.parse(window.__ume_state || 'null');
    return state?.screen === 'fight' && state.clock?.round_state === 2 && state.arcade?.stage === 'native:brazil';
  }, null, { timeout: 90000 });
  await page.waitForFunction(() => JSON.parse(window.__ume_state || 'null')?.clock?.tick >= 120, null, { timeout: 20000 });
  const output = process.env.WEB_ROUTE_STAGE_SHOTS;
  if (output) {
    fs.mkdirSync(output, { recursive: true });
    await page.screenshot({ path: path.join(output, 'arcade-route-brazil.png') });
  }
  const state = await page.evaluate(() => JSON.parse(window.__ume_state));
  await browser.close();
  server.close();
  if (errors.length) throw new Error(errors.join('\n'));
  console.log(`WEB ROUTE STAGE OK fight=${state.arcade.fight} stage=${state.arcade.stage}`);
})().catch(error => {
  console.error('WEB ROUTE STAGE FAILED', error);
  server.close();
  process.exit(1);
});
