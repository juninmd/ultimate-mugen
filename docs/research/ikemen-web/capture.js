// Harvests every artifact the game fetches, plus the request order and sizes.
// Usage: node capture.js [outDir]   (needs playwright + chromium)
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const URL = 'https://w8.snokido.com/games/html5/dbz-vs-marvel-vs-capcom/index.html';
const OUT = path.resolve(process.argv[2] || 'capture');
fs.mkdirSync(path.join(OUT, 'bodies'), { recursive: true });

(async () => {
  const browser = await chromium.launch({ headless: true, args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
  const page = await (await browser.newContext({ viewport: { width: 1280, height: 720 } })).newPage();
  const rows = [];
  const seen = new Set();

  page.on('response', async (res) => {
    const url = res.url();
    if (url.startsWith('data:')) return;
    const ctype = res.headers()['content-type'] || '';
    let size = null;
    try { const body = await res.body(); size = body.length; } catch {}
    rows.push({ url, status: res.status(), ctype, size });
    const wanted = /javascript|json|zip|octet-stream/.test(ctype) || /\.(js|json|wasm|bin|zip)(\?|$)/.test(url);
    if (wanted && !seen.has(url)) {
      seen.add(url);
      try {
        const body = await res.body();
        const name = url.replace(/^https?:\/\//, '').replace(/[^a-zA-Z0-9._-]/g, '_').slice(0, 180);
        fs.writeFileSync(path.join(OUT, 'bodies', name), body);
      } catch {}
    }
  });

  await page.goto(URL, { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(45000);
  const frames = page.frames().map((f) => f.url());
  fs.writeFileSync(path.join(OUT, 'network.json'), JSON.stringify({ frames, rows }, null, 1));
  console.log('frames:', frames.join('\n         '));
  for (const r of rows) if (!/webp|woff2/.test(r.url)) console.log(`${r.status} ${String(r.size).padStart(9)} ${r.ctype.padEnd(34)} ${r.url}`);
  console.log(`\nmedia requests: ${rows.filter((r) => /webp/.test(r.url)).length}`);
  await browser.close();
})();
