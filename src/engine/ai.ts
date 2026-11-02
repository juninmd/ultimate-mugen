import type { Fighter } from './fighter';
import type { InputSource } from './input';
import { K } from './input';
import type { CommandDef } from './commands';

/** CPU opponent: works for any character (native or MUGEN) by replaying command definitions. */
export class AIController implements InputSource {
  f: Fighter | null = null;
  queue: number[] = [];
  hold = 0;
  holdTicks = 0;
  cooldown = 0;
  level: number;
  attackCmds: CommandDef[] = [];
  constructor(level = 4) { this.level = level; }
  attach(f: Fighter) {
    this.f = f;
    const names = new Set(f.def.aiCommands ?? []);
    this.attackCmds = f.def.commands.filter((c) => names.has(c.name) && c.steps.length > 0 && c.steps.some((s) => s.btns));
  }
  private abs(rel: number, facing: number): number {
    let b = rel & 3; // U D
    const back = rel & 4, fwd = rel & 8;
    if (facing >= 0) { if (back) b |= K.L; if (fwd) b |= K.R; } else { if (back) b |= K.R; if (fwd) b |= K.L; }
    return b;
  }
  private seqFor(c: CommandDef, facing: number): number[] {
    const out: number[] = [];
    for (let i = 0; i < c.steps.length; i++) {
      const s = c.steps[i];
      const d = this.abs(s.dir, facing);
      if (s.release) {
        out.push(d | s.btns, d | s.btns);
        if (s.charge) for (let k = 0; k < s.charge; k++) out.push(d | s.btns);
        continue; // next step releases it
      }
      if (s.btns) { const prevDir = out.length ? out[out.length - 1] & 15 : 0; out.push((d || prevDir) | s.btns, 0); }
      else { out.push(d, d); }
    }
    return out;
  }
  poll(): number {
    const f = this.f;
    if (!f) return 0;
    const m = f.match;
    if (this.queue.length) return this.queue.shift()!;
    if (m.roundState !== 2) return 0;
    const e = f.enemy(0);
    if (!e) return 0;
    if (this.holdTicks > 0) { this.holdTicks--; return this.hold; }
    const dist = Math.abs(e.pos.x - f.pos.x);
    const fwd = f.facing > 0 ? K.R : K.L, back = f.facing > 0 ? K.L : K.R;
    const skill = this.level / 8;
    const r = Math.random();
    // react to attacks: guard
    if ((e.moveType === 'A' || m.projectiles.some((p) => p.owner.root === e)) && dist < 140 && r < 0.25 + skill * 0.6 && f.moveType !== 'H') {
      this.hold = back | (e.stateType === 'C' || Math.random() < 0.3 ? K.D : 0);
      this.holdTicks = 8 + Math.floor(Math.random() * 10);
      return this.hold;
    }
    if (this.cooldown > 0) { this.cooldown--; return this.idleMove(dist, fwd, back); }
    if (!f.ctrl) return 0;
    // attack decision
    const cmds = this.attackCmds;
    if (cmds.length && (dist < 70 || (dist > 110 && Math.random() < 0.08 + skill * 0.06))) {
      if (Math.random() < 0.2 + skill * 0.5) {
        const pool = dist < 70 ? cmds : cmds.filter((c) => c.steps.length > 1);
        const c = pool[Math.floor(Math.random() * pool.length)];
        if (c) {
          this.queue = this.seqFor(c, f.facing);
          this.cooldown = Math.floor(20 + (1 - skill) * 40 + Math.random() * 20);
          return this.queue.shift() ?? 0;
        }
      }
    }
    if (dist > 60 && Math.random() < 0.02 + skill * 0.02) { this.queue = [K.U | fwd, K.U | fwd, fwd]; this.cooldown = 30; return this.queue.shift()!; }
    return this.idleMove(dist, fwd, back);
  }
  private idleMove(dist: number, fwd: number, back: number): number {
    if (dist > 80) { this.hold = fwd; this.holdTicks = 6 + Math.floor(Math.random() * 10); return fwd; }
    if (dist < 35 && Math.random() < 0.1) { this.hold = back; this.holdTicks = 8; return back; }
    if (Math.random() < 0.03) { this.hold = K.D; this.holdTicks = 10; return K.D; }
    return 0;
  }
}
