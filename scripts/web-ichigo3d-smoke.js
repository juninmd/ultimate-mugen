const fs = require('fs');
const path = require('path');
const { openMatch, wait } = require('./web-match-driver');

const dir = process.argv[2] || 'web/dist';
const outDir = process.env.WEB_SMOKE_SHOTS || 'ichigo3d-shots';
const FWD = 'ArrowRight', BACK = 'ArrowLeft', DOWN = 'ArrowDown';
const keys = (...k) => new Set(k);

async function untilFight(session, extra = () => true) {
  for (let i = 0; i < 400; i++) {
    const s = await session.state();
    if (s && s.screen === 'fight' && s.clock?.round_state === 2 && extra(s)) return s;
    await wait(300);
  }
  throw new Error('never reached a running fight');
}

const marks = [];
let clock0 = Date.now();

async function capture(session, name, count) {
  const start = Date.now() - clock0;
  await wait(count * 60);
  marks.push({ name, start, end: Date.now() - clock0 });
  await session.shot(name);
}

async function motion(session, steps) {
  for (const step of steps) {
    await session.setKeys(step);
    await session.frames(1);
  }
}

const QCF = button => [keys(DOWN), keys(DOWN, FWD), keys(FWD), keys(FWD, button)];
const QCB = button => [keys(DOWN), keys(DOWN, BACK), keys(BACK), keys(BACK, button)];

async function player(session) {
  const s = await session.state();
  return s.fighters.find(f => f.side === 0);
}

async function performMoves(session, report) {
  const moves = [
    { name: 'slash-combo', steps: [keys('q'), keys(), keys('q'), keys(), keys('w')], shots: 14 },
    { name: 'getsuga', steps: QCF('q'), shots: 16 },
    { name: 'getsuga-charged', steps: QCB('w'), hold: keys('w'), holdShots: 22, shots: 16 },
    { name: 'cero-charged', steps: QCB('a'), hold: keys('a'), holdShots: 20, shots: 14 },
    { name: 'shunpo', steps: QCB('s'), shots: 14 },
  ];
  for (const move of moves) {
    await session.setKeys(keys());
    await wait(500);
    await motion(session, move.steps);
    const states = [(await player(session))?.state];
    if (move.hold) {
      await session.setKeys(move.hold);
      await capture(session, `${move.name}-hold`, move.holdShots);
      states.push((await player(session))?.state);
    }
    await session.setKeys(keys());
    await capture(session, move.name, move.shots);
    states.push((await player(session))?.state);
    report.moves.push({ name: move.name, states });
  }
}

async function training(report) {
  clock0 = Date.now();
  const session = await openMatch({ dir, query: 'e2e=1&p1=ichigo3d&p2=kaito&modo=treino&luta=1', settings: { mute: 1 }, shotDir: outDir, label: 'training', videoDir: path.join(outDir, 'video-training') });
  try {
    await untilFight(session);
    await session.setKeys(keys(FWD));
    await wait(700);
    await session.setKeys(keys());
    await capture(session, 'idle', 10);
    await performMoves(session, report);
    await session.tap('t');
    await untilFight(session, s => s.fighters.find(f => f.side === 0)?.form === 1 && s.fighters.find(f => f.side === 0)?.state !== 19902);
    await session.frames(30);
    await motion(session, QCB('s'));
    await session.setKeys(keys());
    await capture(session, 'sonido', 14);
    const t0 = Date.now();
    await session.frames(120);
    report.meanFrameMs = Number(((Date.now() - t0) / 120).toFixed(2));
    report.problems.push(...session.problems);
  } finally {
    await session.close();
  }
}

async function guard(report) {
  clock0 = Date.now();
  marks.push({ name: 'versus-start', start: 0, end: 0 });
  const session = await openMatch({ dir, query: 'e2e=1&p1=ichigo3d&p2=kaito&modo=versus&luta=1', settings: { mute: 1, round_time: 99 }, shotDir: outDir, label: 'guard', videoDir: path.join(outDir, 'video-versus') });
  try {
    await untilFight(session);
    await wait(2000);
    const start = Date.now() - clock0;
    report.guardStates = [];
    for (let k = 0; k < 60; k++) {
      if (k % 20 === 0) await motion(session, [keys(BACK, 'k'), keys(BACK, 'k', 'j'), keys(BACK, 'j'), keys(BACK, 'j', 'Numpad4')]);
      await session.setKeys(keys(BACK));
      await session.frames(1);
      const state = (await player(session))?.state;
      if (state >= 120 && state <= 155 && !report.guardStates.includes(state)) report.guardStates.push(state);
    }
    marks.push({ name: 'guard', start, end: Date.now() - clock0 });
    await session.setKeys(keys());
    if (!report.guardStates.length) report.problems.push('Ichigo never entered a guard state');
    report.problems.push(...session.problems);
  } finally {
    await session.close();
  }
}

(async () => {
  const report = { moves: [], problems: [] };
  await training(report);
  await guard(report);
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, 'marks.json'), JSON.stringify(marks, null, 1));
  console.log(JSON.stringify(report));
  if (report.problems.length) process.exit(1);
})().catch(e => { console.error(e); process.exit(2); });
