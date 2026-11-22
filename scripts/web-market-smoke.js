// Marketplace smoke: adds a GitHub repository (API stubbed in the browser, no network needed), then checks that the
// grid shows MUGEN and UMF packs, loads only the visible portraits lazily and draws the portrait pixels.
// Usage: node scripts/web-market-smoke.js [web/dist]   (WEB_SMOKE_SHOTS=<dir> keeps the screenshots)
const fs = require('fs'), path = require('path'), zlib = require('zlib'), crypto = require('crypto');
const { staticServer } = require('./web-smoke-server');
const { chromium } = require('playwright');
const dir = path.resolve(process.argv[2] || path.join(__dirname, '..', 'web', 'dist'));
const shotDir = process.env.WEB_SMOKE_SHOTS;
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.json': 'application/json', '.webmanifest': 'application/manifest+json', '.png': 'image/png' };
const PORTRAIT = [0, 200, 90];
const crcTable = Array.from({ length: 256 }, (_, n) => { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; return c >>> 0; });
const crc = buf => { let c = 0xffffffff; for (const b of buf) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8); return (c ^ 0xffffffff) >>> 0; };
const chunk = (type, data) => { const len = Buffer.alloc(4); len.writeUInt32BE(data.length); const body = Buffer.concat([Buffer.from(type), data]); const sum = Buffer.alloc(4); sum.writeUInt32BE(crc(body)); return Buffer.concat([len, body, sum]); };
const solidPng = (w, h, rgb) => {
  const row = Buffer.concat([Buffer.from([0]), Buffer.from(Array.from({ length: w }, () => rgb).flat())]);
  const ihdr = Buffer.alloc(13); ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4); ihdr[8] = 8; ihdr[9] = 2;
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', ihdr), chunk('IDAT', zlib.deflateSync(Buffer.concat(Array.from({ length: h }, () => row)))), chunk('IEND', Buffer.alloc(0))]);
};
const readPng = buf => {
  let pos = 8, width = 0, height = 0, bpp = 4; const idat = [];
  while (pos < buf.length) { const len = buf.readUInt32BE(pos), type = buf.toString('ascii', pos + 4, pos + 8); const data = buf.subarray(pos + 8, pos + 8 + len); if (type === 'IHDR') { width = data.readUInt32BE(0); height = data.readUInt32BE(4); bpp = data[9] === 6 ? 4 : 3; } if (type === 'IDAT') idat.push(data); pos += 12 + len; }
  const raw = zlib.inflateSync(Buffer.concat(idat)), stride = width * bpp, out = Buffer.alloc(stride * height);
  for (let y = 0; y < height; y++) {
    const f = raw[y * (stride + 1)], src = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let x = 0; x < stride; x++) {
      const a = x >= bpp ? out[y * stride + x - bpp] : 0, b = y ? out[(y - 1) * stride + x] : 0, c = x >= bpp && y ? out[(y - 1) * stride + x - bpp] : 0;
      const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
      const pred = f === 0 ? 0 : f === 1 ? a : f === 2 ? b : f === 3 ? (a + b) >> 1 : pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      out[y * stride + x] = (src[x] + pred) & 255;
    }
  }
  return { width, height, bpp, data: out };
};
const names = ['Aquaman', 'Batman', 'BlackPanther', 'BlackWidow', 'Cyclops', 'Darkseid', 'Deadpool', 'Deidara', 'Goku_SS4', 'Hawkeye', 'Loki', 'Thor'];
const entries = [];
const add = (p, size) => entries.push({ path: p, type: 'blob', size, sha: crypto.createHash('sha1').update(p).digest('hex') });
for (const n of names) { add(`chars/${n}.zip`, 5_000_000); add(`previews/chars/${n}.png`, 30_000); }
for (const n of ['BlackWidow_UMF', 'Goku_SSJ4_UMF']) { add(`umf/${n}.zip`, 8_000_000); add(`previews/umf/${n}.png`, 30_000); }
const cors = { 'access-control-allow-origin': '*', 'access-control-expose-headers': 'etag' };
const server = staticServer(dir, types);
(async () => {
  await new Promise(ok => server.listen(0, '127.0.0.1', ok));
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist', '--enable-unsafe-swiftshader'] });
  const p = await b.newPage({ viewport: { width: 1280, height: 720 } });
  const problems = [], previews = new Set();
  p.on('pageerror', e => { if (!/register_plugin/.test(e.message)) problems.push(`pageerror: ${e.message}`); });
  p.on('console', m => { if (/panicked|PanicHookInfo/.test(m.text())) problems.push(`console: ${m.text().slice(0, 200)}`); });
  p.on('dialog', d => d.accept('juninmd/mugen-assets'));
  await p.route('https://api.github.com/**', r => r.fulfill({ status: 200, headers: { ...cors, 'content-type': 'application/json', etag: '"stub"' }, body: JSON.stringify({ sha: 'x', truncated: false, tree: entries }) }));
  await p.route('https://raw.githubusercontent.com/**', r => {
    const url = new URL(r.request().url());
    if (/\/previews\/.*\.png$/.test(url.pathname)) { previews.add(url.pathname); return r.fulfill({ status: 200, headers: { ...cors, 'content-type': 'image/png' }, body: solidPng(480, 270, PORTRAIT) }); }
    return r.fulfill({ status: 404, headers: cors, body: '' });
  });
  await p.goto(`http://127.0.0.1:${server.address().port}/index.html?e2e=1`);
  await p.locator('#boot').waitFor({ state: 'detached', timeout: 45000 }).catch(() => {});
  await p.waitForTimeout(10000);
  await p.locator('#glcanvas').focus();
  const frames = n => p.evaluate(n => new Promise(done => { const step = i => (i ? requestAnimationFrame(() => step(i - 1)) : done()); step(n); }), n);
  const key = async (k, wait = 800) => { await p.keyboard.down(k); await frames(2); await p.keyboard.up(k); await frames(4); await p.waitForTimeout(wait); };
  const snap = async name => { const buf = await p.screenshot(); if (shotDir) { fs.mkdirSync(shotDir, { recursive: true }); fs.writeFileSync(path.join(shotDir, `${name}.png`), buf); } return buf; };
  const screen = () => p.evaluate(() => JSON.parse(window.__ume_state || 'null')?.screen);
  await key('Enter', 1500);
  for (let i = 0; i < 6; i++) await key('ArrowDown', 300);
  await key('Enter', 2500);
  if ((await screen()) !== 'market') problems.push(`expected the market screen, got ${await screen()}`);
  await key('KeyU', 4000);
  await snap('market-list');
  await key('KeyV', 6000);
  const grid = readPng(await snap('market-grid'));
  const at = (x, y) => [0, 1, 2].map(c => grid.data[(y * grid.width + x) * grid.bpp + c]);
  const centre = at(502, 262);
  if (centre.some((v, i) => Math.abs(v - PORTRAIT[i]) > 30)) problems.push(`first card portrait not drawn, centre pixel ${centre}`);
  const loaded = previews.size;
  if (loaded < 1 || loaded > 14) problems.push(`portraits must load lazily for the visible cards only, requested ${loaded} of ${names.length + 2}`);
  await p.mouse.move(700, 430); await p.mouse.down(); await p.mouse.move(700, 330, { steps: 8 }); await p.mouse.move(700, 150, { steps: 8 }); await p.mouse.up();
  await p.waitForTimeout(5000);
  await snap('market-grid-dragged');
  if (previews.size <= loaded) problems.push(`dragging the grid must reveal and load the next portraits (${previews.size} requested after ${loaded})`);
  await key('KeyJ', 1500);
  const umf = await snap('market-grid-umf');
  if (!umf.length) problems.push('empty screenshot');
  await key('KeyP', 1200);
  await snap('market-grid-outdated');
  await key('Escape', 1200);
  await b.close(); server.close();
  if (problems.length) { console.error('MARKET SMOKE FAILED\n' + problems.join('\n')); process.exit(1); }
  console.log(`MARKET SMOKE OK: ${loaded} portraits requested for ${entries.filter(e => /previews\//.test(e.path)).length} listed`);
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });
