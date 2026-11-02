import { describe, it, expect } from 'vitest';
import { ROSTER } from '../src/chars/index';
import { compileNative, STANDARD_ANIMS } from '../src/native/compile';
import { Match } from '../src/engine/match';
import { commonDef, flatStage } from '../src/engine/common';
import { K } from '../src/engine/input';

const factory = () => null;
const REQUIRED = ['idle', 'walk', 'walkBack', 'turn', 'crouchStart', 'crouch', 'crouchEnd', 'jumpStart', 'jumpUp', 'jumpFwd', 'jumpBack', 'jumpDown', 'land', 'run', 'hop',
  'guardStart', 'guard', 'crouchGuard', 'airGuard', 'guardEnd', 'guardHit', 'crouchGuardHit', 'airGuardHit', 'hitLight', 'hitMedium', 'hitHeavy', 'hitLowLight', 'hitLowMedium',
  'hitLowHeavy', 'hitCrouch', 'hitAir', 'fall', 'bounce', 'bounceAir', 'lieDown', 'getUp', 'dead', 'trip', 'airRecover', 'win', 'lose', 'intro'];

class Script { seq: number[] = []; poll() { return this.seq.shift() ?? 0; } }

describe('original roster', () => {
  it('has 10 unique fighters', () => {
    expect(ROSTER.length).toBe(10);
    expect(new Set(ROSTER.map((c) => c.id)).size).toBe(10);
  });

  for (const ch of ROSTER) {
    describe(ch.id, () => {
      const def = compileNative(ch, { factory });
      // mirrors compileNative's numbering: standard names keep their MUGEN number, others count up from 1000
      const custom = Object.keys(ch.anims).filter((n) => !(n in STANDARD_ANIMS));
      const animNo = (n: string) => (n in STANDARD_ANIMS ? STANDARD_ANIMS[n] : 1000 + custom.indexOf(n));

      it('compiles with metadata', () => {
        expect(def.name).toBe(ch.name);
        expect(ch.format).toBe('umf/1');
        expect(ch.palettes?.length).toBeGreaterThanOrEqual(2);
        expect(ch.winQuote && ch.description && ch.series && ch.color).toBeTruthy();
        // alternate palettes compile too
        expect(() => compileNative(ch, { factory, palNo: 2 })).not.toThrow();
        expect(() => compileNative(ch, { factory, palNo: 3 })).not.toThrow();
      });

      it('has every standard animation', () => {
        for (const name of REQUIRED) expect(ch.anims[name], `${ch.id}.${name}`).toBeDefined();
        for (const [name, no] of Object.entries(STANDARD_ANIMS)) if (ch.anims[name]) expect(def.anims.has(no), `${ch.id} anim ${name}`).toBe(true);
      });

      it('references only existing poses', () => {
        const poses = (ch.art as any).poses as Record<string, any>;
        for (const [name, a] of Object.entries(ch.anims)) for (const f of a.frames) if (typeof f.pose === 'string') expect(poses[f.pose], `${ch.id}.${name} pose ${f.pose}`).toBeDefined();
      });

      it('moves: anims exist, active frames have hitboxes, full command set', () => {
        const kinds = { normal: 0, special: 0, super: 0 };
        const simple = new Set<string>();
        for (const m of ch.moves) {
          kinds[m.kind]++;
          if (m.simple) simple.add(m.simple);
          const a = ch.anims[m.anim];
          expect(a, `${ch.id} move ${m.id} anim ${m.anim}`).toBeDefined();
          const compiled = def.anims.get(animNo(m.anim));
          expect(compiled, `${ch.id} compiled anim ${m.anim}`).toBeDefined();
          for (const h of m.hits ?? []) {
            expect(h.at).toBeGreaterThanOrEqual(1);
            expect(h.at).toBeLessThanOrEqual(a.frames.length);
            expect(compiled!.frames[h.at - 1].clsn1.length, `${ch.id} ${m.id} hit frame ${h.at}`).toBeGreaterThan(0);
          }
          for (const p of ([] as any[]).concat(m.projectile ?? [])) {
            expect(p.at).toBeLessThanOrEqual(a.frames.length);
            const pa = ch.anims[p.anim];
            expect(pa, `${ch.id} projectile anim ${p.anim}`).toBeDefined();
            expect(pa.frames.some((f) => !!f.effect)).toBe(true);
            if (p.hitAnim) expect(ch.anims[p.hitAnim]).toBeDefined();
          }
          if (m.kind !== 'normal') expect((m.hits?.length ?? 0) + ([] as any[]).concat(m.projectile ?? []).length).toBeGreaterThan(0);
        }
        expect(kinds.normal).toBeGreaterThanOrEqual(9);
        expect(kinds.special).toBeGreaterThanOrEqual(3);
        expect(kinds.super).toBeGreaterThanOrEqual(1);
        expect([...simple].sort()).toEqual(['sp1', 'sp2', 'sp3', 'super']);
        for (const btn of ['x', 'y', 'a', 'b']) expect(ch.moves.some((m) => m.input === btn && (m.when ?? 'stand') === 'stand'), `stand ${btn}`).toBe(true);
        for (const btn of ['x', 'a', 'b']) expect(ch.moves.some((m) => m.input === btn && m.when === 'crouch'), `crouch ${btn}`).toBe(true);
        expect(ch.moves.filter((m) => m.when === 'air').length).toBeGreaterThanOrEqual(2);
        const sup = ch.moves.find((m) => m.kind === 'super')!;
        expect(sup.cost).toBe(1000);
        expect(sup.input).toBe('~D, DF, F, D, DF, F, x');
      });

      it('idle has hurt boxes and a sane size', () => {
        const idle = def.anims.get(0)!;
        expect(idle.frames.length).toBeGreaterThanOrEqual(4);
        for (const f of idle.frames) {
          expect(f.clsn2.length).toBeGreaterThanOrEqual(2);
          expect(f.clsn1.length).toBe(0);
          const top = Math.min(...f.clsn2.map((b) => b[1])), bottom = Math.max(...f.clsn2.map((b) => b[3]));
          expect(bottom - top).toBeGreaterThanOrEqual(80);
          expect(bottom - top).toBeLessThanOrEqual(130);
          expect(bottom).toBeLessThanOrEqual(2);
          expect(bottom).toBeGreaterThanOrEqual(-2);
          for (const b of f.clsn2) { expect(b[0]).toBeLessThan(b[2]); expect(b[1]).toBeLessThan(b[3]); }
          const spr = def.sprites.get(`${f.group},${f.index}`)!;
          const h = spr.height * (spr.scale ?? 1);
          expect(h).toBeGreaterThan(80);
          expect(h).toBeLessThan(160);
          expect(spr.axisY * (spr.scale ?? 1)).toBeGreaterThan(80);
        }
      });
    });
  }
});

describe('roster headless match', () => {
  const a = compileNative(ROSTER.find((c) => c.id === 'kaito')!, { factory });
  const b = compileNative(ROSTER.find((c) => c.id === 'shion')!, { factory, palNo: 2 });

  it('jab spam lands and specials trigger', () => {
    const m = new Match(commonDef(), null, flatStage());
    const s1 = new Script(), s2 = new Script();
    const p1 = m.addPlayer(a, 0, s1), p2 = m.addPlayer(b, 1, s2);
    m.startRound();
    for (let i = 0; i < 120 && m.roundState !== 2; i++) m.tick();
    for (let i = 0; i < 30; i++) m.tick();
    expect(m.roundState).toBe(2);
    expect(p1.ctrl).toBe(true);
    // walk in close
    for (let i = 0; i < 200 && Math.abs(p2.pos.x - p1.pos.x) > 45; i++) { s1.seq.push(p1.facing > 0 ? K.R : K.L); m.tick(); }
    s1.seq.length = 0;
    expect(Math.abs(p2.pos.x - p1.pos.x)).toBeLessThanOrEqual(46);
    const life0 = p2.life;
    let sawAttack = false;
    for (let n = 0; n < 8; n++) {
      s1.seq.push(K.x, 0, 0);
      for (let i = 0; i < 16; i++) { m.tick(); if (p1.moveType === 'A') sawAttack = true; }
      if (Math.abs(p2.pos.x - p1.pos.x) > 45) for (let i = 0; i < 10; i++) { s1.seq.push(p1.facing > 0 ? K.R : K.L); m.tick(); }
    }
    expect(sawAttack).toBe(true);
    expect(p2.life).toBeLessThan(life0);
    // special: ~D, DF, F, x (Gale Fist projectile)
    for (let i = 0; i < 40; i++) m.tick();
    const fwd = p1.facing > 0 ? K.R : K.L;
    s1.seq.push(K.D, K.D, K.D | fwd, K.D | fwd, fwd, fwd | K.x, 0);
    const seen = new Set<number>();
    let proj = false;
    for (let i = 0; i < 60; i++) { m.tick(); seen.add(p1.stateNo); if ((m as any).projectiles?.length) proj = true; }
    expect([...seen].some((s) => s >= 1000 && s < 3000)).toBe(true);
    expect(proj).toBe(true);
    // super: ~D, DF, F, D, DF, F, x with a full bar
    for (let i = 0; i < 90; i++) m.tick();
    p1.power = 3000;
    s1.seq.push(K.D, K.D, K.D | fwd, K.D | fwd, fwd, fwd, K.D, K.D, K.D | fwd, K.D | fwd, fwd, fwd | K.x, 0);
    const seen2 = new Set<number>();
    for (let i = 0; i < 40; i++) { m.tick(); seen2.add(p1.stateNo); }
    expect(seen2.has(3000)).toBe(true);
    expect(p1.power).toBeLessThan(3000);
  });
});
