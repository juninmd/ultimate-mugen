import { describe, it, expect } from 'vitest';
// @ts-ignore
import fs from 'node:fs';
// @ts-ignore
import path from 'node:path';
import { loadMugenCharacter, normPath, type VFS } from '../src/mugen/loader';
import { Match } from '../src/engine/match';
import { commonDef, flatStage } from '../src/engine/common';
import { K } from '../src/engine/input';

const SP = '/tmp/claude-0/-home-user-ultimate-mugen/fa14b17d-4289-5fed-86c2-aacfb04ddae6/scratchpad/sp';
const KFM = path.join(SP, 'chars/kfm');

function vfsFromDir(dir: string, prefix: string): VFS {
  const v: VFS = new Map();
  for (const f of fs.readdirSync(dir)) v.set(normPath(`${prefix}/${f}`), new Uint8Array(fs.readFileSync(path.join(dir, f))));
  return v;
}

class Script { seq: number[] = []; poll() { return this.seq.shift() ?? 0; } }

describe.skipIf(!fs.existsSync(KFM))('KFM headless match', () => {
  const vfs = vfsFromDir(KFM, 'kfm');
  const def = loadMugenCharacter(vfs, 'kfm/kfm.def', { makeImage: () => null });

  it('loads kfm', () => {
    expect(def.name).toBe('Kung Fu Man');
    expect(def.anims.size).toBeGreaterThan(50);
    expect(def.states.size).toBeGreaterThan(30);
    expect(def.commands.length).toBeGreaterThan(20);
    expect(def.sprites.size).toBeGreaterThan(100);
  });

  it('plays a round: walk, punch hits, life decreases', () => {
    const m = new Match(commonDef(), null, flatStage());
    const s1 = new Script(), s2 = new Script();
    const p1 = m.addPlayer(def, 0, s1), p2 = m.addPlayer(def, 1, s2);
    m.startRound();
    for (let i = 0; i < 120; i++) m.tick();
    expect(m.roundState).toBe(2);
    expect(p1.stateNo).toBe(0);
    expect(p1.ctrl).toBe(true);
    // walk forward
    for (let i = 0; i < 40; i++) s1.seq.push(K.R);
    const x0 = p1.pos.x;
    for (let i = 0; i < 20; i++) m.tick();
    expect(p1.stateNo).toBe(20);
    expect(p1.pos.x).toBeGreaterThan(x0 + 10);
    for (let i = 0; i < 25; i++) m.tick();
    // jab
    s1.seq.push(K.x, 0, 0);
    const lifeBefore = p2.life;
    let sawAttack = false;
    for (let i = 0; i < 40; i++) { m.tick(); if (p1.moveType === 'A') sawAttack = true; }
    expect(sawAttack).toBe(true);
    console.log('p1', p1.stateNo, p1.pos.x, 'p2', p2.stateNo, p2.pos.x, p2.life);
    expect(p2.life).toBeLessThan(lifeBefore);
    // jump
    for (let i = 0; i < 60; i++) m.tick();
    s1.seq.push(K.U, K.U, K.U);
    let minY = 0;
    for (let i = 0; i < 60; i++) { m.tick(); minY = Math.min(minY, p1.pos.y); }
    expect(minY).toBeLessThan(-40);
    for (let i = 0; i < 30; i++) m.tick();
    expect(p1.pos.y).toBe(0);
    // fireball: D, DF, F, x
    for (let i = 0; i < 60; i++) m.tick();
    s1.seq.push(K.D, K.D, K.D | K.R, K.D | K.R, K.R, K.R | K.x, 0);
    const seen = new Set<number>();
    for (let i = 0; i < 60; i++) { m.tick(); seen.add(p1.stateNo); }
    expect(seen.has(1000)).toBe(true); // Kung Fu Palm
    // match goes on until KO with repeated palms
    for (let n = 0; n < 40 && m.roundState === 2; n++) {
      for (let i = 0; i < 50; i++) m.tick();
      s1.seq.push(K.D, K.D, K.D | K.R, K.D | K.R, K.R, K.R | K.x, 0);
      if (Math.abs(p1.pos.x - p2.pos.x) > 70) for (let i = 0; i < 20; i++) s1.seq.unshift(p1.facing > 0 ? K.R : K.L);
    }
    expect(p2.life).toBeLessThan(500);
  });
});

describe.skipIf(!fs.existsSync(KFM))('KFM advanced', () => {
  const vfs = vfsFromDir(KFM, 'kfm');
  const def = loadMugenCharacter(vfs, 'kfm/kfm.def', { makeImage: () => null });
  it('throw puts P2 in a custom state and damages', () => {
    const m = new Match(commonDef(), null, flatStage());
    const s1 = new Script(), s2 = new Script();
    const p1 = m.addPlayer(def, 0, s1), p2 = m.addPlayer(def, 1, s2);
    m.startRound();
    for (let i = 0; i < 110; i++) m.tick();
    for (let i = 0; i < 60; i++) s1.seq.push(K.R);
    for (let i = 0; i < 60; i++) m.tick();
    s1.seq.push(K.R | K.y, K.R);
    let custom = false;
    const st = new Set<number>();
    for (let i = 0; i < 120; i++) { m.tick(); st.add(p1.stateNo); if (p2.stDef === p1.def && p2.stateNo >= 800) custom = true; }
    console.log('p1 states', [...st].join(','), 'p2 life', p2.life);
    expect(custom).toBe(true);
    expect(p2.life).toBeLessThan(1000);
  });
  it('super: Triple Kung Fu Palm with power', () => {
    const m = new Match(commonDef(), null, flatStage());
    const s1 = new Script(), s2 = new Script();
    const p1 = m.addPlayer(def, 0, s1), p2 = m.addPlayer(def, 1, s2);
    m.startRound();
    for (let i = 0; i < 110; i++) m.tick();
    p1.power = 3000;
    for (let i = 0; i < 40; i++) s1.seq.push(K.R);
    for (let i = 0; i < 40; i++) m.tick();
    const qcf = [K.D, K.D, K.D | K.R, K.D | K.R, K.R, K.R];
    s1.seq.push(...qcf, 0, ...qcf.slice(0, 5), K.R | K.x, 0);
    let paused = false; const st = new Set<number>();
    for (let i = 0; i < 150; i++) { m.tick(); if (m.pauseState) paused = true; st.add(p1.stateNo); }
    console.log('super states', [...st].join(','), 'power', p1.power, 'p2 life', p2.life);
    expect(paused).toBe(true);
    expect(p1.power).toBeLessThan(3000);
  });
});

const KFM720 = path.join(SP, 'chars/kfm720');
describe.skipIf(!fs.existsSync(KFM720))('HD character (localcoord 1280x720)', () => {
  it('kfm720 moves/jumps at the same engine scale as kfm', () => {
    const d1 = loadMugenCharacter(vfsFromDir(KFM, 'kfm'), 'kfm/kfm.def', { makeImage: () => null });
    const d2 = loadMugenCharacter(vfsFromDir(KFM720, 'kfm720'), 'kfm720/kfm720.def', { makeImage: () => null });
    expect(d2.localW).toBe(1280);
    const m = new Match(commonDef(), null, flatStage());
    const s1 = new Script(), s2 = new Script();
    const a = m.addPlayer(d1, 0, s1), b = m.addPlayer(d2, 1, s2);
    m.startRound();
    for (let i = 0; i < 110; i++) m.tick();
    const ax = a.pos.x, bx = b.pos.x;
    for (let i = 0; i < 30; i++) { s1.seq.push(K.R); s2.seq.push(K.L); }
    for (let i = 0; i < 30; i++) m.tick();
    const da = a.pos.x - ax, db = bx - b.pos.x;
    console.log('walk dist', da.toFixed(1), db.toFixed(1));
    expect(Math.abs(da - db)).toBeLessThan(15);
    let minY = 0;
    s2.seq.push(K.U, K.U);
    for (let i = 0; i < 60; i++) { m.tick(); minY = Math.min(minY, b.pos.y); }
    console.log('jump height hd', minY.toFixed(1));
    expect(minY).toBeLessThan(-50); expect(minY).toBeGreaterThan(-120);
    // HD hitboxes scaled
    const hb = b.boxes(2);
    const hgt = Math.max(...hb.map((x) => b.pos.y - x[1]));
    expect(hgt).toBeGreaterThan(60); expect(hgt).toBeLessThan(140);
  });
});
