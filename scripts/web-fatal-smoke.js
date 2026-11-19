// Fatal-error smoke: an uncaught wasm trap (panic, OOM abort) must show a message and a reload button, not a frozen canvas.
// Usage: node scripts/web-fatal-smoke.js [web/dist]   (build first with ./scripts/build-web.sh)
const path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const server = staticServer(dir, types);
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const p = await b.newPage({ viewport: { width: 1280, height: 720 } });
  await p.goto(url);
  await p.waitForSelector('#boot', { state: 'detached', timeout: 120000 });
  await p.evaluate(() => setTimeout(() => { throw new WebAssembly.RuntimeError('unreachable'); }, 0));
  await p.waitForTimeout(300);
  const shown = await p.evaluate(() => document.getElementById('fatal')?.classList.contains('on') === true);
  const button = await p.evaluate(() => document.getElementById('fatalbtn')?.textContent || '');
  await b.close();
  server.close();
  console.log(JSON.stringify({ shown, button }));
  if (!shown || !button) { console.error('FAIL: fatal error left the canvas frozen with no message'); process.exit(1); }
  console.log('OK: fatal error shows message and reload');
})().catch(e => { console.error(e); process.exit(1); });
