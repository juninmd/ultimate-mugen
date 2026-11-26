const fs = require('fs');
const path = require('path');
const { openMatch, wait } = require('./web-match-driver');

const dir = process.argv[2] || 'web/dist';
const outDir = process.env.WEB_SMOKE_SHOTS || 'cinematic-shots';
const p1 = process.env.P1 || 'ichigo3d';
const p2 = process.env.P2 || 'kaito';

(async () => {
  const report = { phases: {}, problems: [] };
  const clock0 = Date.now();
  const session = await openMatch({
    dir,
    query: `e2e=1&p1=${p1}&p2=${p2}&modo=versus&luta=1`,
    settings: { mute: 1, round_time: 2 },
    shotDir: outDir,
    label: 'cinematic',
    videoDir: path.join(outDir, 'video'),
  });
  try {
    let shotIntro = false, shotVictory = false;
    for (let i = 0; i < 900; i++) {
      const s = await session.state();
      const rs = s?.clock?.round_state;
      if (rs !== undefined && !(rs in report.phases)) report.phases[rs] = Date.now() - clock0;
      if (rs === 1 && !shotIntro && s.clock.round_time > 40) { shotIntro = true; await session.shot('intro'); }
      if (rs === 2) await session.setKeys(new Set(['ArrowRight', ['q', 'w', 'a', 's'][i % 4]]));
      if (rs === 4 && !shotVictory && s.clock.round_time > 40) { shotVictory = true; await session.shot('victory'); }
      if (rs === 4 && s.clock.round_time > 150) break;
      await wait(250);
    }
    await session.setKeys(new Set());
    if (!(1 in report.phases)) report.problems.push('never saw the intro');
    if (!(4 in report.phases)) report.problems.push('never saw the victory');
    report.problems.push(...session.problems);
  } finally {
    await session.close();
  }
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, 'phases.json'), JSON.stringify(report.phases, null, 1));
  console.log(JSON.stringify(report));
  if (report.problems.length) process.exit(1);
})().catch(e => { console.error(e); process.exit(2); });
