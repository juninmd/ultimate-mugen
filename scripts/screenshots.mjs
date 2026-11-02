// Usage: node scripts/screenshots.mjs [outDir] [baseUrl]   (start `npm run dev` first)
import { createRequire } from 'module';
import fs from 'fs';
const { chromium } = createRequire(import.meta.url)('playwright');
const out = process.argv[2] || 'screenshots';
const base = process.argv[3] || 'http://localhost:5173/';
fs.mkdirSync(out, { recursive: true });
const b = await chromium.launch();
const log = (p) => { p.on('pageerror', (e) => console.log('pageerror:', e.message)); p.on('console', (m) => { if (m.type() === 'error') console.log('console:', m.text()); }); };

async function waitFor(p, fn, timeout = 20000) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeout) { if (await p.evaluate(fn)) return true; await p.waitForTimeout(50); }
  return false;
}

const p = await b.newPage({ viewport: { width: 1280, height: 720 } });
log(p);
await p.goto(base);
await p.waitForTimeout(800);
await p.screenshot({ path: `${out}/01-title.png` });
await p.keyboard.press('Enter');
await p.waitForTimeout(400);
await p.screenshot({ path: `${out}/02-menu.png` });
// import KFM + stages if available
if (fs.existsSync('test-assets/kfm.zip')) {
  await p.goto(base + '?screen=import');
  await p.setInputFiles('input.f1', ['test-assets/kfm.zip', 'test-assets/stages.zip'].filter((f) => fs.existsSync(f)));
  await p.waitForTimeout(1500);
  await p.screenshot({ path: `${out}/03-import.png` });
}
await p.goto(base + '?screen=select');
await p.waitForTimeout(1200);
await p.keyboard.press('ArrowRight'); await p.keyboard.press('ArrowRight');
await p.waitForTimeout(600);
await p.screenshot({ path: `${out}/04-select.png` });
await p.keyboard.press('Enter');
await p.keyboard.press('ArrowDown'); await p.keyboard.press('ArrowRight');
await p.waitForTimeout(500);
await p.screenshot({ path: `${out}/05-select-p2.png` });
await p.keyboard.press('Enter');
await p.waitForTimeout(500);
await p.screenshot({ path: `${out}/06-stages.png` });
await p.keyboard.press('ArrowRight');
await p.keyboard.press('Enter');
await p.waitForTimeout(900);
await p.screenshot({ path: `${out}/07-vs.png` });

// native vs native, CPU vs CPU
async function fight(page, q, name, ticks, opts = {}) {
  await page.goto(base + '?auto=fight&mode=watch&' + q);
  await waitFor(page, () => window.__ume?.session?.match?.roundState === 2);
  await page.evaluate(() => { window.__ume.session.paused = true; });
  for (let i = 0; i < ticks; i += 60) await page.evaluate((n) => window.__ume.session.advance(n), Math.min(60, ticks - i));
  if (opts.super) {
    await page.evaluate(() => {
      const m = window.__ume.session.match;
      for (const f of m.roots) f.power = 3000;
    });
    const ok = await page.evaluate(() => {
      const s = window.__ume.session, m = s.match;
      for (let i = 0; i < 1200; i++) { s.advance(1); if (m.pauseState && m.pauseState.super && m.pauseState.time < 25) return true; }
      return false;
    });
    if (ok) await page.evaluate(() => window.__ume.session.advance(18));
    console.log(name, 'super caught:', ok);
  }
  if (opts.hit) {
    await page.evaluate(() => {
      const s = window.__ume.session, m = s.match;
      for (let i = 0; i < 900; i++) { s.advance(1); const h = m.roots.find((f) => f.moveType === 'H' && f.hitShake > 3 && !f.ghv.guarded); if (h && m.explods.length) return; }
    });
  }
  await page.screenshot({ path: `${out}/${name}.png` });
}
const ids = await p.evaluate(() => window.__ume.reg.chars.map((c) => c.id));
const stages = await p.evaluate(() => window.__ume.reg.stages.map((c) => c.id));
console.log(ids, stages);
const nat = ids.filter((i) => i.startsWith('umf:'));
const mug = ids.filter((i) => i.startsWith('mugen:'));
await fight(p, `p1=${nat[0]}&p2=${nat[1]}&stage=native:dojo`, '08-fight-native', 300, { hit: true });
await fight(p, `p1=${nat[2]}&p2=${nat[3]}&stage=native:neon`, '09-fight-neon', 420, { hit: true });
await fight(p, `p1=${nat[1]}&p2=${nat[4] ?? nat[0]}&stage=native:spirit`, '10-fight-super', 120, { super: true });
await fight(p, `p1=${nat[5] ?? nat[0]}&p2=${nat[6] ?? nat[1]}&stage=native:bamboo`, '11-fight-bamboo', 500, { hit: true });
await fight(p, `p1=${nat[7] ?? nat[0]}&p2=${nat[8] ?? nat[1]}&stage=native:volcano`, '12-fight-volcano', 600, { hit: true });
if (mug.length) {
  await fight(p, `p1=${mug[0]}&p2=${nat[2]}&stage=native:plains`, '13-mugen-vs-native', 400, { hit: true });
  const ms = stages.find((s) => s.startsWith('mugenstage') && s.endsWith('kfm.def'));
  if (ms) await fight(p, `p1=${nat[9] ?? nat[0]}&p2=${mug[0]}&stage=${ms}`, '14-native-on-mugen-stage', 400, { hit: true });
}
// debug hitboxes
await p.goto(base + `?auto=fight&mode=watch&debug=1&p1=${nat[0]}&p2=${nat[3]}&stage=native:plains`);
await waitFor(p, () => window.__ume?.session?.match?.roundState === 2);
await p.evaluate(() => { const s = window.__ume.session; s.paused = true; for (let i = 0; i < 600; i++) { s.advance(1); if (s.match.roots.some((f) => f.boxes(1).length)) break; } });
await p.screenshot({ path: `${out}/15-hitboxes.png` });
await p.close();

// phone landscape with touch controls
const ph = await b.newPage({ viewport: { width: 844, height: 390 }, deviceScaleFactor: 2, hasTouch: true, isMobile: true });
log(ph);
await ph.goto(base + `?touch=1`);
await ph.waitForTimeout(600);
await ph.screenshot({ path: `${out}/16-mobile-title.png` });
await ph.goto(base + `?touch=1&screen=select`);
await ph.waitForTimeout(1000);
await ph.screenshot({ path: `${out}/17-mobile-select.png` });
await ph.goto(base + `?auto=fight&mode=cpu&touch=1&p1=${nat[2]}&p2=${nat[5] ?? nat[1]}&stage=native:dojo`);
await waitFor(ph, () => window.__ume?.session?.match?.roundState === 2);
await ph.evaluate(() => { const s = window.__ume.session; s.paused = true; s.advance(200); });
await ph.screenshot({ path: `${out}/18-mobile-fight.png` });
await b.close();
console.log('done');
