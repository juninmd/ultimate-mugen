// Virtual pad smoke: real multi-touch (CDP) must move the stick, press buttons, navigate menus and go back.
// Fails when the pad ignores a finger. Usage: node scripts/web-touch-pad-smoke.js [web/dist] (build with ./scripts/build-web.sh).
const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium, devices } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const server = staticServer(dir, types);
const profile = devices[process.env.TOUCH_DEVICE || 'Pixel 7 landscape'];
const shotDir = process.env.WEB_SMOKE_SHOTS;
const MIN_CHANGE = 0.004;
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const ctx = await b.newContext({ ...profile });
  const p = await ctx.newPage();
  const cdp = await ctx.newCDPSession(p);
  const { width: W, height: H } = profile.viewport;
  const problems = [];
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => { if (/panicked|PanicHookInfo/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`); });
  if (shotDir) fs.mkdirSync(shotDir, { recursive: true });
  const wait = ms => p.waitForTimeout(ms);
  const frames = n => p.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const shot = async (name, clip) => {
    const buf = await p.screenshot(clip ? { clip } : {});
    if (shotDir && name) fs.writeFileSync(path.join(shotDir, `${name}.png`), buf);
    return buf;
  };
  const change = (a, b) => p.evaluate(async ([a, b]) => {
    const px = async s => { const bmp = await createImageBitmap(await (await fetch(`data:image/png;base64,${s}`)).blob()); const c = new OffscreenCanvas(bmp.width, bmp.height).getContext('2d'); c.drawImage(bmp, 0, 0); return c.getImageData(0, 0, bmp.width, bmp.height).data; };
    const [x, y] = [await px(a), await px(b)];
    let diff = 0;
    for (let i = 0; i < x.length; i += 4) diff += (Math.abs(x[i] - y[i]) + Math.abs(x[i + 1] - y[i + 1]) + Math.abs(x[i + 2] - y[i + 2])) / 765;
    return diff / (x.length / 4);
  }, [a.toString('base64'), b.toString('base64')]);
  const touch = (type, pts) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map(([x, y], i) => ({ x, y, id: i + 1 })) });
  const tap = async (x, y, hold = 120) => { await touch('touchStart', [[x, y]]); await wait(hold); await touch('touchEnd', []); await wait(900); };
  const expectChange = async (label, before, after) => {
    const c = await change(before, after);
    if (c < MIN_CHANGE) problems.push(`${label}: the screen did not react to touch (change ${c.toFixed(4)})`);
    else console.log(`ok ${label} (change ${c.toFixed(4)})`);
  };
  const key = async (k, w = 1000) => { await p.keyboard.down(k); await frames(2); await p.keyboard.up(k); await frames(5); await wait(w); };

  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  await wait(12000);

  const title = await shot('00-title');
  await tap(W / 2, H / 2);
  const menu = await shot('01-menu');
  await expectChange('tap on the title opens the menu', title, menu);

  await tap(W * 0.195, H * 0.438, 120);
  const select = await shot('02-select');
  await expectChange('tapping a menu item opens its screen', menu, select);

  const back = [W - Math.min(44, Math.max(20, H * 0.075)) * 1.4 - Math.min(44, Math.max(20, H * 0.075)) * 2.1, H * 0.62];
  await tap(back[0] + Math.min(44, H * 0.075) * 2.1, back[1] - 12);
  const afterBack = await shot('03-back');
  await expectChange('the BACK pad button leaves the screen', select, afterBack);

  await p.reload();
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  await wait(12000);
  await p.locator('#glcanvas').focus();
  await key('Enter');
  for (let i = 0; i < 3; i++) await key('ArrowDown');
  await key('Enter', 1500); await key('Enter', 1500); await key('Enter', 1500);
  for (let attempt = 0; attempt < 4; attempt++) await key('Enter', 500);
  await wait(8000);

  const r = Math.min(44, Math.max(20, H * 0.075)), gap = r * 2.1, bx = W - r * 1.4 - gap * 2, by = H - r * 2.3;
  const pause = [W / 2, H * 0.19];
  await tap(pause[0], pause[1]);
  const paused = await shot('04-paused');
  const stickClip = { x: 0, y: Math.round(H * 0.55), width: Math.round(W * 0.2), height: Math.round(H * 0.45) };
  const idleStick = await shot(null, stickClip);
  await touch('touchStart', [[W * 0.095, H * 0.77]]); await wait(100);
  await touch('touchMove', [[W * 0.155, H * 0.77]]); await wait(500);
  const heldStick = await shot('05-stick-held', stickClip);
  await touch('touchEnd', []);
  await expectChange('the stick follows a held finger', idleStick, heldStick);

  const idlePad = await shot(null, { x: Math.round(bx - r * 1.2), y: Math.round(by - gap - r * 1.2), width: Math.round(r * 2.4), height: Math.round(r * 2.4) });
  await touch('touchStart', [[bx, by - gap]]); await wait(150);
  const pressedPad = await shot('06-button-held', { x: Math.round(bx - r * 1.2), y: Math.round(by - gap - r * 1.2), width: Math.round(r * 2.4), height: Math.round(r * 2.4) });
  await touch('touchEnd', []);
  await expectChange('a face button lights up while touched', idlePad, pressedPad);

  await touch('touchStart', [[W * 0.095, H * 0.77], [bx, by - gap]]); await wait(100);
  await touch('touchMove', [[W * 0.155, H * 0.77], [bx, by - gap]]); await wait(400);
  const both = await shot('07-stick-and-button', stickClip);
  await touch('touchEnd', []);
  await expectChange('stick and button work with two fingers', idleStick, both);

  await tap(W / 2, H * 0.49);
  await wait(1500);
  const resumed = await shot('08-resumed');
  await expectChange('Continue in the pause menu resumes the fight', paused, resumed);

  if (problems.length) console.error(problems.join('\n'));
  console.log(problems.length ? 'WEB TOUCH PAD SMOKE FAILED' : 'WEB TOUCH PAD SMOKE OK');
  await b.close(); server.close();
  process.exit(problems.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
