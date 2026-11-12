const crypto = require('crypto');
const { openMatch, wait } = require('./web-match-driver');
const dir = process.argv[2] || 'web/dist';
const shotDir = process.env.WEB_SMOKE_SHOTS;
const stallMs = Number(process.env.MATCH_SMOKE_STALL_MS || 20000);
const limitMs = Number(process.env.MATCH_SMOKE_LIMIT_MS || 360000);
const only = process.env.MATCH_SMOKE_SCENARIOS ? process.env.MATCH_SMOKE_SCENARIOS.split(',') : null;
const fastRules = { round_time: 2, mute: 1, gamespeed: 2, life: 2 };
const scenarios = [
  { name: 'cpu-bot-best-of-3', query: 'e2e=1&p1=kaito&p2=shion&modo=cpu&luta=1', settings: { ...fastRules, rounds: 3 }, bot: true, minRounds: 2 },
  { name: 'watch-cpu-vs-cpu', query: 'e2e=1&p1=akio&p2=bruno&modo=watch&luta=1', settings: { ...fastRules, rounds: 0, single_round: 1 }, bot: false, minRounds: 1 },
];
const attackKeys = ['q', 'w', 'a', 's', 'e', 'd'];

function botKeys(s, mash) {
  const me = s.fighters.find(f => f.side === 0), foes = s.fighters.filter(f => f.side === 1 && f.alive);
  const want = new Set();
  if (!me || !foes.length || s.over >= 0) return want;
  const foe = foes.reduce((a, f) => (Math.abs(f.x - me.x) < Math.abs(a.x - me.x) ? f : a));
  const dx = foe.x - me.x;
  if (Math.abs(dx) > 55) want.add(dx > 0 ? 'ArrowRight' : 'ArrowLeft');
  if (foe.lane - me.lane > 5) want.add('x'); else if (me.lane - foe.lane > 5) want.add('z');
  if (Math.abs(dx) < 90) want.add(attackKeys[(mash >> 1) % attackKeys.length]);
  return want;
}

async function play(session, scenario) {
  const { problems } = session;
  const started = Date.now();
  const log = [];
  const hashes = new Set();
  const roundsSeen = new Set();
  let screen = '', screenSince = started, lastTick = -1, lastTickAt = started, lastRaf = -1, lastRafAt = started;
  let lastShotAt = 0, fightMs = 0, mash = 0, lastNudge = 0, finished = false, maxTick = 0;
  while (Date.now() - started < limitMs) {
    const now = Date.now();
    let s;
    try { s = await session.state(); } catch (e) { problems.push(`main thread unresponsive: ${e.message}`); break; }
    if (problems.some(p => /panic|crashed|pageerror/.test(p))) break;
    if (!s) { if (now - started > 120000) { problems.push('game state never published (e2e hook missing or wasm never booted)'); break; } await wait(300); continue; }
    if (s.screen !== screen) { log.push(`+${((now - started) / 1000).toFixed(1)}s ${s.screen}`); screen = s.screen; screenSince = now; lastTickAt = now; }
    if (s.raf !== lastRaf) { lastRaf = s.raf; lastRafAt = now; } else if (now - lastRafAt > stallMs) { problems.push(`no animation frame for ${now - lastRafAt} ms on screen ${screen}`); break; }
    if (screen === 'fight') {
      fightMs = now - screenSince;
      const clock = s.clock;
      if (clock) {
        roundsSeen.add(clock.round);
        maxTick = Math.max(maxTick, clock.tick);
        if (clock.tick !== lastTick) { lastTick = clock.tick; lastTickAt = now; }
        else if (!s.paused && s.over < 0 && now - lastTickAt > stallMs) { problems.push(`simulation froze at tick ${lastTick} (round ${clock.round}, state ${clock.round_state}) for ${now - lastTickAt} ms`); break; }
      }
      if (s.paused) { await session.setKeys(new Set()); await session.tap('Enter'); }
      else if (scenario.bot) await session.setKeys(botKeys(s, ++mash));
      if (now - lastShotAt > 4000) { lastShotAt = now; const buf = await session.shot(`fight-${log.length}`); if (buf) hashes.add(crypto.createHash('md5').update(buf).digest('hex')); }
      await session.frames(3);
    } else {
      await session.setKeys(new Set());
      if (screen === 'results') { finished = true; break; }
      if (now - screenSince > 150000) { problems.push(`stuck on screen ${screen} for ${now - screenSince} ms`); break; }
      if (['vs', 'stages', 'select', 'cutscene'].includes(screen) && now - lastNudge > 3000) { lastNudge = now; await session.tap('Enter'); }
      await wait(300);
    }
  }
  await session.setKeys(new Set());
  const result = { finished, fightMs, maxTick, rounds: roundsSeen.size, distinctFrames: hashes.size, log };
  if (!finished && !problems.length) problems.push(`match never reached the results screen within ${limitMs} ms (last screen ${screen}, tick ${lastTick})`);
  if (finished && roundsSeen.size < scenario.minRounds) problems.push(`expected at least ${scenario.minRounds} round(s), saw ${roundsSeen.size}`);
  if (finished && maxTick < 300) problems.push(`simulation barely ran (max tick ${maxTick})`);
  if (finished && fightMs > 12000 && hashes.size < 2) problems.push('rendered image never changed during the fight');
  return result;
}

async function leaveResults(session) {
  const deadline = Date.now() + 40000;
  while (Date.now() < deadline) {
    const s = await session.state().catch(() => null);
    if (s && s.screen !== 'results') return true;
    await session.tap('Enter');
    await wait(1200);
  }
  session.problems.push('results screen could not be left with Enter');
  return false;
}

async function run(scenario) {
  const session = await openMatch({ dir, query: scenario.query, settings: scenario.settings, shotDir, label: scenario.name });
  let result = null;
  try {
    result = await play(session, scenario);
    await session.shot('end');
    if (result.finished && !session.problems.length) await leaveResults(session);
  } catch (e) { session.problems.push(`script error: ${e.message}`); }
  const out = { name: scenario.name, problems: [...session.problems], result, consoleTail: session.consoleTail };
  await session.close();
  return out;
}

(async () => {
  const failed = [];
  for (const scenario of scenarios.filter(sc => !only || only.includes(sc.name))) {
    const started = Date.now();
    const out = await run(scenario);
    const summary = out.result ? `fight ${(out.result.fightMs / 1000).toFixed(0)}s, tick ${out.result.maxTick}, rounds ${out.result.rounds}, distinct frames ${out.result.distinctFrames}` : 'no result';
    console.log(`[${out.problems.length ? 'FAIL' : 'ok'}] ${scenario.name} in ${((Date.now() - started) / 1000).toFixed(0)}s: ${summary}`);
    if (out.result) console.log(`    screens: ${out.result.log.join(' > ')}`);
    if (out.problems.length) { failed.push(out); console.error(out.problems.map(p => `    ${p}`).join('\n') + '\n    last console messages:\n' + out.consoleTail.map(l => `      ${l}`).join('\n')); }
  }
  if (failed.length) { console.error(`WEB MATCH SMOKE FAILED: ${failed.map(f => f.name).join(', ')}`); process.exit(1); }
  console.log('WEB MATCH SMOKE OK');
})().catch(e => { console.error('SCRIPT FAIL', e); process.exit(2); });
