const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');
const { staticServer } = require('./web-smoke-server');

const saveKey = 'ume.story-checkpoint.json';
const out = process.env.STORY_CHECKPOINT_SHOTS || process.env.WEB_SMOKE_SHOTS;
const server = staticServer(process.argv[2] || 'web/dist', {
  '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.png': 'image/png',
});

(async () => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const url = 'http://127.0.0.1:' + server.address().port + '/index.html?e2e=1';
  const browser = await chromium.launch({
    args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
  });
  try {
    if (out) fs.mkdirSync(out, { recursive: true });
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => {
      if (/panicked|PanicHookInfo/.test(message.text())) errors.push(message.text());
    });
    const state = () => page.evaluate(() => JSON.parse(window.__ume_state || 'null'));
    const wait = screen => page.waitForFunction(expected =>
      JSON.parse(window.__ume_state || 'null')?.screen === expected, screen, { timeout: 90000 });
    const key = async name => {
      await page.keyboard.down(name);
      await page.evaluate(() => new Promise(done =>
        requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(done, 120)))));
      await page.keyboard.up(name);
      await page.waitForTimeout(600);
    };
    const shot = name => out && page.screenshot({ path: path.join(out, name + '.png') });
    const boot = async (query = '') => {
      await page.goto(url + query);
      await wait('title');
      await page.locator('#glcanvas').focus();
      await key('Enter');
      await wait('menu');
      await key('Enter');
    };

    await boot('&e2e_story_checkpoint=1');
    await wait('select');
    await key('Enter');
    await page.waitForFunction(key => Boolean(localStorage.getItem(key)), saveKey);
    const saved = await page.evaluate(key => JSON.parse(localStorage.getItem(key)), saveKey);
    assert.equal(saved.stage, 1);
    assert.ok(saved.score > 0);
    assert.ok(saved.fighter);
    await shot('01-stage-cleared');

    await boot();
    await wait('story_checkpoint');
    await shot('02-checkpoint-menu');
    await key('Enter');
    await wait('story');
    const restored = (await state()).story;
    assert.deepEqual(restored, {
      stage: saved.stage, wave: 0, credits: saved.credits, score: saved.score, phase: 'Intro',
      mercy: null, mercy_pending: false, helper_pending: false, helper_cursor: 0, helper_chosen: false,
    });
    await shot('03-resumed-stage');

    await key('Escape');
    await wait('menu');
    await key('Enter');
    await wait('story_checkpoint');
    await key('ArrowRight');
    await key('Enter');
    await wait('select');
    assert.deepEqual(await page.evaluate(key => JSON.parse(localStorage.getItem(key)), saveKey), saved);
    assert.deepEqual(errors, []);
    console.log('WEB STORY CHECKPOINT OK', saved);
  } finally {
    await browser.close();
    server.close();
  }
})().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
