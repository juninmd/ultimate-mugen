import { stripComment } from './ini';

export type Box = [number, number, number, number]; // x1,y1,x2,y2 (normalized: x1<x2, y1<y2)

export interface AnimFrame {
  group: number;
  index: number;
  x: number;
  y: number;
  time: number; // -1 = infinite
  flipH: boolean;
  flipV: boolean;
  blend: 'none' | 'add' | 'sub' | 'add1';
  alpha?: [number, number];
  xscale: number;
  yscale: number;
  angle: number;
  clsn1: Box[];
  clsn2: Box[];
}

export interface Animation {
  id: number;
  frames: AnimFrame[];
  loopStart: number;
  /** Sum of frame times (-1 if any infinite). */
  totalTime: number;
}

function normBox(a: number[]): Box {
  const [x1, y1, x2, y2] = a;
  return [Math.min(x1, x2), Math.min(y1, y2), Math.max(x1, x2), Math.max(y1, y2)];
}

export function computeTotal(anim: Animation) {
  let t = 0;
  for (const f of anim.frames) {
    if (f.time < 0) { t = -1; break; }
    t += f.time;
  }
  anim.totalTime = t;
}

export function parseAir(text: string): Map<number, Animation> {
  const anims = new Map<number, Animation>();
  let cur: Animation | null = null;
  let def1: Box[] = [], def2: Box[] = [];
  let next1: Box[] | null = null, next2: Box[] | null = null;
  let pending: { kind: 1 | 2; isDefault: boolean; list: Box[] } | null = null;
  const lines = text.replace(/^﻿/, '').split(/\r?\n|\r/);
  for (const raw of lines) {
    const line = stripComment(raw).trim();
    if (!line) continue;
    const ll = line.toLowerCase();
    if (ll.startsWith('[')) {
      const m = /begin\s+action\s+(-?\d+)/i.exec(line);
      if (cur) computeTotal(cur);
      if (m) {
        cur = { id: parseInt(m[1]), frames: [], loopStart: 0, totalTime: 0 };
        anims.set(cur.id, cur);
      } else cur = null;
      def1 = []; def2 = []; next1 = next2 = null; pending = null;
      continue;
    }
    if (!cur) continue;
    let m: RegExpExecArray | null;
    if ((m = /^clsn([12])(default)?\s*:\s*(\d+)/i.exec(line))) {
      const kind = m[1] === '1' ? 1 : 2;
      const isDefault = !!m[2];
      const list: Box[] = [];
      pending = { kind, isDefault, list };
      if (isDefault) { if (kind === 1) def1 = list; else def2 = list; }
      else { if (kind === 1) next1 = list; else next2 = list; }
      continue;
    }
    if ((m = /^clsn([12])\s*\[\s*\d+\s*\]\s*=\s*(.+)$/i.exec(line))) {
      const vals = m[2].split(',').map((s) => parseFloat(s));
      const kind = m[1] === '1' ? 1 : 2;
      const box = normBox(vals);
      if (pending && pending.kind === kind) pending.list.push(box);
      else {
        // tolerate missing header
        if (kind === 1) (next1 ??= []).push(box); else (next2 ??= []).push(box);
      }
      continue;
    }
    if (ll.startsWith('loopstart')) { cur.loopStart = cur.frames.length; continue; }
    if (ll.startsWith('interpolate')) continue;
    if (/^-?\d/.test(line)) {
      const p = line.split(',').map((s) => s.trim());
      const fl = (p[5] ?? '').toUpperCase();
      const bl = (p[6] ?? '').toUpperCase();
      let blend: AnimFrame['blend'] = 'none';
      let alpha: [number, number] | undefined;
      if (bl.startsWith('A1')) blend = 'add1';
      else if (bl.startsWith('AS')) {
        blend = 'add';
        const mm = /AS(\d+)D(\d+)/.exec(bl);
        if (mm) alpha = [parseInt(mm[1]), parseInt(mm[2])];
      } else if (bl.startsWith('A')) blend = 'add';
      else if (bl.startsWith('S')) blend = 'sub';
      const f: AnimFrame = {
        group: parseInt(p[0]) || 0,
        index: parseInt(p[1]) || 0,
        x: parseFloat(p[2]) || 0,
        y: parseFloat(p[3]) || 0,
        time: parseInt(p[4]),
        flipH: fl.includes('H'),
        flipV: fl.includes('V'),
        blend, alpha,
        xscale: p[7] ? parseFloat(p[7]) || 1 : 1,
        yscale: p[8] ? parseFloat(p[8]) || 1 : 1,
        angle: p[9] ? parseFloat(p[9]) || 0 : 0,
        clsn1: next1 ?? def1,
        clsn2: next2 ?? def2,
      };
      if (!Number.isFinite(f.time)) f.time = 1;
      cur.frames.push(f);
      next1 = next2 = null; pending = null;
    }
  }
  if (cur) computeTotal(cur);
  for (const a of anims.values()) {
    if (a.loopStart >= a.frames.length) a.loopStart = 0;
    computeTotal(a);
  }
  return anims;
}
