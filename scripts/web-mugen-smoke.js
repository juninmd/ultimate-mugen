// Web M.U.G.E.N import homologation: boots the wasm build, imports real character packs through the same file input the
// player uses, then deep-links into a fight with the imported fighter and checks that the match runs (ticks advance, no
// panic, frames keep coming, memory bounded).  Usage: node scripts/web-mugen-smoke.js [web/dist] [zip...]
// (needs `playwright`; build first with ./scripts/build-web.sh).
const fs = require('fs'), path = require('path');
const { openMatch, wait } = require('./web-match-driver');

const dir = process.argv[2] || 'web/dist';
const zips = process.argv.slice(3);
const envZips = (process.env.MUGEN_ZIPS || '').split(/[;,]/).filter(Boolean);
const packs = (zips.length ? zips : envZips).map(z => path.resolve(z));
const bootMs = Number(process.env.MUGEN_SMOKE_IMPORT_MS || 180000);
const shotDir = process.env.WEB_SMOKE_SHOTS;
const maxWasmMb = Number(process.env.MUGEN_SMOKE_MAX_WASM_MB || 1024);
const forcedChar = process.env.MUGEN_SMOKE_CHAR || '';

async function readFrames(session, ms) {
  const before = await session.state();
  await wait(ms);
  const after = await session.state();
  return { before, after, frames: (after?.raf || 0) - (before?.raf || 0) };
}

async function importPacks(session, startChars) {
  const { page, problems } = session;
  for (const pack of packs) {
    if (!fs.existsSync(pack)) { problems.push(`missing pack: ${pack}`); continue; }
    await page.setInputFiles('#ume-file', pack).catch(e => problems.push(`setInputFiles ${pack}: ${e.message}`));
    const started = Date.now();
    let settled = 0;
    while (Date.now() - started < bootMs) {
      const state = await session.state().catch(() => null);
      if (!state) { await wait(250); continue; }
      const imp = state.import || {};
      if (!imp.busy && !imp.queued) {
        settled += 1;
        if (settled >= 3) break;
      } else {
        settled = 0;
      }
      await wait(250);
    }
    const state = await session.state();
    const gained = (state?.roster?.chars || 0) - startChars;
    if (gained <= 0) console.log(`note: ${path.basename(pack)} added no new character (already in the roster, deduplicated by content key)`);
    else console.log(`imported ${path.basename(pack)}: +${gained} character(s), roster ${state.roster.chars}, wasm ${state.wasm_mb} MiB`);
    startChars = state?.roster?.chars || startChars;
  }
  return startChars;
}

function needleFrom(state) {
  const tail = (state?.roster?.tail || [])[0] || '';
  const def = tail.split(':').pop() || '';
  return def.split('/').pop().replace(/\.def$/i, '') || def;
}

(async () => {
  if (!packs.length) {
    console.error('no M.U.G.E.N pack given (pass a zip path or set MUGEN_ZIPS)');
    process.exit(2);
  }
  const session = await openMatch({ dir, query: 'e2e=1', settings: { mute: 1, gamespeed: 2 }, shotDir, label: 'mugen' });
  const { problems, page } = session;
  const close = () => session.close();
  let state = await session.state();
  for (let i = 0; i < 40 && (!state || !state.roster); i++) { await wait(500); state = await session.state(); }
  if (!state || !state.roster) { problems.push('the wasm build never published its roster'); await close(); finish(problems); }
  const startChars = state.roster.chars;
  console.log(`boot: roster ${startChars} character(s), ${state.roster.stages} stage(s), wasm ${state.wasm_mb} MiB`);
  await session.shot('00-boot');
  await importPacks(session, startChars);
  await session.shot('01-imported');
  state = await session.state();
  state = await session.state();
  const needle = forcedChar || needleFrom(state);
  if (!needle) { problems.push('could not name an imported fighter for the deep link'); await close(); finish(problems); }
  const url = new URL(page.url());
  url.search = `?e2e=1&p1=${encodeURIComponent(needle)}&p2=${encodeURIComponent(needle)}&modo=watch&luta=1`;
  console.log(`deep link: ${needle}`);
  await page.goto(url.href);
  await page.locator('#boot').waitFor({ state: 'detached', timeout: 60000 }).catch(() => {});
  await page.locator('#glcanvas').focus();
  let screen = '';
  for (let i = 0; i < 120; i++) {
    state = await session.state();
    screen = state?.screen || '';
    if (screen === 'fight') break;
    await wait(500);
  }
  if (screen !== 'fight') problems.push(`never reached the fight screen (stuck at ${screen})`);
  await session.shot('02-fight-start');
  const sample = await readFrames(session, 3000);
  const tickStart = sample.before?.clock?.tick ?? -1;
  const tickEnd = sample.after?.clock?.tick ?? -1;
  if (tickStart < 0 || tickEnd <= tickStart) problems.push(`the match is frozen (tick ${tickStart} -> ${tickEnd})`);
  // Software GL on a CI runner draws the animated stage far slower than a desktop GPU: 6 frames in 3 s still
  // proves the loop is alive, and the tick counter above proves the match itself is advancing.
  if (sample.frames < 6) problems.push(`only ${sample.frames} frames in 3 s (renderer stalled)`);
  const fighters = sample.after?.fighters || [];
  if (!fighters.length) problems.push('no fighter published in the match state');
  const signature = state => (state?.fighters || []).map(f => `${f.state}:${f.anim}:${f.anim_elem}`).join('|');
  if (signature(sample.before) === signature(sample.after)) problems.push('fighters never changed state or animation in 3 s');
  const wasm = sample.after?.wasm_mb || 0;
  if (wasm > maxWasmMb) problems.push(`wasm heap grew to ${wasm} MiB`);
  console.log(`fight: screen=${screen} tick ${tickStart}->${tickEnd} frames=${sample.frames} wasm=${wasm} MiB fighters=${fighters.map(f => `life ${f.life}/${f.max} st ${f.state}`).join(' | ')}`);
  await wait(2000);
  await session.shot('03-fight-later');
  await wait(3000);
  await session.shot('04-fight-end');
  const tail = await session.state();
  if ((tail?.roster?.errors || 0) > (state?.roster?.errors || 0)) problems.push('import errors grew while playing');
  await close();
  finish(problems);
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });

function finish(problems) {
  if (problems.length) { console.error('WEB MUGEN SMOKE FAILED\n' + problems.join('\n')); process.exit(1); }
  console.log('WEB MUGEN SMOKE OK');
  process.exit(0);
}
