const { chromium } = require('playwright');
const { staticServer } = require('./web-smoke-server');
const path = require('node:path');
const fs = require('node:fs');
const captureDirectory = process.env.STORY_FLOW_SHOTS;
if (captureDirectory) fs.mkdirSync(captureDirectory, { recursive: true });
const server = staticServer(process.argv[2] || 'web/dist', { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.png': 'image/png' });
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
  try {
    const page = await browser.newPage({viewport: {width: 1280, height: 720}});
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', msg => { if (/panicked|PanicHookInfo/.test(msg.text())) errors.push(msg.text()); });
    const state = () => page.evaluate(() => JSON.parse(window.__ume_state));
    const wait = (screen, epilogue) => page.waitForFunction(([screen, epilogue]) => {
      const state = JSON.parse(window.__ume_state || 'null');
      return state?.screen === screen && (epilogue == null || state.epilogue === epilogue);
    }, [screen, epilogue], {timeout: 90000});
    const key = async (name) => {
      await page.keyboard.down(name);
      await page.evaluate(() => new Promise(done => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(done, 120)))));
      await page.keyboard.up(name);
      await page.waitForTimeout(600);
    };
    const shot = name => captureDirectory && page.screenshot({path: path.join(captureDirectory, name + '.png')});
    const boot = async query => {
      await page.goto('http://127.0.0.1:' + server.address().port + '/index.html?e2e=1' + query);
      await wait('title');
      await page.locator('#glcanvas').focus();
      await key('Enter');
      await wait('menu');
    };
    await boot('&e2e_prefight=1');
    await key('Enter');
    await wait('select');
    await key('Enter');
    await wait('story');
    for (let pageIndex = 0; pageIndex < 4; pageIndex++) await key('Enter');
    await shot('00-mercy-options');
    await page.mouse.move(384, 510);
    await page.mouse.down();
    await page.waitForTimeout(200);
    await page.mouse.up();
    await page.waitForTimeout(900);
    await shot('00-mercy-accepted');
    await boot('&e2e_epilogue=1');
    await key('Enter');
    await wait('select');
    await key('Enter');
    await wait('story', 0);
    await page.waitForTimeout(5500);
    await shot('01-epilogue-rooftop');
    const before = (await state()).gpu;
    await key('Space');
    await wait('story', 1);
    await page.waitForTimeout(4500);
    await shot('02-epilogue-reply');
    const after = (await state()).gpu;
    console.log('Epilogue page transition GPU counters', { before, after });
    if (before.resets !== after.resets) throw new Error('Epilogue page transition reset the RGBA atlas');
    await key('Escape');
    await page.waitForTimeout(1200);
    await shot('03-story-clear');
    await key('Enter');
    await wait('menu');
    await boot('');
    await key('ArrowDown');
    await key('Enter');
    await wait('select');
    await key('Enter');
    await wait('cutscene');
    await page.waitForTimeout(4500);
    await shot('04-arcade-prologue');
    await key('Escape');
    await wait('stages');
    await shot('05-arcade-stage-select');
    for (const [count, boss] of ['oboro', 'garan', 'valdric'].entries()) {
      await boot('&e2e_arcade_clear=' + count);
      await key('ArrowDown');
      await key('Enter');
      await wait('select');
      await key('Enter');
      await wait('cutscene');
      if (count === 0) await key('Escape');
      await page.waitForTimeout(4500);
      await shot('06-arcade-ending-' + boss);
      await key('Escape');
      await wait('menu');
    }
    if (errors.length) throw new Error(errors.join('\n'));
    console.log('STORY AND ARCADE FLOW OK');
  } finally {
    await browser.close();
    server.close();
  }
})().catch(e => { console.error(e); process.exitCode = 1; });
