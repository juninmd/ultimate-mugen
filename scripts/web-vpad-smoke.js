const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium, devices } = require('playwright');

const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const deviceName = process.env.VPAD_DEVICE || 'Pixel 7 landscape';
const shotDir = process.env.VPAD_SHOTS || process.env.WEB_SMOKE_SHOTS;
const tag = deviceName.toLowerCase().replace(/[^a-z0-9]+/g, '-');
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.zip': 'application/zip' };
const TRAINING = 4;
const K = { U: 1, D: 2, L: 4, R: 8, A: 16, X: 128 };
const server = staticServer(dir, types);
const checks = [];

(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader', '--autoplay-policy=no-user-gesture-required'] });
  const ctx = await browser.newContext({ ...devices[deviceName] });
  const page = await ctx.newPage();
  const cdp = await ctx.newCDPSession(page);
  const problems = [];
  page.on('pageerror', e => problems.push(`pageerror: ${e.message}`));
  page.on('console', m => { if (/panicked|PanicHookInfo|unreachable executed/.test(m.text())) problems.push(`panic: ${m.text().slice(0, 200)}`); });
  if (shotDir) fs.mkdirSync(shotDir, { recursive: true });

  const wait = ms => page.waitForTimeout(ms);
  const state = () => page.evaluate(() => { try { return JSON.parse(window.__ume_state); } catch { return null; } });
  const pad = () => page.evaluate(() => window.__vpad.state());
  const p1 = async () => (await state()).fighters.find(f => f.side === 0);
  const ok = (name, cond, detail = '') => { checks.push(`${cond ? 'PASS' : 'FAIL'} ${name}${detail ? ` (${detail})` : ''}`); if (!cond) throw new Error(`${name} ${detail}`); };
  const until = async (name, fn, ms = 20000) => {
    const end = Date.now() + ms;
    while (Date.now() < end) { const v = await fn(); if (v) return v; await wait(100); }
    throw new Error(`timeout: ${name}; state=${JSON.stringify(await state())?.slice(0, 300)}`);
  };
  const shot = async name => { if (shotDir) await page.screenshot({ path: path.join(shotDir, `${tag}-${name}.png`) }); };
  const touch = (type, pts) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map(([x, y, id]) => ({ x, y, id, radiusX: 6, radiusY: 6, force: 1 })) });
  const center = async sel => { const b = await page.locator(sel).first().boundingBox(); return [b.x + b.width / 2, b.y + b.height / 2, b.width / 2]; };
  const btn = label => `#vp-right .vp-hit:text-is("${label}")`;
  const frames = n => page.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const tapSel = async (sel) => { const [x, y] = await center(sel); await touch('touchStart', [[x, y, 1]]); await frames(2); await touch('touchEnd', []); await frames(2); };
  const stickAt = async (dx, dy) => { const [x, y, r] = await center('#vp-base'); return [x + dx * r, y + dy * r]; };
  const flick = async (dx, dy) => { const [x, y] = await stickAt(dx, dy); await touch('touchStart', [[x, y, 1]]); await frames(2); await touch('touchEnd', []); await frames(2); };

  const padVisible = async screen => {
    const r = await page.evaluate(() => {
      const root = document.getElementById('vpad');
      const rects = [...root.querySelectorAll('.vp-hit')].map(e => e.id === 'vp-stick' ? document.getElementById('vp-base').getBoundingClientRect() : e.getBoundingClientRect());
      const inside = rects.every(b => b.left >= 0 && b.top >= 0 && b.right <= innerWidth + 0.5 && b.bottom <= innerHeight + 0.5 && b.width >= 30);
      let overlap = 0;
      for (let i = 0; i < rects.length; i++) for (let j = i + 1; j < rects.length; j++) {
        const a = rects[i], b = rects[j];
        if (a.left < b.right - 1 && b.left < a.right - 1 && a.top < b.bottom - 1 && b.top < a.bottom - 1) overlap++;
      }
      const shown = getComputedStyle(root).display !== 'none' && root.classList.contains('on');
      const top = rects.every(b => { const e = document.elementFromPoint(b.left + b.width / 2, b.top + b.height / 2); return e && e.closest('#vpad'); });
      return { shown, inside, overlap, top, count: rects.length };
    });
    ok(`pad visible on ${screen}`, r.shown && r.inside && r.overlap === 0 && r.top && r.count === 13, JSON.stringify(r));
    await shot(screen);
  };

  await page.goto(`http://127.0.0.1:${server.address().port}/index.html?e2e=1`);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 90000 }).catch(() => {});
  await until('title screen', async () => (await state())?.screen === 'title', 60000);
  await wait(1500);
  await padVisible('title');

  await tapSel(btn('OK'));
  await until('menu after OK', async () => (await state()).screen === 'menu');
  await wait(800);
  await padVisible('menu');
  const c0 = (await state()).menu_cur;
  await flick(0, 0.9);
  await until('menu cursor down', async () => (await state()).menu_cur === (c0 + 1) % 13, 5000).catch(() => {});
  const c1 = (await state()).menu_cur;
  ok('stick down moves the menu cursor', c1 === (c0 + 1) % 13, `${c0} -> ${c1}`);
  await flick(0, -0.9);
  await until('menu cursor back', async () => (await state()).menu_cur === c0, 5000).catch(() => {});
  let back = (await state()).menu_cur;
  if (back !== c0 && back === (c0 - 1 + 13) % 13) { await flick(0, 0.9); await until('menu cursor recover', async () => (await state()).menu_cur === c0, 5000).catch(() => {}); back = (await state()).menu_cur; }
  ok('stick up moves the menu cursor back', back === c0, `${c1} -> ${back}`);
  for (let i = 0; i < 16 && (await state()).menu_cur !== TRAINING; i++) await flick(0, (await state()).menu_cur < TRAINING ? 0.9 : -0.9);
  ok('menu reaches training by touch', (await state()).menu_cur === TRAINING);
  await tapSel(btn('OK'));
  await until('select screen', async () => (await state()).screen === 'select');
  await wait(800);
  await padVisible('select');
  await tapSel(btn('BACK'));
  await until('back to menu', async () => (await state()).screen === 'menu');
  ok('BACK leaves the select screen', true);
  await wait(600);
  for (let i = 0; i < 16 && (await state()).menu_cur !== TRAINING; i++) await flick(0, (await state()).menu_cur < TRAINING ? 0.9 : -0.9);
  await tapSel(btn('OK'));
  await until('select again', async () => (await state()).screen === 'select');
  await wait(600);
  const seen = new Set(['select']);
  for (let i = 0; i < 20 && (await state()).screen !== 'fight'; i++) {
    const s = (await state()).screen;
    if (!seen.has(s)) { seen.add(s); await padVisible(s); }
    await tapSel(btn('OK'));
    await wait(500);
  }
  await until('fight screen', async () => (await state()).screen === 'fight', 60000);
  await until('round running', async () => (await state()).clock?.round_state === 2, 30000);
  await wait(500);
  await padVisible('fight');

  const [rx, ry] = await stickAt(0.9, 0);
  const [lx, ly] = await stickAt(-0.9, 0);
  let x0 = (await p1()).x;
  await touch('touchStart', [[rx, ry, 1]]);
  await wait(1200);
  await shot('fight-walk');
  let x1 = (await p1()).x;
  await touch('touchEnd', []);
  ok('stick right walks right', x1 > x0 + 10, `${x0} -> ${x1}`);
  await wait(400);
  x0 = (await p1()).x;
  await touch('touchStart', [[lx, ly, 1]]);
  await wait(1200);
  x1 = (await p1()).x;
  await touch('touchEnd', []);
  ok('stick left walks left', x1 < x0 - 10, `${x0} -> ${x1}`);
  await wait(500);

  const [ux, uy] = await stickAt(0, -0.9);
  await touch('touchStart', [[ux, uy, 1]]);
  let minY = 0;
  for (let i = 0; i < 12; i++) { await wait(50); minY = Math.min(minY, (await p1()).y); }
  await shot('fight-jump');
  await touch('touchEnd', []);
  ok('stick up jumps', minY < -20, `min y ${minY}`);
  await wait(1200);

  const [bx, by] = await center(btn('X'));
  const [ax, ay] = await center(btn('A'));
  await touch('touchStart', [[rx, ry, 1]]);
  await wait(300);
  await touch('touchMove', [[rx, ry, 1], [bx, by, 2], [ax, ay, 3]]);
  const both = await until('pad reports stick + X + A', async () => { const v = await pad(); return (v.bits & (K.R | K.X | K.A)) === (K.R | K.X | K.A) && v.fingers === 3 && v; }, 3000).catch(() => pad());
  let attacked = false, attackState = 0;
  for (let i = 0; i < 40 && !attacked; i++) { attackState = (await p1()).state; attacked = attackState >= 200 && attackState < 5000; if (!attacked) await wait(30); }
  await shot('fight-attack');
  ok('three fingers at once: stick + X + A', (both.bits & (K.R | K.X | K.A)) === (K.R | K.X | K.A) && both.fingers === 3, JSON.stringify(both));
  ok('attack while holding the stick', attacked, `state ${attackState}`);
  await touch('touchEnd', [[bx, by, 2], [ax, ay, 3]]);
  const afterLift = await until('pad reports only the stick', async () => { const v = await pad(); return v.fingers === 1 && !(v.bits & (K.X | K.A)) && v; }, 3000).catch(() => pad());
  ok('lifting the button fingers keeps the stick held', afterLift.fingers === 1 && (afterLift.bits & K.R) && !(afterLift.bits & (K.X | K.A)), JSON.stringify(afterLift));
  await touch('touchCancel', []);
  const cleared = await until('pad cleared', async () => { const v = await pad(); return v.fingers === 0 && v.bits === 0 && v; }, 3000).catch(() => pad());
  ok('touchcancel releases everything', cleared.fingers === 0 && cleared.bits === 0, JSON.stringify(cleared));

  await tapSel(btn('II'));
  await until('paused', async () => (await state()).paused === true, 5000);
  await wait(400);
  await padVisible('pause');
  await tapSel(btn('BACK'));
  await until('resumed', async () => (await state()).paused === false, 5000);
  ok('II pauses and BACK resumes', true);

  ok('no page errors', problems.length === 0, problems.join(' | '));
  console.log(checks.join('\n'));
  console.log(`WEB VPAD OK ${deviceName}`);
  await browser.close();
  server.close();
})().catch(async e => {
  console.log(checks.join('\n'));
  console.error(`WEB VPAD FAILED ${deviceName}:`, e.message);
  server.close();
  process.exit(1);
});
