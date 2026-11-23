// Touch pad probe smoke: real multi-touch (CDP) in a real fight. Reads the pad state the engine publishes to
// localStorage (ume.touchdebug.out = "bits pressedMask extrasOpen") and fails when a finger does not do what it should.
// Usage: node scripts/web-touch-probe-smoke.js [web/dist] (build with ./scripts/build-web.sh).
const path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium, devices } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const server = staticServer(dir, types);
const profile = devices[process.env.TOUCH_DEVICE || 'Pixel 7 landscape'];
const K = { U: 1, D: 2, L: 4, R: 8, A: 16, B: 32, X: 128, Y: 256, D2: 2048, W: 4096, T: 16384, S: 1024 };
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const ctx = await b.newContext({ ...profile });
  await ctx.addInitScript(() => { try { localStorage.setItem('ume.touchdebug', '1'); } catch (e) {} });
  const p = await ctx.newPage();
  const cdp = await ctx.newCDPSession(p);
  const { width: W, height: H } = profile.viewport;
  const problems = [];
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => { if (/panicked|PanicHookInfo/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`); });
  const wait = ms => p.waitForTimeout(ms);
  const frames = n => p.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const key = async (k, w = 1000) => { await p.keyboard.down(k); await frames(2); await p.keyboard.up(k); await frames(5); await wait(w); };
  const touch = (type, pts) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map(([x, y, id], i) => ({ x, y, id: id ?? i + 1 })) });
  const lift = ended => touch('touchEnd', ended);
  const probe = async () => { const [bits, mask, open] = (await p.evaluate(() => localStorage.getItem('ume.touchdebug.out') || '0 0 0')).split(' ').map(Number); return { bits, mask, open }; };
  const expect = async (label, test) => { await wait(250); const s = await probe(); if (test(s)) console.log(`ok ${label} ${JSON.stringify(s)}`); else problems.push(`${label}: ${JSON.stringify(s)}`); };

  await p.goto(url);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  await wait(12000);
  await p.locator('#glcanvas').focus();
  await key('Enter');
  for (let i = 0; i < 3; i++) await key('ArrowDown');
  await key('Enter', 1500); await key('Enter', 1500); await key('Enter', 1500);
  for (let attempt = 0; attempt < 4; attempt++) await key('Enter', 500);
  await wait(9000);
  await touch('touchStart', [[W * 0.5, H * 0.5]]); await touch('touchEnd', []); await wait(300);

  const r = Math.min(44, Math.max(20, H * 0.075)), gap = r * 2.1, small = r * 0.72, sg = small * 2.3;
  const bx = W - r * 1.4 - gap * 2, by = H - r * 2.3;
  const core = [[bx, by - gap], [bx + gap, by - gap], [bx + 2 * gap, by - gap], [bx, by], [bx + gap, by], [bx + 2 * gap, by]];
  const extra = n => [bx - gap * 0.85 - (3 - (n % 4)) * sg, by - Math.floor(n / 4) * sg];
  const toggle = [bx + 2 * gap, by - 2 * gap + r * 0.2];
  const home = [W * 0.12, H * 0.72];
  const off = (d) => [home[0] + d[0] * r * 1.6, home[1] + d[1] * r * 1.6];

  await touch('touchStart', [[...home]]); await wait(100);
  for (const [name, d, want] of [['left', [-1, 0], K.L], ['right', [1, 0], K.R], ['up', [0, -1], K.U], ['down', [0, 1], K.D], ['down-left', [-1, 1], K.D | K.L]]) {
    await touch('touchMove', [off(d)]);
    await expect(`the left stick steers ${name}`, s => (s.bits & want) === want);
  }
  await touch('touchEnd', []);
  await expect('releasing the stick clears the direction', s => (s.bits & 15) === 0);

  const coreBits = [K.X, K.Y, K.D2, K.A, K.B, K.W];
  for (let i = 0; i < 6; i++) {
    await touch('touchStart', [core[i]]);
    await expect(`core button ${i} presses`, s => (s.bits & coreBits[i]) === coreBits[i]);
    await touch('touchEnd', []);
    await expect(`core button ${i} releases`, s => (s.bits & coreBits[i]) === 0);
  }

  await touch('touchStart', [[...home]]); await wait(100);
  await touch('touchMove', [off([1, 0])]); await wait(150);
  for (let i = 0; i < 6; i++) {
    await touch('touchStart', [off([1, 0]), [...core[i], 2]]);
    await expect(`button ${i} presses while the stick holds right`, s => (s.bits & coreBits[i]) === coreBits[i] && (s.bits & K.R) !== 0);
    await lift([[...core[i], 2]]);
    await expect(`button ${i} released, stick still right`, s => (s.bits & coreBits[i]) === 0 && (s.bits & K.R) !== 0);
  }
  await touch('touchEnd', []);

  await touch('touchStart', [[...toggle]]); await touch('touchEnd', []);
  await expect('the + button opens the side keys', s => s.open === 1);
  for (let n = 0; n < 11; n++) {
    await touch('touchStart', [extra(n)]);
    await expect(`side key ${n} presses`, s => (s.mask >> (6 + n)) & 1);
    await touch('touchEnd', []);
    await expect(`side key ${n} releases`, s => ((s.mask >> (6 + n)) & 1) === 0);
  }
  await touch('touchStart', [[...home]]); await wait(100);
  await touch('touchMove', [off([-1, 0])]); await wait(150);
  for (const n of [1, 4, 7]) {
    await touch('touchStart', [off([-1, 0]), [...extra(n), 2]]);
    await expect(`side key ${n} presses while the stick holds left`, s => ((s.mask >> (6 + n)) & 1) && (s.bits & K.L) !== 0);
    await lift([[...extra(n), 2]]);
  }
  await touch('touchEnd', []);
  await touch('touchStart', [[...toggle]]); await touch('touchEnd', []);
  await expect('the x button closes the side keys', s => s.open === 0);

  const shotDir = process.env.WEB_SMOKE_SHOTS;
  const fs = require('fs');
  if (shotDir) fs.mkdirSync(shotDir, { recursive: true });
  const shot = async (name, clip) => { const buf = await p.screenshot(clip ? { clip } : {}); if (shotDir && name) fs.writeFileSync(path.join(shotDir, `${name}.png`), buf); return buf; };
  const change = (a, c) => p.evaluate(async ([a, c]) => {
    const px = async s => { const bmp = await createImageBitmap(await (await fetch(`data:image/png;base64,${s}`)).blob()); const cv = new OffscreenCanvas(bmp.width, bmp.height).getContext('2d'); cv.drawImage(bmp, 0, 0); return cv.getImageData(0, 0, bmp.width, bmp.height).data; };
    const [x, y] = [await px(a), await px(c)];
    let diff = 0;
    for (let i = 0; i < x.length; i += 4) diff += (Math.abs(x[i] - y[i]) + Math.abs(x[i + 1] - y[i + 1]) + Math.abs(x[i + 2] - y[i + 2])) / 765;
    return diff / (x.length / 4);
  }, [a.toString('base64'), c.toString('base64')]);
  const tapAt = async ([x, y]) => { await touch('touchStart', [[x, y]]); await wait(120); await touch('touchEnd', []); await wait(700); };
  const reacts = async (label, act, clip, min = 0.004) => {
    const before = await shot(null, clip); await act(); const after = await shot(null, clip);
    const c = await change(before, after);
    if (c < min) problems.push(`${label}: the screen did not react (change ${c.toFixed(4)})`); else console.log(`ok ${label} (change ${c.toFixed(4)})`);
  };
  const sr = r * 2, nr = r;
  const arrow = d => [2.8 * r + d[0] * nr * 1.7, H - 3.1 * r + d[1] * nr * 1.7];
  const okBtn = core[5], backBtn = core[2];
  const full = { x: 0, y: 0, width: W, height: H };
  const list = { x: Math.round(W * 0.3), y: Math.round(H * 0.42), width: Math.round(W * 0.4), height: Math.round(H * 0.38) };
  const slots = { x: Math.round(W * 0.14), y: Math.round(H * 0.24), width: Math.round(W * 0.72), height: Math.round(H * 0.14) };
  const paused = await shot(null, full);
  await tapAt([W / 2, H * 0.16]);
  const pausedShot = await shot('10-pause-with-nav', full);
  const pc = await change(paused, pausedShot);
  if (pc < 0.05) problems.push(`the pause button did not open the pause menu (change ${pc.toFixed(4)})`);
  await reacts('the pad down arrow moves the pause selection', () => tapAt(arrow([0, 1])), list);
  await reacts('the pad up arrow moves it back', () => tapAt(arrow([0, -1])), list);
  await reacts('the pad right arrow switches to the Moves tab', () => tapAt(arrow([1, 0])), full, 0.02);
  await shot('11-pause-moves', full);
  await reacts('OK binds the selected move to a side key slot', () => tapAt(okBtn), slots);
  await shot('12-pause-bound', full);
  await reacts('OK again on the same move clears the side key slot', () => tapAt(okBtn), slots);
  await reacts('the pad down arrow scrolls the move list', () => tapAt(arrow([0, 1])), full, 0.004);
  await reacts('BACK resumes the fight', () => tapAt(backBtn), full, 0.05);
  await shot('13-resumed', full);
  await touch('touchStart', [[...home]]); await wait(100);
  await touch('touchMove', [off([-1, 0])]);
  await expect('the stick steers again after the pause', s => (s.bits & K.L) !== 0);
  await touch('touchEnd', []);

  if (problems.length) console.error(problems.join('\n'));
  console.log(problems.length ? 'WEB TOUCH PROBE SMOKE FAILED' : 'WEB TOUCH PROBE SMOKE OK');
  await b.close(); server.close();
  process.exit(problems.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
