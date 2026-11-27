const fs = require('node:fs');
const path = require('node:path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');

const dist = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const output = process.env.WEB_ROUTE_STORYBOARD_SHOTS;
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.json': 'application/json' };
const server = staticServer(dist, types);

(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, hasTouch: true, recordVideo: output ? { dir: output, size: { width: 1280, height: 720 } } : undefined });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (/panicked|PanicHookInfo/.test(message.text())) errors.push(message.text()); });
  await page.goto(`http://127.0.0.1:${server.address().port}/index.html?e2e=1&e2e_arcade_storyboard=1`);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await page.waitForTimeout(5000);
  const frames = count => page.evaluate(count => new Promise(resolve => {
    const step = remaining => remaining ? requestAnimationFrame(() => step(remaining - 1)) : resolve();
    step(count);
  }), count);
  await page.keyboard.down('Enter');
  await frames(2);
  await page.keyboard.up('Enter');
  await frames(5);
  await page.waitForFunction(() => {
    const state = JSON.parse(window.__ume_state || 'null');
    return state?.arcade?.storyboard === true && state.arcade.scene === 0;
  }, null, { timeout: 45000 }).catch(async error => {
    const state = await page.evaluate(() => window.__ume_state || 'unpublished');
    throw new Error(`Storyboard did not start: ${state}; ${error.message}`);
  });
  await page.waitForTimeout(1500);
  if (output) {
    fs.mkdirSync(output, { recursive: true });
    await page.screenshot({ path: path.join(output, 'arcade-route-storyboard.png') });
  }
  await page.touchscreen.tap(640, 360);
  await page.waitForFunction(() => {
    const state = JSON.parse(window.__ume_state || 'null');
    return state?.arcade?.storyboard === true && state.arcade.scene === 1;
  }, null, { timeout: 5000 });
  if (output) await page.screenshot({ path: path.join(output, 'arcade-route-storyboard-touch-advanced.png') });
  await page.locator('#glcanvas').focus();
  for (let scene = 2; scene < 5; scene += 1) {
    await page.keyboard.down('Enter');
    await page.waitForTimeout(120);
    await page.keyboard.up('Enter');
    await page.waitForFunction(expected => {
      const state = JSON.parse(window.__ume_state || 'null');
      return state?.arcade?.storyboard === true && state.arcade.scene === expected;
    }, scene, { timeout: 5000 });
  }
  await page.waitForTimeout(500);
  if (output) await page.screenshot({ path: path.join(output, 'arcade-route-storyboard-scene-4.png') });
  await page.touchscreen.tap(1060, 20);
  await page.waitForFunction(() => {
    const state = JSON.parse(window.__ume_state || 'null');
    return state?.screen === 'vs' && state.arcade?.stage === 'native:brazil' && state.arcade.fight === 1;
  }, null, { timeout: 30000 });
  await page.waitForFunction(() => {
    const state = JSON.parse(window.__ume_state || 'null');
    return state?.screen === 'fight' && state.clock?.round_state === 2 && state.arcade?.stage === 'native:brazil';
  }, null, { timeout: 90000 });
  await page.waitForFunction(() => JSON.parse(window.__ume_state || 'null')?.clock?.tick >= 120, null, { timeout: 20000 });
  if (output) await page.screenshot({ path: path.join(output, 'arcade-route-brazil.png') });
  const state = await page.evaluate(() => JSON.parse(window.__ume_state));
  await context.close();
  await browser.close();
  server.close();
  if (errors.length) throw new Error(errors.join('\n'));
  console.log(`WEB ROUTE STORYBOARD OK scenes=0..4 tap_advanced=1 skipped_by_tap=1 fight=${state.arcade.fight} stage=${state.arcade.stage}`);
})().catch(error => {
  console.error('WEB ROUTE STORYBOARD FAILED', error);
  server.close();
  process.exit(1);
});
