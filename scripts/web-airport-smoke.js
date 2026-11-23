const { openMatch, wait } = require('./web-match-driver');
const dir = process.argv[2] || 'web/dist';
const shotDir = process.env.WEB_SMOKE_SHOTS || 'airport-shots';
const stage = process.env.AIRPORT_STAGE || '21';
const fight = process.env.AIRPORT_FIGHT || 'p1=ichigo3d&p2=kaito';

(async () => {
  const session = await openMatch({
    dir,
    query: `e2e=1&${fight}&cenario=${stage}&modo=watch&luta=1`,
    settings: { mute: 1, round_time: 99, gamespeed: 1 },
    shotDir,
    label: 'airport',
  });
  const { problems } = session;
  let fightSeen = false;
  for (let i = 0; i < 400 && !fightSeen; i++) {
    const s = await session.state();
    fightSeen = !!s && s.screen === 'fight';
    if (!fightSeen) await wait(300);
  }
  if (!fightSeen) problems.push('never reached the fight screen');
  const frames = [];
  for (let k = 0; k < 6; k++) {
    await wait(2500);
    const before = (await session.state())?.raf || 0;
    const t0 = Date.now();
    await session.frames(60);
    const after = (await session.state())?.raf || 0;
    frames.push({ ms: Date.now() - t0, raf: after - before });
    await session.shot(`t${k}`);
  }
  const mean = frames.reduce((a, f) => a + f.ms, 0) / frames.length / 60;
  console.log(JSON.stringify({ fightSeen, meanFrameMs: Number(mean.toFixed(2)), samples: frames }));
  await session.close();
  if (problems.length) { console.error(problems.join('\n')); process.exit(1); }
})().catch(e => { console.error(e); process.exit(2); });
