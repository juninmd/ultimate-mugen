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
