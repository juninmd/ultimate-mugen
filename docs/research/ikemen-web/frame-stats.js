// Counts the WebGL work the engine issues per frame, from inside the engine's own iframe.
// The game runs in frame.html (a child frame), so patch the prototypes there and read the counters
// with frame.evaluate(); page.evaluate() only sees the menu shell and always reports zeros.
// Usage: node frame-stats.js   (needs playwright + chromium)
const { chromium } = require('playwright');

const URL = 'https://w8.snokido.com/games/html5/dbz-vs-marvel-vs-capcom/index.html';

const INIT = `(() => {
  const S = window.__rea = { drawArrays: 0, drawElements: 0, uniforms: 0, bufferData: 0, bufferSubData: 0,
    useProgram: 0, bindTexture: 0, texImage2D: 0, texImage2Dbytes: 0, createTexture: 0, raf: 0, rafTimes: [] };
  const patch = (proto, name, fn) => { const o = proto[name]; if (o) proto[name] = function (...a) { try { fn(a); } catch (e) {} return o.apply(this, a); }; };
  const W = window.WebGL2RenderingContext && window.WebGL2RenderingContext.prototype;
  if (!W) return;
  patch(W, 'drawArrays', () => S.drawArrays++);
  patch(W, 'drawElements', () => S.drawElements++);
  patch(W, 'bufferData', () => S.bufferData++);
  patch(W, 'bufferSubData', () => S.bufferSubData++);
  patch(W, 'useProgram', () => S.useProgram++);
  patch(W, 'bindTexture', () => S.bindTexture++);
  patch(W, 'createTexture', () => S.createTexture++);
  patch(W, 'texImage2D', (a) => { S.texImage2D++; if (typeof a[5] === 'number' && a[3] > 1) S.texImage2Dbytes += a[3] * a[4] * 4; });
  for (const u of ['uniform1f','uniform1i','uniform2f','uniform3f','uniform4f','uniform4fv','uniformMatrix4fv','uniform1fv','uniformMatrix3fv'])
    patch(W, u, () => S.uniforms++);
  let last = performance.now();
  const tick = () => { const now = performance.now(); S.raf++; S.rafTimes.push(now - last); last = now; if (S.rafTimes.length > 4000) S.rafTimes.shift(); requestAnimationFrame(tick); };
  requestAnimationFrame(tick);
  window.__reaLongTasks = [];
  try { new PerformanceObserver((l) => { for (const e of l.getEntries()) window.__reaLongTasks.push(+e.duration.toFixed(0)); }).observe({ entryTypes: ['longtask'] }); } catch (e) {}
})();`;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

(async () => {
  const browser = await chromium.launch({ headless: true, args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--autoplay-policy=no-user-gesture-required'] });
  const page = await (await browser.newContext({ viewport: { width: 1280, height: 720 } })).newPage();
  await page.addInitScript(INIT);
  const t0 = Date.now();
  await page.goto(URL, { waitUntil: 'domcontentloaded' });

  const visible = (sel) => page.locator(sel).isVisible().catch(() => false);
  const waitFor = async (sel, ms = 120000) => { while (Date.now() - t0 < ms) { if (await visible(sel)) return Date.now() - t0; await sleep(200); } return null; };

  console.log('menu at', await waitFor('#screen-menu'), 'ms');
  await page.locator('#menu-list > *').first().click().catch(() => {});
  console.log('select at', await waitFor('#screen-select'), 'ms');
  await page.locator('#select-random').click().catch(() => {});
  await sleep(300);
  await page.locator('#select-confirm').click().catch(() => {});
  console.log('fight at', await waitFor('#screen-fight', 300000), 'ms');

  let frame = null;
  for (let i = 0; i < 100 && !frame; i++) { frame = page.frames().find((f) => f.url().includes('frame.html')); if (!frame) await sleep(200); }
  if (!frame) throw new Error('engine frame never appeared');
  await sleep(4000);

  const sample = async (label, seconds) => {
    const a = await frame.evaluate(() => ({ ...window.__rea, rafTimes: undefined }));
    await sleep(seconds * 1000);
    const b = await frame.evaluate(() => {
      const S = window.__rea;
      const ft = S.rafTimes.slice(-240).sort((x, y) => x - y);
      return { ...S, rafTimes: undefined, p50: ft[ft.length >> 1], p95: ft[Math.floor(ft.length * 0.95)], worst: ft[ft.length - 1],
        engineFrames: window.__ikemenEngineFrames, canvas: (() => { const c = document.querySelector('canvas'); return c ? [c.width, c.height, c.clientWidth, c.clientHeight] : null; })(),
        heapMb: performance.memory ? +(performance.memory.usedJSHeapSize / 1048576).toFixed(1) : null, longTasks: (window.__reaLongTasks || []).slice(-30) };
    });
    const per = (k) => +(((b[k] || 0) - (a[k] || 0)) / seconds).toFixed(1);
    console.log(`\n== ${label} (${seconds}s) ==`);
    console.log({ drawArraysPerSec: per('drawArrays'), uniformsPerSec: per('uniforms'), bufferDataPerSec: per('bufferData'), useProgramPerSec: per('useProgram'), textureBindsPerSec: per('bindTexture') });
    console.log({ canvas: b.canvas, engineFrames: b.engineFrames, rafP50ms: +b.p50.toFixed(1), rafP95ms: +b.p95.toFixed(1), heapMb: b.heapMb, longTasks: b.longTasks });
  };

  await sample('idle fight', 5);
  for (const k of ['a', 's', 'd', 'z', 'x', 'c', 'w']) { await page.keyboard.press(k).catch(() => {}); await sleep(90); }
  await sample('combat', 5);

  const perFrame = await frame.evaluate(() => new Promise((res) => {
    const S = window.__rea, out = [];
    let prev = { d: S.drawArrays, u: S.uniforms, b: S.bufferData, f: window.__ikemenEngineFrames };
    const step = () => {
      const cur = { d: S.drawArrays, u: S.uniforms, b: S.bufferData, f: window.__ikemenEngineFrames };
      out.push({ draw: cur.d - prev.d, uniforms: cur.u - prev.u, bufferData: cur.b - prev.b, engineFrames: cur.f - prev.f });
      prev = cur;
      if (out.length < 120) requestAnimationFrame(step); else res(out);
    };
    requestAnimationFrame(step);
  }));
  const draws = perFrame.map((s) => s.draw).sort((a, b) => a - b);
  console.log('\n== per-rAF drawArrays over 120 rAF callbacks ==');
  console.log({ p50: draws[60], p95: draws[114], max: draws[draws.length - 1],
    uniformsPerRaf: +(perFrame.reduce((a, s) => a + s.uniforms, 0) / perFrame.length).toFixed(1),
    bufferDataPerRaf: +(perFrame.reduce((a, s) => a + s.bufferData, 0) / perFrame.length).toFixed(1),
    engineFramesPerRaf: +(perFrame.reduce((a, s) => a + s.engineFrames, 0) / perFrame.length).toFixed(3) });

  await browser.close();
})();
