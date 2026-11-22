// Mobile preflight smoke: a phone without WebAssembly or WebGL2 must get an explanatory message and a reload button, not a stuck loader.
// Usage: node scripts/web-preflight-smoke.js [web/dist]   (build first with ./scripts/build-web.sh)
const path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium, devices } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json' };
const cases = [
  { name: 'no-wasm', device: 'iPhone 13', stub: () => { delete window.WebAssembly; }, expect: /WebAssembly/ },
  { name: 'no-webgl2', device: 'Pixel 7', stub: () => { const g = HTMLCanvasElement.prototype.getContext; HTMLCanvasElement.prototype.getContext = function (t, ...a) { return t === 'webgl2' ? null : g.call(this, t, ...a); }; }, expect: /WebGL2/ },
];
(async () => {
  const server = staticServer(dir, types);
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  let failed = false;
  for (const c of cases) {
    const ctx = await b.newContext({ ...devices[c.device] });
    const p = await ctx.newPage();
    await p.addInitScript(c.stub);
    await p.goto(url);
    await p.waitForSelector('#fatal.on', { timeout: 15000 }).catch(() => {});
    const r = await p.evaluate(() => ({ on: document.getElementById('fatal').classList.contains('on'), text: document.getElementById('fataltx').textContent, button: document.getElementById('fatalbtn').textContent, boot: !!document.getElementById('boot') }));
    if (process.env.SHOT) await p.screenshot({ path: path.join(process.env.SHOT, `preflight-${c.name}.png`) });
    const ok = r.on && c.expect.test(r.text) && r.button && !r.boot;
    console.log(JSON.stringify({ case: c.name, ...r, ok }));
    if (!ok) failed = true;
    await ctx.close();
  }
  await b.close();
  server.close();
  if (failed) { console.error('FAIL: unsupported phone did not get an explanatory message'); process.exit(1); }
  console.log('OK: unsupported phones get a clear message and reload');
})().catch(e => { console.error(e); process.exit(1); });
