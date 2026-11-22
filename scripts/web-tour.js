const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium, devices } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const out = path.resolve(process.argv[3] || 'tour');
const mobile = process.env.TOUR_MOBILE === '1';
const lang = process.env.TOUR_LANG || '';
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip', '.png': 'image/png' };
const server = staticServer(dir, types);

(async () => {
  fs.mkdirSync(out, { recursive: true });
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const phone = { ...devices['Pixel 7 landscape'] };
  const ctx = await browser.newContext({ ...(mobile ? phone : { viewport: { width: 1280, height: 720 } }), locale: lang || 'en-US' });
  const page = await ctx.newPage();
  const problems = [];
  page.on('pageerror', e => problems.push(`pageerror ${e.message.slice(0, 200)}`));
  page.on('console', m => { if (/panicked|RuntimeError/.test(m.text())) problems.push(m.text().slice(0, 300)); });
  const t0 = Date.now();
  const shot = async name => { await page.screenshot({ path: path.join(out, `${name}.png`) }); console.log('shot', name, `+${Date.now() - t0}ms`, await state()); };
  const state = () => page.evaluate(() => { const s = JSON.parse(window.__ume_state || 'null'); return s ? s.screen : '-'; }).catch(() => '?');
  await page.goto(`http://127.0.0.1:${server.address().port}/index.html?e2e=1`);
  for (const ms of [400, 1500, 3000]) { await page.waitForTimeout(ms); await shot(`00-boot-${ms}`); }
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  await page.waitForTimeout(8000);
  await shot('01-title');
  await page.locator('#glcanvas').focus();
  const frames = n => page.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const key = async (k, wait = 900) => { await page.keyboard.down(k); await frames(2); await page.keyboard.up(k); await frames(3); await page.waitForTimeout(wait); };
  await key('Enter', 1500);
  await shot('02-menu');
  const menu = ['story', 'arcade', 'survival', 'versus', 'training', 'mugen', 'market', 'settings', 'replays', 'player-card', 'power-arena', 'tower'];
  for (let i = 1; i < menu.length; i++) {
    await key('ArrowDown', 500);
    if (['mugen', 'market', 'settings', 'replays', 'player-card'].includes(menu[i])) {
      await key('Enter', 1500);
      await shot(`03-${String(i).padStart(2, '0')}-${menu[i]}`);
      if (menu[i] === 'settings') for (let t = 1; t < 5; t++) { await key('ArrowRight', 600); await shot(`03-settings-tab${t}`); }
      await key('Escape', 1200);
      if ((await state()) !== 'menu') { await key('Backspace', 1200); }
    }
  }
  for (let i = menu.length - 1; i > 3; i--) await key('ArrowUp', 300);
  await shot('04-menu-versus');
  await key('Enter', 2000);
  await shot('05-select');
  for (let i = 0; i < 3; i++) await key('ArrowRight', 300);
  await shot('05-select-moved');
  await key('Enter', 1500);
  await shot('06-select-p2');
  await key('Enter', 1500);
  await shot('07-stages');
  await key('ArrowRight', 800);
  await shot('07-stages-next');
  await key('Enter', 100);
  for (let i = 0; i < 8; i++) { await page.waitForTimeout(450); await shot(`08-vs-${i}`); }
  await page.waitForFunction(() => JSON.parse(window.__ume_state || 'null')?.screen === 'fight', null, { timeout: 90000 }).catch(() => problems.push('fight never started'));
  for (let i = 0; i < 4; i++) { await page.waitForTimeout(700); await shot(`09-intro-${i}`); }
  await page.keyboard.down('ArrowRight'); await page.waitForTimeout(700); await page.keyboard.up('ArrowRight');
  for (const k of ['KeyA', 'KeyS', 'KeyD', 'KeyQ', 'KeyW', 'KeyE']) { await key(k, 120); }
  await shot('10-fight');
  await page.waitForTimeout(3000);
  await shot('10-fight-later');
  await key('Escape', 1000);
  await shot('11-pause');
  await key('ArrowDown', 500); await shot('11-pause-2');
  await key('Escape', 1000);
  await shot('12-resumed');
  await browser.close(); server.close();
  if (problems.length) { console.error('TOUR PROBLEMS\n' + problems.join('\n')); process.exit(1); }
  console.log('TOUR OK');
})().catch(e => { console.error('TOUR FAIL', e); process.exit(2); });
