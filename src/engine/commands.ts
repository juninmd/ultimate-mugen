import { InputHistory, K, relDirs } from './input';

/** One symbol of a MUGEN command string. */
export interface CmdSymbol {
  release: boolean; // ~
  hold: boolean; // /
  four: boolean; // $
  strict: boolean; // >
  charge: number; // ~30a
  dir: number; // relative dir mask U=1 D=2 B=4 F=8 (0 if button symbol)
  btns: number; // absolute button bits
}

export interface CommandDef {
  name: string;
  steps: CmdSymbol[];
  time: number;
  bufferTime: number;
}

const DIRS: Record<string, number> = { U: 1, D: 2, B: 4, F: 8, UB: 5, UF: 9, DB: 6, DF: 10 };

export function parseCommand(name: string, str: string, time = 15, bufferTime = 1): CommandDef {
  const steps: CmdSymbol[] = [];
  for (let part of str.split(',')) {
    part = part.trim();
    if (!part) continue;
    const s: CmdSymbol = { release: false, hold: false, four: false, strict: false, charge: 0, dir: 0, btns: 0 };
    for (;;) {
      const c = part[0];
      if (c === '>') { s.strict = true; part = part.slice(1).trim(); }
      else if (c === '/') { s.hold = true; part = part.slice(1).trim(); }
      else if (c === '$') { s.four = true; part = part.slice(1).trim(); }
      else if (c === '~') {
        s.release = true;
        part = part.slice(1).trim();
        const m = /^(\d+)/.exec(part);
        if (m) { s.charge = parseInt(m[1]); part = part.slice(m[1].length).trim(); }
      } else break;
    }
    for (let tok of part.split('+')) {
      tok = tok.trim();
      // modifiers may appear inside a+ combos too
      while (tok && /[\/$~>]/.test(tok[0])) {
        if (tok[0] === '/') s.hold = true; else if (tok[0] === '$') s.four = true; else if (tok[0] === '>') s.strict = true; else s.release = true;
        tok = tok.slice(1);
      }
      if (DIRS[tok] !== undefined) s.dir |= DIRS[tok];
      else if ((K as any)[tok] !== undefined && tok.length === 1) s.btns |= (K as any)[tok];
    }
    steps.push(s);
  }
  return { name, steps, time: time > 0 ? time : 15, bufferTime: Math.max(1, bufferTime) };
}

function dirMatches(rel: number, s: CmdSymbol): boolean {
  if (s.four) {
    // 4-way: required single direction must be present
    return (rel & s.dir) === s.dir;
  }
  // exact (8-way): direction must match exactly, ignoring nothing
  return rel === s.dir;
}

/** Is the symbol "held/satisfied" at tick t (state, not event). */
function symHeld(h: InputHistory, t: number, s: CmdSymbol): boolean {
  const bits = h.at(t);
  if (s.dir) {
    const rel = relDirs(bits, h.faceAt(t));
    if (!dirMatches(rel, s)) return false;
  }
  if (s.btns && (bits & s.btns) !== s.btns) return false;
  return true;
}

function heldDuration(h: InputHistory, t: number, s: CmdSymbol): number {
  let n = 0;
  while (t - n >= 0 && n < 240 && symHeld(h, t - n, s)) n++;
  return n;
}

/** Does symbol s "fire" at tick t? */
function symFires(h: InputHistory, t: number, s: CmdSymbol): boolean {
  if (s.hold) return symHeld(h, t, s);
  if (s.release) {
    if (symHeld(h, t, s) || !symHeld(h, t - 1, s)) return false;
    return s.charge ? heldDuration(h, t - 1, s) >= s.charge : true;
  }
  if (s.btns) {
    // simultaneous press: all held now, and each pressed within last 2 ticks, at least one pressed now
    if (!symHeld(h, t, s)) return false;
    const now = h.at(t), prev = h.at(t - 1);
    const pressedNow = s.btns & now & ~prev;
    if (!pressedNow) return false;
    const p2 = h.at(t - 2), p3 = h.at(t - 3);
    for (let b = 1; b <= s.btns; b <<= 1) {
      if (!(s.btns & b)) continue;
      if (!((now & ~prev) & b) && !((prev & ~p2) & b) && !((p2 & ~p3) & b)) return false;
    }
    return true;
  }
  // direction press: held now and not held previous tick
  return symHeld(h, t, s) && !symHeld(h, t - 1, s);
}

/** Backward matcher: true if command completed with its last symbol within bufferTime ticks. */
export function matchCommand(h: InputHistory, cmd: CommandDef): boolean {
  const n = cmd.steps.length;
  if (!n) return false;
  const T = h.tick;
  const last = cmd.steps[n - 1];
  for (let tEnd = T; tEnd > T - cmd.bufferTime && tEnd >= 0; tEnd--) {
    if (!symFires(h, tEnd, last)) continue;
    if (n === 1) return true;
    // the earliest allowed tick for first step
    const limit = tEnd - cmd.time;
    let t = tEnd;
    let ok = true;
    for (let i = n - 2; i >= 0; i--) {
      const s = cmd.steps[i];
      const strictNext = cmd.steps[i + 1].strict;
      let found = -1;
      for (let tt = t - (s.hold || s.release ? 0 : 1); tt >= Math.max(0, limit); tt--) {
        if (symFires(h, tt, s)) { found = tt; break; }
        if (strictNext && tt < t && h.at(tt) !== h.at(tt - 1) && !s.hold) { break; }
      }
      if (found < 0) { ok = false; break; }
      t = found;
    }
    if (ok) return true;
  }
  return false;
}

export class CommandState {
  active = new Set<string>();
  constructor(public cmds: CommandDef[]) {}
  update(h: InputHistory) {
    this.active.clear();
    for (const c of this.cmds) if (!this.active.has(c.name) && matchCommand(h, c)) this.active.add(c.name);
  }
}
