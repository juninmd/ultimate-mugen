const fs = require('fs'), path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip', '.png': 'image/png' };
const ignoredConsole = /favicon|sw\.js|GPU stall|GroupMarker|WebGL: INVALID_OPERATION: texSubImage|\[perf\]/;
const wait = ms => new Promise(r => setTimeout(r, ms));
const settingsText = s => Object.entries(s).map(([k, v]) => `${k}=${v}`).join('\n') + '\n';

async function openMatch({ dir, query, settings, shotDir, label }) {
  const server = staticServer(path.resolve(dir), types);
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html?${query}`;
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader', '--autoplay-policy=no-user-gesture-required'] });
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  const problems = [], consoleTail = [];
  page.on('crash', () => problems.push('Chromium page crashed'));
  page.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message.slice(0, 300)}`); });
  page.on('console', m => {
    const text = m.text();
    consoleTail.push(`${m.type()}: ${text.slice(0, 160)}`);
    if (consoleTail.length > 40) consoleTail.shift();
    if (/panicked|PanicHookInfo|unreachable executed|RuntimeError/.test(text)) problems.push(`panic: ${text.slice(0, 300)}`);
    else if (m.type() === 'error' && !ignoredConsole.test(text)) problems.push(`console error: ${text.slice(0, 200)}`);
  });
  await page.addInitScript(cfg => {
    try { localStorage.setItem('ume.ultimate-mugen.cfg', cfg); } catch (e) {}
    window.__rafFrames = 0;
    const count = () => { window.__rafFrames++; requestAnimationFrame(count); };
    requestAnimationFrame(count);
  }, settingsText(settings));
  await page.goto(url);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  await page.locator('#glcanvas').focus();
  if (shotDir) fs.mkdirSync(shotDir, { recursive: true });
  const held = new Set();
  const withTimeout = (promise, ms, what) => Promise.race([promise, new Promise((_, no) => setTimeout(() => no(new Error(`${what} timed out after ${ms} ms`)), ms))]);
  const api = {
    page, problems, consoleTail,
    state: () => withTimeout(page.evaluate(() => { try { return { ...JSON.parse(window.__ume_state), raf: window.__rafFrames }; } catch { return null; } }), 30000, 'page state read'),
    frames: n => page.evaluate(n => new Promise(done => { const s = i => (i ? requestAnimationFrame(() => s(i - 1)) : done()); s(n); }), n),
    async tap(k) { await page.keyboard.down(k); await api.frames(2); await page.keyboard.up(k); await api.frames(3); },
    async setKeys(want) {
      for (const k of [...held]) if (!want.has(k)) { await page.keyboard.up(k); held.delete(k); }
      for (const k of want) if (!held.has(k)) { await page.keyboard.down(k); held.add(k); }
    },
    async shot(name) {
      const buffer = await page.screenshot().catch(() => null);
      if (buffer && shotDir) fs.writeFileSync(path.join(shotDir, `${label}-${name}.png`), buffer);
      return buffer;
    },
    async close() { await browser.close().catch(() => {}); server.close(); },
  };
  return api;
}

module.exports = { openMatch, wait };
