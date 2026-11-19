// Audio unlock smoke: after the first user gesture the page's AudioContext must be running, even before any sound plays.
// Usage: node scripts/web-audio-smoke.js [web/dist]   (build first with ./scripts/build-web.sh)
const path = require('path');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.zip': 'application/zip' };
const server = staticServer(dir, types);
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/index.html`;
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader', '--autoplay-policy=user-gesture-required'] });
  const p = await b.newPage({ viewport: { width: 1280, height: 720 } });
  await p.addInitScript(() => {
    const Native = window.AudioContext;
    const proto = Native.prototype;
    window.__audio = [];
    window.AudioContext = class extends Native { constructor(...args) { super(...args); window.__audio.push(this); } };
    window.AudioContext.prototype = proto;
    Object.defineProperty(proto, "state", { configurable: true, get() { return this.__unlocked ? "running" : "suspended"; } });
    proto.resume = function () { if (navigator.userActivation.isActive) this.__unlocked = true; return Promise.resolve(); };
  });
  const problems = [];
  p.on('pageerror', e => problems.push(String(e)));
  await p.goto(url);
  await p.waitForSelector('#boot', { state: 'detached', timeout: 120000 });
  const before = await p.evaluate(() => window.__audio.map(c => c.state));
  await p.mouse.click(640, 360);
  await p.waitForTimeout(400);
  const after = await p.evaluate(() => window.__audio.map(c => c.state));
  await b.close();
  server.close();
  const ok = after.includes('running') && problems.length === 0;
  console.log(JSON.stringify({ before, after, problems }));
  if (!ok) { console.error('FAIL: AudioContext not running after a user gesture'); process.exit(1); }
  console.log('OK: AudioContext running after gesture');
})().catch(e => { console.error(e); process.exit(1); });
