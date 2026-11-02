/**
 * Procedural character art: a 2D skeletal rig rendered with an anime / fighting-game look
 * (bold outlines, two-tone cel shading with a rim light, flowing hair & cloth), plus an
 * effect renderer for projectiles and super moves.
 *
 * Conventions (engine units): the feet / ground point is the axis (0,0), the fighter faces RIGHT,
 * y grows downwards (negative = up). Canvases are rendered at RES x and returned with scale 1/RES.
 * All geometry (hit/hurt boxes, size, axis) is computed without a canvas so that headless code
 * (tests, servers) gets identical collision data.
 */
import type { ActiveSpec, Box } from './types';

export type CanvasFactory = (w: number, h: number) => { canvas: CanvasImageSource; ctx: CanvasRenderingContext2D } | null;
export interface RigRender { canvas: CanvasImageSource | null; width: number; height: number; axisX: number; axisY: number; scale: number; hurt: Box[]; hit: Box[] }

const RES = 3;
const PAD = 7;

// ------------------------------------------------------------------ math
interface P { x: number; y: number }
const D2R = Math.PI / 180;
/** direction for an angle in degrees: 0 = down, 90 = forward (+x), 180 = up */
const dir = (a: number): P => ({ x: Math.sin(a * D2R), y: Math.cos(a * D2R) });
const add = (p: P, q: P, s = 1): P => ({ x: p.x + q.x * s, y: p.y + q.y * s });
const sub = (p: P, q: P): P => ({ x: p.x - q.x, y: p.y - q.y });
const mul = (p: P, s: number): P => ({ x: p.x * s, y: p.y * s });
const len = (p: P) => Math.hypot(p.x, p.y);
const norm = (p: P): P => { const l = len(p) || 1; return { x: p.x / l, y: p.y / l }; };
const lerpP = (a: P, b: P, t: number): P => ({ x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t });
/** perpendicular (rotated +90 deg on screen: (x,y) -> (-y,x)) */
const perp = (p: P): P => ({ x: -p.y, y: p.x });
const angOf = (p: P) => Math.atan2(p.x, p.y) / D2R; // inverse of dir()
const num = (v: any, d: number) => (typeof v === 'number' && isFinite(v) ? v : d);
function hash(n: number) { const s = Math.sin(n * 127.1 + 311.7) * 43758.5453; return s - Math.floor(s); }

// ------------------------------------------------------------------ colour
function rgb(c: string): [number, number, number] {
  let h = (c || '#888').trim();
  if (h[0] === '#') h = h.slice(1);
  if (h.length === 3) h = h.split('').map((ch) => ch + ch).join('');
  const v = parseInt(h.slice(0, 6), 16);
  if (isNaN(v)) return [136, 136, 136];
  return [(v >> 16) & 255, (v >> 8) & 255, v & 255];
}
const toHex = (r: number, g: number, b: number) => '#' + [r, g, b].map((x) => Math.max(0, Math.min(255, Math.round(x))).toString(16).padStart(2, '0')).join('');
function mix(a: string, b: string, t: number): string { const x = rgb(a), y = rgb(b); return toHex(x[0] + (y[0] - x[0]) * t, x[1] + (y[1] - x[1]) * t, x[2] + (y[2] - x[2]) * t); }
function rgba(c: string, a: number) { const x = rgb(c); return `rgba(${x[0]},${x[1]},${x[2]},${a})`; }
const lum = (c: string) => { const x = rgb(c); return (0.3 * x[0] + 0.59 * x[1] + 0.11 * x[2]) / 255; };
const shadeOf = (c: string) => mix(c, '#2b1748', 0.42);
const lightOf = (c: string) => mix(c, '#fffaf0', 0.38);
const backOf = (c: string) => mix(c, '#120a22', 0.3);

const DEFAULT_COLORS: Record<string, string> = {
  skin: '#f2c49b', hair: '#2a2230', eye: '#3a6fd8', top: '#e8e4dc', top2: '#b02a2a', trim: '#1f1f2a',
  pants: '#e8e4dc', pants2: '#2a2a3a', belt: '#1c1c22', glove: '#b02a2a', boot: '#3a2a22', weapon: '#c9d3e0',
  weapon2: '#3a3a48', hilt: '#5a1f1f', aura: '#7fd4ff', energy: '#7fd4ff', accent: '#ffcc33', outline: '#160e1e',
  horn: '#efe2c4', metal: '#9aa6b8', visor: '#59f0ff', cloth: '#b02a2a', mask: '#2a2a33', hat: '#2a2030', lips: '#c0606a',
};

// ------------------------------------------------------------------ shapes
type PathFn = (ctx: CanvasRenderingContext2D, ox: number, oy: number) => void;
interface Shape { path: PathFn; fill: string; shade?: number; rim?: number; line?: boolean; noOutline?: boolean; flat?: boolean; glow?: string; alpha?: number }
interface Layer { shapes: Shape[]; after?: (ctx: CanvasRenderingContext2D) => void; before?: (ctx: CanvasRenderingContext2D) => void; aura?: string | null; outline?: number }

class BBox {
  x1 = Infinity; y1 = Infinity; x2 = -Infinity; y2 = -Infinity;
  addP(p: P, r = 0) { this.x1 = Math.min(this.x1, p.x - r); this.y1 = Math.min(this.y1, p.y - r); this.x2 = Math.max(this.x2, p.x + r); this.y2 = Math.max(this.y2, p.y + r); }
  addPts(ps: P[], r = 0) { for (const p of ps) this.addP(p, r); }
  valid() { return this.x1 < this.x2 && this.y1 < this.y2; }
}

function polyPath(pts: P[], smooth = false): PathFn {
  return (ctx, ox, oy) => {
    if (pts.length < 2) return;
    if (!smooth || pts.length < 3) {
      ctx.moveTo(pts[0].x + ox, pts[0].y + oy);
      for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i].x + ox, pts[i].y + oy);
      ctx.closePath();
      return;
    }
    const n = pts.length;
    const m0 = lerpP(pts[n - 1], pts[0], 0.5);
    ctx.moveTo(m0.x + ox, m0.y + oy);
    for (let i = 0; i < n; i++) {
      const p = pts[i], q = pts[(i + 1) % n];
      const m = lerpP(p, q, 0.5);
      ctx.quadraticCurveTo(p.x + ox, p.y + oy, m.x + ox, m.y + oy);
    }
    ctx.closePath();
  };
}
/** polygon where some points are sharp (corner = true) and others smooth */
function mixedPath(pts: { p: P; c?: boolean }[]): PathFn {
  return (ctx, ox, oy) => {
    const n = pts.length;
    if (n < 3) return;
    const mid = (i: number) => lerpP(pts[i % n].p, pts[(i + 1) % n].p, 0.5);
    // start at a corner if any
    let s = pts.findIndex((q) => q.c);
    if (s < 0) s = 0;
    const st = pts[s].c ? pts[s].p : mid(s);
    ctx.moveTo(st.x + ox, st.y + oy);
    for (let k = 1; k <= n; k++) {
      const i = (s + k) % n;
      const q = pts[i];
      if (q.c) ctx.lineTo(q.p.x + ox, q.p.y + oy);
      else {
        const nx = pts[(i + 1) % n];
        const e = nx.c ? nx.p : mid(i);
        ctx.quadraticCurveTo(q.p.x + ox, q.p.y + oy, e.x + ox, e.y + oy);
        if (nx.c) { /* line continues from e which is the corner itself */ }
      }
    }
    ctx.closePath();
  };
}
function capsulePath(a: P, b: P, ra: number, rb: number): PathFn {
  return (ctx, ox, oy) => {
    const th = Math.atan2(b.y - a.y, b.x - a.x);
    ctx.moveTo(a.x + ox + Math.cos(th + Math.PI / 2) * ra, a.y + oy + Math.sin(th + Math.PI / 2) * ra);
    ctx.arc(a.x + ox, a.y + oy, ra, th + Math.PI / 2, th + Math.PI * 1.5);
    ctx.arc(b.x + ox, b.y + oy, rb, th - Math.PI / 2, th + Math.PI / 2);
    ctx.closePath();
  };
}
function ellipsePath(c: P, rx: number, ry: number, rot = 0): PathFn {
  return (ctx, ox, oy) => { ctx.moveTo(c.x + ox + Math.cos(rot) * rx, c.y + oy + Math.sin(rot) * rx); ctx.ellipse(c.x + ox, c.y + oy, rx, ry, rot, 0, Math.PI * 2); };
}
/** tapered ribbon along a polyline */
function ribbonPts(spine: P[], w0: number, w1: number): P[] {
  const L: P[] = [], R: P[] = [];
  for (let i = 0; i < spine.length; i++) {
    const a = spine[Math.max(0, i - 1)], b = spine[Math.min(spine.length - 1, i + 1)];
    const n = norm(perp(sub(b, a)));
    const t = i / (spine.length - 1);
    const w = (w0 + (w1 - w0) * t) / 2;
    L.push(add(spine[i], n, w));
    R.push(add(spine[i], n, -w));
  }
  return [...L, ...R.reverse()];
}
/** waving spine from start in direction angle (deg, dir() convention) */
function waveSpine(start: P, ang: number, length: number, amp: number, phase: number, n = 8, curl = 0): P[] {
  const out: P[] = [];
  let p = start;
  out.push(p);
  for (let i = 1; i <= n; i++) {
    const t = i / n;
    const a = ang + Math.sin(phase + t * 4.2) * amp * t + curl * t;
    p = add(p, dir(a), length / n);
    out.push(p);
  }
  return out;
}

// ------------------------------------------------------------------ skeleton
interface Skel {
  b: number; r: number; fem: boolean; S: number;
  L: { thigh: number; shin: number; torso: number; neck: number; up: number; fore: number; hand: number; foot: number };
  rot: number; lean: number; torsoA: number; headA: number; headTh: number;
  hip: P; hipF: P; hipB: P; neck: P; head: P; shF: P; shB: P;
  elF: P; elB: P; wrF: P; wrB: P; handF: P; handB: P; upAF: number; upAB: number; foreAF: number; foreAB: number;
  knF: P; knB: P; anF: P; anB: P; thAF: number; thAB: number; shAF: number; shAB: number; ftAF: number; ftAB: number;
  up: P; fwd: P; torsoPts: P[]; footF: P[]; footB: P[];
  W: number; pose: Record<string, any>;
}

function torsoLocal(W: number, fem: boolean, Lt: number, shape: string): [number, number][] {
  const k = Lt / 29;
  const w = W;
  const pts: [number, number][] = fem
    ? [[-7.6 * w, -3], [-6.4 * w, 5], [-5.6 * w, 12], [-6.8 * w, 19], [-7.4 * w, 25], [-4.8 * w, 29.5], [-1.5, 31], [2.5 * w, 30.5], [6 * w, 27], [8.6 * w, 22], [8.4 * w, 17.5], [5.6 * w, 14.5], [5.6 * w, 9], [7.2 * w, 2], [6.2 * w, -3]]
    : [[-7.6 * w, -3], [-7 * w, 6], [-8.2 * w, 15], [-9.2 * w, 22], [-8.2 * w, 27.5], [-5 * w, 30.8], [-1.5, 31.6], [3 * w, 31], [7.6 * w, 27.5], [9.6 * w, 21.5], [8.4 * w, 15], [6.8 * w, 8], [7.4 * w, 1], [6.2 * w, -3]];
  if (shape === 'heavy') for (const p of pts) { if (p[1] > 2 && p[1] < 20) p[0] *= 1.12; }
  return pts.map(([v, u]) => [v, u * k]);
}

function solve(style: Record<string, any>, pose: Record<string, any>): Skel {
  const S = num(style.scale, 1);
  const b = num(style.build, 1);
  const fem = !!style.female;
  const pr = style.proportions ?? {};
  const L = {
    thigh: num(pr.thigh, fem ? 23.5 : 23) * S, shin: num(pr.shin, fem ? 22 : 21.5) * S, torso: num(pr.torso, fem ? 27 : 29) * S,
    neck: num(pr.neck, 3.5) * S, up: num(pr.upperArm, fem ? 15.5 : 16.5) * S, fore: num(pr.foreArm, fem ? 14.5 : 15) * S,
    hand: 3.9 * S * Math.pow(b, 0.4), foot: 1 * S,
  };
  const r = num(pr.head, fem ? 8.9 : 9.3) * S;
  const W = b * S;
  const rot = num(pose.rot, 0), lean = num(pose.lean, 0);
  const torsoA = 180 - lean - rot;
  const headA = torsoA - num(pose.head, 0);
  const up = dir(torsoA), fwd = dir(torsoA - 90);
  const pelvA = 180 - rot;
  const pf = dir(pelvA - 90);
  const hip: P = { x: 0, y: 0 };
  const T = (v: number, u: number) => add(add(hip, fwd, v), up, u);
  const hipF = add(add(hip, pf, 2.2 * W), dir(pelvA), -1), hipB = add(add(hip, pf, -2.6 * W), dir(pelvA), 0.5);
  const tl = torsoLocal(W, fem, L.torso, style.torsoShape ?? '');
  const torsoPts = tl.map(([v, u]) => T(v, u));
  const neck = T(0.6 * S, L.torso * 0.98);
  const head = add(add(neck, dir(headA), L.neck + r * 0.82), dir(headA - 90), 1.4 * S);
  const shF = T(2.6 * W, L.torso * 0.86), shB = T(-3.6 * W, L.torso * 0.88);
  const arm = (sh: P, sa: number, ea: number, wa: number) => {
    const upA = -lean - rot + sa;
    const el = add(sh, dir(upA), L.up);
    const foreA = upA + ea;
    const wr = add(el, dir(foreA), L.fore);
    const hand = add(wr, dir(foreA + wa), L.hand * 0.85);
    return { upA, el, foreA, wr, hand };
  };
  const aF = arm(shF, num(pose.shoulderF, 20), num(pose.elbowF, 70), num(pose.wristF, 0));
  let aB = arm(shB, num(pose.shoulderB, 25), num(pose.elbowB, 80), num(pose.wristB, 0));
  const wpn = style.weapon;
  if (wpn && wpn.twoHand && !pose.oneHand && pose.weapon !== false && !pose.noWeapon) {
    // two-handed grip: back hand reaches the handle just behind the front hand (2-bone IK)
    const ang = typeof pose.blade === 'number' ? pose.blade - rot : aF.foreA + num(pose.weapon, num(wpn.angle, 75));
    const tgt = add(aF.hand, dir(ang), -6.5 * S);
    const a = L.up, bl = L.fore + L.hand * 0.85;
    const Dv = sub(tgt, shB);
    const dist = Math.max(Math.abs(a - bl) + 0.1, Math.min(a + bl - 0.05, len(Dv)));
    const base = angOf(Dv);
    const A = Math.acos(Math.max(-1, Math.min(1, (a * a + dist * dist - bl * bl) / (2 * a * dist)))) / D2R;
    let best: any = null;
    for (const sgn of [1, -1]) {
      const upA = base + sgn * A;
      const el = add(shB, dir(upA), a);
      const foreA = angOf(sub(tgt, el));
      const cand = { upA, el, foreA, wr: add(el, dir(foreA), L.fore), hand: add(el, dir(foreA), bl) };
      if (!best || cand.el.y > best.el.y) best = cand;
    }
    aB = best;
  }
  const leg = (hp: P, ta: number, ka: number, an: any) => {
    const thA = ta - rot;
    const kn = add(hp, dir(thA), L.thigh);
    const shA = thA - ka;
    const ank = add(kn, dir(shA), L.shin);
    const ftA = typeof an === 'number' ? shA + 90 - an : 90 - rot;
    return { thA, kn, shA, ank, ftA };
  };
  const lF = leg(hipF, num(pose.thighF, 8), num(pose.kneeF, 8), pose.ankleF);
  const lB = leg(hipB, num(pose.thighB, -8), num(pose.kneeB, 6), pose.ankleB);
  const footPts = (an: P, fa: number) => {
    const f = dir(fa), dn = perp(f);
    const k = S;
    const loc: [number, number][] = [[-2.6, -1.6], [-3.2, 2.6], [-2.4, 4.4], [8.8, 4.4], [10.4, 3.6], [9.2, 2.2], [4, 0.2], [1.6, -2.2]];
    return loc.map(([s, t]) => add(add(an, f, s * k), dn, t * k));
  };
  const footF = footPts(lF.ank, lF.ftA), footB = footPts(lB.ank, lB.ftA);
  const sk: Skel = {
    b, r, fem, S, L, rot, lean, torsoA, headA, headTh: (180 - headA) * D2R,
    hip, hipF, hipB, neck, head, shF, shB,
    elF: aF.el, elB: aB.el, wrF: aF.wr, wrB: aB.wr, handF: aF.hand, handB: aB.hand, upAF: aF.upA, upAB: aB.upA, foreAF: aF.foreA, foreAB: aB.foreA,
    knF: lF.kn, knB: lB.kn, anF: lF.ank, anB: lB.ank, thAF: lF.thA, thAB: lB.thA, shAF: lF.shA, shAB: lB.shA, ftAF: lF.ftA, ftAB: lB.ftA,
    up, fwd, torsoPts, footF, footB, W, pose,
  };
  // ---- grounding: lowest body point sits on y = 0 (+ pose.y offset), x offset by pose.x
  let maxY = -Infinity;
  const consider = (p: P, rr = 0) => { maxY = Math.max(maxY, p.y + rr); };
  for (const p of [...footF, ...footB, ...torsoPts]) consider(p);
  consider(head, r); consider(lF.kn, 4 * W); consider(lB.kn, 4 * W); consider(aF.el, 3 * W); consider(aB.el, 3 * W);
  consider(aF.hand, L.hand); consider(aB.hand, L.hand);
  const dy = pose.free ? -num(pose.hipH, 34) * S + num(pose.y, 0) : -maxY + num(pose.y, 0);
  const dx = num(pose.x, 0);
  shiftSkel(sk, dx, dy);
  return sk;
}
function shiftSkel(sk: Skel, dx: number, dy: number) {
  const o = { x: dx, y: dy };
  for (const k of ['hip', 'hipF', 'hipB', 'neck', 'head', 'shF', 'shB', 'elF', 'elB', 'wrF', 'wrB', 'handF', 'handB', 'knF', 'knB', 'anF', 'anB'] as const) (sk as any)[k] = add((sk as any)[k], o);
  sk.torsoPts = sk.torsoPts.map((p) => add(p, o));
  sk.footF = sk.footF.map((p) => add(p, o));
  sk.footB = sk.footB.map((p) => add(p, o));
}

// ------------------------------------------------------------------ weapon geometry
interface WeaponGeo { blade: P[]; handle: P[]; guard: P[]; tip: P; base: P; edge?: P[]; extra?: P[]; type: string; dir: P }
function weaponGeo(style: Record<string, any>, sk: Skel): WeaponGeo | null {
  const w = style.weapon;
  if (!w || !w.type) return null;
  const pose = sk.pose;
  if (pose.weapon === false || pose.noWeapon) return null;
  const inBack = (w.hand ?? 'F') === 'B';
  const hand = inBack ? sk.handB : sk.handF;
  const fa = inBack ? sk.foreAB : sk.foreAF;
  const S = sk.S;
  const ang = typeof pose.blade === 'number' ? pose.blade - sk.rot : fa + num(pose.weapon, num(w.angle, 75));
  const d = dir(ang), n = perp(d);
  const at = (s: number, t: number) => add(add(hand, d, s * S), n, t * S);
  const type = w.type;
  const Lb = num(w.length, type === 'greatsword' ? 70 : type === 'katana' ? 46 : type === 'cutlass' ? 38 : type === 'kunai' ? 11 : type === 'staff' ? 60 : 40);
  if (type === 'greatsword') {
    const wd = num(w.width, 11);
    const blade = [at(4, -wd * 0.42), at(Lb * 0.55, -wd * 0.5), at(Lb - 2, -wd * 0.48), at(Lb + 6, -wd * 0.1), at(Lb, wd * 0.52), at(Lb * 0.35, wd * 0.5), at(10, wd * 0.5), at(8, wd * 0.25), at(5.5, wd * 0.35), at(4, wd * 0.42)];
    const edge = [at(Lb * 0.35, wd * 0.5), at(Lb, wd * 0.52), at(Lb + 6, -wd * 0.1), at(Lb + 2.5, -wd * 0.05), at(Lb - 2, wd * 0.3), at(Lb * 0.35, wd * 0.3)];
    return { type, blade, edge, handle: [at(-11, -1.6), at(2, -1.8), at(2, 1.8), at(-11, 1.6)], guard: [at(1.5, -wd * 0.62), at(4.5, -wd * 0.62), at(4.5, wd * 0.62), at(1.5, wd * 0.62)], tip: at(Lb + 6, 0), base: at(6, 0), dir: d, extra: [at(-12, 0)] };
  }
  if (type === 'katana' || type === 'cutlass') {
    const cut = type === 'cutlass';
    const wd = num(w.width, cut ? 4.2 : 2.8);
    const curve = cut ? 3.2 : 1.6;
    const pts: P[] = [];
    const N = 8;
    for (let i = 0; i <= N; i++) { const t = i / N; pts.push(at(3 + t * (Lb - 3), -wd * 0.5 * (cut ? 1 + t * 0.5 : 1) - Math.sin(t * Math.PI * 0.9) * curve * t)); }
    pts.push(at(Lb + 4, -curve * 1.1 - wd * 0.2));
    for (let i = N; i >= 0; i--) { const t = i / N; pts.push(at(3 + t * (Lb - 3), wd * 0.5 * (cut ? 1 + t * 0.2 : 1) - Math.sin(t * Math.PI * 0.9) * curve * t)); }
    const gw = cut ? 4.6 : 3.4;
    return { type, blade: pts, handle: [at(cut ? -7 : -12, -1.4), at(2, -1.6), at(2, 1.6), at(cut ? -7 : -12, 1.4)], guard: cut ? [at(1, -gw), at(4, -gw), at(4, gw + 3), at(-5, gw + 4), at(-7, 2), at(1, gw * 0.5)] : [at(1.5, -gw), at(3.5, -gw), at(3.5, gw), at(1.5, gw)], tip: at(Lb + 4, -curve), base: at(4, 0), dir: d };
  }
  if (type === 'kunai') {
    return { type, blade: [at(3, -1.6), at(5, -2.6), at(Lb, -0.2), at(Lb + 2, 0), at(Lb, 0.2), at(5, 2.6), at(3, 1.6)], handle: [at(-5, -1), at(3, -1.1), at(3, 1.1), at(-5, 1)], guard: [at(-8.5, -2), at(-5, -2), at(-5, 2), at(-8.5, 2)], tip: at(Lb + 2, 0), base: at(4, 0), dir: d };
  }
  if (type === 'staff') {
    return { type, blade: [at(-Lb * 0.45, -1.6), at(Lb * 0.55, -1.6), at(Lb * 0.55, 1.6), at(-Lb * 0.45, 1.6)], handle: [], guard: [], tip: at(Lb * 0.55, 0), base: at(0, 0), dir: d };
  }
  return null;
}

// ------------------------------------------------------------------ main pose renderer
function colorsOf(style: Record<string, any>, palette?: Record<string, any>): Record<string, string> {
  const c: Record<string, string> = { ...DEFAULT_COLORS, ...(style.colors ?? {}) };
  if (palette) Object.assign(c, palette.colors ?? palette);
  return c;
}

const PART_ALIAS: Record<string, string> = {
  handR: 'handF', handL: 'handB', footR: 'footF', footL: 'footB', fistF: 'handF', fistB: 'handB', kneeR: 'kneeF', kneeL: 'kneeB',
  elbowR: 'elbowF', elbowL: 'elbowB', hand: 'handF', foot: 'footF', fist: 'handF', blade: 'weapon', sword: 'weapon', tip: 'weapon',
};

function partPoint(sk: Skel, part: string): P | null {
  const p = PART_ALIAS[part] ?? part;
  switch (p) {
    case 'handF': return sk.handF;
    case 'handB': return sk.handB;
    case 'footF': return lerpP(sk.footF[3], sk.footF[4], 0.5);
    case 'footB': return lerpP(sk.footB[3], sk.footB[4], 0.5);
    case 'heelF': return sk.footF[1];
    case 'heelB': return sk.footB[1];
    case 'kneeF': return sk.knF;
    case 'kneeB': return sk.knB;
    case 'elbowF': return sk.elF;
    case 'elbowB': return sk.elB;
    case 'head': return sk.head;
    case 'shoulder': case 'shoulderF': return sk.shF;
    case 'body': case 'torso': return lerpP(sk.hip, sk.neck, 0.55);
    case 'hip': return sk.hip;
    default: return null;
  }
}

function computeBoxes(sk: Skel, active: ActiveSpec | undefined, wg: WeaponGeo | null): { hurt: Box[]; hit: Box[] } {
  const r = sk.r;
  const box = (pts: P[], pad: number, pady = pad): Box => {
    const bb = new BBox();
    for (const p of pts) bb.addP(p);
    return [Math.round(bb.x1 - pad), Math.round(bb.y1 - pady), Math.round(bb.x2 + pad), Math.round(bb.y2 + pady)];
  };
  const hurt: Box[] = [];
  hurt.push(box([sk.head], r + 1.5));
  hurt.push(box([sk.neck, sk.shF, sk.shB, sk.hip, sk.hipF, sk.hipB, lerpP(sk.hip, sk.neck, 0.5)], 6 * sk.W, 2));
  hurt.push(box([sk.hipF, sk.hipB, sk.knF, sk.knB, sk.anF, sk.anB, sk.footF[3], sk.footB[3], sk.footF[2], sk.footB[2]], 3.5 * sk.W, 1));
  const hit: Box[] = [];
  if (active) {
    if (active.box) hit.push(active.box.slice() as Box);
    else {
      const part = PART_ALIAS[active.part] ?? active.part;
      const rad = num(active.radius, 9);
      if (part === 'weapon' && wg) {
        const pts = [...wg.blade, wg.tip];
        hit.push(box(pts, Math.max(1, rad * 0.35)));
        hurt.push(box([wg.base, sk.handF], 4));
      } else if (part === 'aura' || part === 'body') {
        const bb = box([sk.neck, sk.hip, sk.shF, sk.handF, sk.footF[3], sk.head], rad * 0.5);
        hit.push(bb);
      } else {
        const p = partPoint(sk, part);
        if (p) {
          hit.push([Math.round(p.x - rad), Math.round(p.y - rad), Math.round(p.x + rad), Math.round(p.y + rad)]);
          // attacking limb stays vulnerable (slightly smaller than the hit box)
          const isArm = part.startsWith('hand') || part.startsWith('elbow');
          const isLeg = part.startsWith('foot') || part.startsWith('knee') || part.startsWith('heel');
          if (isArm) { const back = part.endsWith('B'); hurt.push(box([back ? sk.elB : sk.elF, back ? sk.handB : sk.handF], 3.5)); }
          else if (isLeg) { const back = part.endsWith('B'); hurt.push(box([back ? sk.knB : sk.knF, back ? sk.anB : sk.anF], 3.5)); }
        }
      }
    }
  }
  return { hurt: hurt.map(normBox), hit: hit.map(normBox) };
}
function normBox(b: Box): Box { return [Math.min(b[0], b[2]), Math.min(b[1], b[3]), Math.max(b[0], b[2]), Math.max(b[1], b[3])]; }

/** Builds every shape for the pose (pure geometry, no canvas) */
function buildLayers(style: Record<string, any>, sk: Skel, C: Record<string, string>, wg: WeaponGeo | null, bb: BBox): Layer[] {
  const pose = sk.pose;
  const S = sk.S, W = sk.W, r = sk.r;
  const layers: Layer[] = [];
  const wind = num(pose.wind, 0.35);
  const phase = num(pose.phase, 0);
  const trail = Math.abs(sk.rot) > 50 && Math.abs(sk.rot) < 130 ? -92 : -(25 + wind * 60) - sk.rot * 0.3; // trailing angle for cloth / hair (dir() convention, negative = backwards)
  const reg = (pts: P[], rr = 0) => { bb.addPts(pts, rr); return pts; };
  const H = (x: number, y: number): P => {
    const c = Math.cos(sk.headTh), s = Math.sin(sk.headTh);
    return { x: sk.head.x + x * c - y * s, y: sk.head.y + x * s + y * c };
  };
  const Hp = (pts: [number, number][]) => pts.map(([x, y]) => H(x * r, y * r));

  const top = style.top ?? 'gi';
  const sleeves = style.sleeves ?? (top === 'tank' || top === 'bare' ? 'none' : top === 'robe' ? 'wide' : 'long');
  const pantsT = style.pants ?? 'gi';
  const skirt = style.skirt as Record<string, any> | undefined;

  // ---------------- back accessories
  const backL: Shape[] = [];
  if (style.cape) {
    const cp = style.cape;
    const clen = num(cp.length, 55) * S;
    const a = add(sk.shB, sk.up, 2), b2 = add(sk.shF, sk.up, 1);
    const spineA = waveSpine(a, trail * 0.6, clen, 10, phase, 6);
    const spineB = waveSpine(b2, trail * 0.45, clen * 0.95, 9, phase + 1.3, 6);
    const pts = reg([...spineA, ...spineB.reverse()]);
    backL.push({ path: polyPath(pts, true), fill: backOf(C[cp.color ?? 'cloth'] ?? cp.color), shade: 4 });
  }
  const hair = style.hair ?? { type: 'short' };
  const hcol = C.hair;
  if (hair.type === 'long' || hair.type === 'flowing') {
    const hl = num(hair.length, 46) * S;
    const root1 = H(-0.85 * r, -0.4 * r), root2 = H(0.1 * r, -0.95 * r);
    const sp1 = waveSpine(root1, 8 + trail * 0.5, hl, 8, phase, 7);
    const sp2 = waveSpine(root2, -10 + trail * 0.55, hl * 1.05, 9, phase + 0.8, 7);
    const tips: P[] = [];
    // jagged tip set
    const e1 = sp1[sp1.length - 1], e2 = sp2[sp2.length - 1];
    for (let i = 0; i < 4; i++) { const t = (i + 0.5) / 4; const base = lerpP(e1, e2, t); tips.push(add(base, dir(trail * 0.5), 6 + hash(i + 3) * 6)); tips.push(lerpP(e1, e2, (i + 1) / 4)); }
    const pts = reg([...sp1, ...tips, ...sp2.reverse()]);
    backL.push({ path: polyPath(pts, true), fill: mix(hcol, shadeOf(hcol), 0.35), shade: 4, rim: 0.8 });
  }
  if (hair.type === 'ponytail') {
    const hl = num(hair.length, 40) * S;
    const root = H(-0.75 * r, -0.65 * r);
    const sp = waveSpine(root, trail * 0.9 - 5, hl, 16, phase, 8, -10);
    const pts = reg(ribbonPts(sp, 8 * S, 1.5));
    backL.push({ path: polyPath(pts, true), fill: hcol, shade: 2.2 });
  }
  if (style.headband) {
    const hb = style.headband;
    const knot = H(-0.95 * r, -0.48 * r);
    const l1 = num(hb.tail, 26) * S;
    for (let i = 0; i < 2; i++) {
      const sp = waveSpine(knot, trail * (0.95 + i * 0.12) + i * 8, l1 * (1 - i * 0.18), 18, phase + i * 1.7, 7);
      backL.push({ path: polyPath(reg(ribbonPts(sp, 3.6 * S, 2.4 * S)), true), fill: i ? backOf(C[hb.color ?? 'top2'] ?? hb.color) : C[hb.color ?? 'top2'] ?? hb.color, shade: 1 });
    }
  }
  if (style.scarf) {
    const sc = style.scarf;
    const st = add(sk.neck, sk.fwd, -3 * W);
    const l1 = num(sc.length, 40) * S;
    for (let i = 0; i < 2; i++) {
      const sp = waveSpine(st, trail * (1 + i * 0.1) + 6 * i, l1 * (1 - i * 0.25), 16, phase + i * 2, 8);
      backL.push({ path: polyPath(reg(ribbonPts(sp, 7 * S, 4 * S)), true), fill: i ? backOf(C[sc.color ?? 'cloth'] ?? sc.color) : C[sc.color ?? 'cloth'] ?? sc.color, shade: 2 });
    }
  }
  if (style.tail) {
    const tl = style.tail;
    const sp = waveSpine(add(sk.hip, sk.fwd, -6 * W), -50 + trail * 0.4, num(tl.length, 30) * S, 25, phase, 8, 40);
    backL.push({ path: polyPath(reg(ribbonPts(sp, 5 * S, 1)), true), fill: C[tl.color ?? 'hair'] ?? tl.color, shade: 1.5 });
  }
  // coat tails (behind legs)
  const flap = (hipP: P, thA: number, shA: number, length: number, w0: number, w1: number, ph: number, gravity = 0.4): P[] => {
    const d1 = norm(lerpP(dir(thA), dir(0), gravity));
    const midL = Math.min(length, sk.L.thigh * 1.0);
    const kneeP = add(hipP, d1, midL);
    let d2 = norm(lerpP(dir(shA), dir(trail * 0.4), 0.55));
    if (d2.y < 0.45) d2 = norm(lerpP(d2, { x: d2.x * 0.3, y: 1 }, 0.7)); // cloth never flows upwards
    const rest = Math.max(0, length - midL);
    const end = add(kneeP, d2, rest);
    const n1 = norm(perp(d1)), n2 = norm(perp(d2));
    const wv = Math.sin(phase * 1.3 + ph) * 1.6;
    return [add(hipP, n1, w0), add(kneeP, n1, (w0 + w1) / 2), add(end, n2, w1), add(lerpP(end, kneeP, 0.02), d2, 2 + wv), add(end, n2, -w1 * 0.1), add(end, n2, -w1), add(kneeP, n1, -(w0 + w1) / 2), add(hipP, n1, -w0)];
  };
  if (skirt && skirt.type === 'coat') {
    const sl = num(skirt.length, 40) * S;
    const col = C[skirt.color ?? 'top'] ?? skirt.color;
    const wst = add(sk.hip, sk.up, 3);
    backL.push({ path: polyPath(reg(flap(add(wst, sk.fwd, -3 * W), sk.thAB - 12, sk.shAB - 20, sl, 6 * W, 9 * W, 0, 0.5)), true), fill: backOf(col), shade: 3, line: true });
    backL.push({ path: polyPath(reg(flap(add(wst, sk.fwd, -1 * W), sk.thAF * 0.5 - 14, sk.shAF * 0.5 - 22, sl * 0.97, 6 * W, 9 * W, 1.4, 0.5)), true), fill: backOf(col), shade: 3, line: true });
  }
  if (backL.length) layers.push({ shapes: backL });

  // ---------------- limbs
  const skinF = C.skin;
  const topC = C.top, pantsC = C.pants;
  const armShapes = (side: 'F' | 'B'): Shape[] => {
    const sh = side === 'F' ? sk.shF : sk.shB, el = side === 'F' ? sk.elF : sk.elB, wr = side === 'F' ? sk.wrF : sk.wrB, hd = side === 'F' ? sk.handF : sk.handB;
    const fa = side === 'F' ? sk.foreAF : sk.foreAB;
    const dk = side === 'B' ? backOf : (c: string) => c;
    const out: Shape[] = [];
    const upR0 = 3.8 * W * (sk.fem ? 0.82 : 1), upR1 = 3.0 * W * (sk.fem ? 0.82 : 1), foR0 = 3.2 * W * (sk.fem ? 0.82 : 1), foR1 = 2.5 * W * (sk.fem ? 0.85 : 1);
    reg([sh, el, wr, hd], 6 * W);
    const armCol = sleeves === 'long' ? topC : skinF;
    // upper arm + deltoid
    out.push({ path: capsulePath(sh, el, upR0, upR1), fill: dk(sleeves === 'none' || sleeves === 'wide' ? skinF : sleeves === 'short' ? skinF : topC), shade: 2.2 });
    if (sleeves === 'none' && !sk.fem && W > 0.9) out.push({ path: ellipsePath(lerpP(sh, el, 0.28), upR0 * 1.18, upR0 * 0.95, Math.atan2(el.y - sh.y, el.x - sh.x)), fill: dk(skinF), shade: 2.4, noOutline: false });
    out.push({ path: capsulePath(el, wr, foR0, foR1), fill: dk(sleeves === 'long' || sleeves === 'gi' && false ? topC : armCol === topC ? topC : skinF), shade: 2 });
    if (sleeves === 'short' || sleeves === 'gi') {
      const end = sleeves === 'gi' ? lerpP(el, wr, 0.35) : lerpP(sh, el, 0.7);
      const pts = sleeves === 'gi' ? [sh, el, end] : [sh, end];
      const rr = upR0 * (sleeves === 'gi' ? 1.3 : 1.25);
      if (sleeves === 'gi') {
        out.push({ path: capsulePath(sh, el, rr, rr * 0.95), fill: dk(topC), shade: 2.6, line: true });
        const n = norm(perp(sub(wr, el)));
        out.push({ path: polyPath(reg([add(el, n, rr * 0.95), add(end, n, rr * 1.05), add(end, n, -rr * 1.05), add(el, n, -rr * 0.95)]), false), fill: dk(topC), shade: 2.2, line: true });
      } else {
        out.push({ path: capsulePath(pts[0], pts[1], rr, rr * 0.9), fill: dk(topC), shade: 2.4, line: true });
      }
    }
    if (sleeves === 'wide') {
      const g = { x: 0, y: 1 };
      const d = norm(sub(wr, el));
      const n = norm(perp(sub(el, sh)));
      const sEnd = add(el, d, sk.L.fore * 0.55);
      const n2 = norm(perp(d));
      const hang = add(g, dir(trail), 0.3);
      const pts = reg([add(sh, n, 4.5 * W), add(el, n, 5 * W), add(sEnd, n2, 6.5 * W), add(add(sEnd, n2, -6.5 * W), hang, 7 * S), add(add(el, n, -5.5 * W), hang, 4 * S), add(sh, n, -4.5 * W)]);
      out.push({ path: polyPath(pts, true), fill: dk(topC), shade: 3, line: true });
    }
    // gloves / wraps
    const glove = style.gloves ?? 'none';
    if (glove === 'gauntlet' || glove === 'glove' || glove === 'long') {
      const st = glove === 'long' ? lerpP(el, wr, 0.15) : lerpP(el, wr, glove === 'gauntlet' ? 0.4 : 0.7);
      out.push({ path: capsulePath(st, wr, foR0 * 1.18, foR1 * 1.22), fill: dk(C.glove), shade: 2, line: true });
    } else if (glove === 'wrap') {
      out.push({ path: capsulePath(lerpP(el, wr, 0.7), wr, foR1 * 1.15, foR1 * 1.12), fill: dk(C.wrap ?? '#ece6d8'), shade: 1.5, line: true });
    }
    // hand
    const hc = glove === 'none' || glove === 'wrap' ? skinF : C.glove;
    const open = (side === 'F' ? pose.handF : pose.handB) === 'open';
    const hr = sk.L.hand;
    const d = dir(fa + num(side === 'F' ? pose.wristF : pose.wristB, 0));
    if (open) {
      const n = perp(d);
      const base = add(wr, d, 0.5);
      const pts = reg([add(base, n, -hr * 0.85), add(add(base, d, hr * 1.4), n, -hr * 0.8), add(add(base, d, hr * 2.3), n, -hr * 0.4), add(base, d, hr * 2.5), add(add(base, d, hr * 2.2), n, hr * 0.5), add(add(base, d, hr * 1.2), n, hr * 0.95), add(add(base, d, hr * 1.1), n, hr * 1.5), add(add(base, d, hr * 0.4), n, hr * 1.3), add(base, n, hr * 0.9)]);
      out.push({ path: polyPath(pts, true), fill: dk(hc), shade: 1.6 });
    } else {
      out.push({ path: ellipsePath(hd, hr * 1.12, hr * 0.98, Math.atan2(d.y, d.x)), fill: dk(hc), shade: 1.6 });
    }
    return out;
  };
  const legShapes = (side: 'F' | 'B'): Shape[] => {
    const hp = side === 'F' ? sk.hipF : sk.hipB, kn = side === 'F' ? sk.knF : sk.knB, an = side === 'F' ? sk.anF : sk.anB;
    const foot = side === 'F' ? sk.footF : sk.footB;
    const dk = side === 'B' ? backOf : (c: string) => c;
    const out: Shape[] = [];
    reg([hp, kn, an], 7 * W); reg(foot, 1);
    const baggy = pantsT === 'gi' ? 1.28 : pantsT === 'hakama' ? 1.5 : pantsT === 'baggy' ? 1.35 : 1;
    const legSkin = pantsT === 'trunks' || pantsT === 'skirt' || pantsT === 'shorts';
    const thR0 = 6 * W * (sk.fem ? 0.92 : 1), thR1 = 4.3 * W, shR0 = 4.4 * W, shR1 = 2.9 * W;
    const legCol = legSkin ? (C.legs ?? skinF) : pantsC;
    out.push({ path: capsulePath(hp, kn, thR0 * (legSkin ? 1 : baggy), thR1 * (legSkin ? 1 : baggy)), fill: dk(legCol), shade: 2.8 });
    // calf bulge
    const calf = add(lerpP(kn, an, 0.32), norm(perp(sub(an, kn))), 0.9 * W);
    out.push({ path: capsulePath(kn, an, shR0 * (legSkin ? 1 : baggy * 0.98), shR1 * (legSkin ? 1 : baggy * (pantsT === 'gi' ? 1.15 : 1))), fill: dk(legCol), shade: 2.4 });
    if (legSkin) out.push({ path: ellipsePath(calf, shR0 * 1.02, shR0 * 1.25, Math.atan2(an.y - kn.y, an.x - kn.x) + Math.PI / 2), fill: dk(legCol), shade: 2.2 });
    if (pantsT === 'trunks' || pantsT === 'shorts') {
      out.push({ path: capsulePath(hp, lerpP(hp, kn, pantsT === 'shorts' ? 0.5 : 0.2), thR0 * 1.04, thR0 * 1.0), fill: dk(pantsC), shade: 2.6, line: true });
    }
    // boots / feet
    const boot = style.boots ?? 'boot';
    const fcol = boot === 'barefoot' || boot === 'wrap' ? skinF : C.boot;
    if (boot === 'boot' || boot === 'greave' || boot === 'tall') {
      const st = lerpP(kn, an, boot === 'tall' ? 0.05 : boot === 'greave' ? 0.25 : 0.45);
      out.push({ path: capsulePath(st, an, shR0 * 1.12 * (boot === 'tall' ? 1.05 : 1), shR1 * 1.35), fill: dk(C.boot), shade: 2.2, line: true });
      if (boot === 'greave') out.push({ path: polyPath(reg([add(st, norm(perp(sub(an, kn))), shR0 * 1.25), lerpP(st, kn, 0.35), add(st, norm(perp(sub(an, kn))), -shR0 * 1.25)]), false), fill: dk(C.metal), shade: 1, line: true });
    } else if (boot === 'wrap' || boot === 'tabi') {
      out.push({ path: capsulePath(lerpP(kn, an, 0.72), an, shR1 * 1.3, shR1 * 1.25), fill: dk(C.wrap ?? '#ece6d8'), shade: 1.5, line: true });
    }
    if (pantsT === 'gi' || pantsT === 'baggy') {
      // pants cuff flare over the ankle
      const d = norm(sub(an, kn)), n = perp(d);
      const c0 = lerpP(kn, an, 0.72), c1 = add(an, d, -0.5);
      const fl = shR1 * baggy * 1.35;
      out.push({ path: polyPath(reg([add(c0, n, shR0 * baggy * 0.95), add(c1, n, fl), add(add(c1, n, fl * 0.2), d, 1.2), add(c1, n, -fl), add(c0, n, -shR0 * baggy * 0.95)]), true), fill: dk(pantsC), shade: 2.2, line: true });
    }
    out.push({ path: polyPath(foot, true), fill: dk(fcol), shade: 1.6, line: boot !== 'barefoot' });
    return out;
  };

  // back arm & back leg
  const weaponLayer = (): Layer | null => {
    if (!wg) return null;
    reg([...wg.blade, ...wg.handle, ...wg.guard, wg.tip], 2);
    const glow = style.weapon.glow || pose.glow ? (C[style.weapon.glowColor ?? 'energy'] ?? style.weapon.glowColor) : null;
    const shapes: Shape[] = [];
    if (wg.handle.length) shapes.push({ path: polyPath(wg.handle), fill: C.hilt, shade: 0.8 });
    const bladeCol = C.weapon;
    shapes.push({ path: polyPath(wg.blade, wg.type === 'katana' || wg.type === 'cutlass' ? false : false), fill: bladeCol, shade: wg.type === 'greatsword' ? 3.5 : 1.2, rim: 0.6 });
    if (wg.edge) shapes.push({ path: polyPath(wg.edge), fill: C.weapon2 ? lightOf(bladeCol) : '#ffffff', shade: 0, noOutline: true, flat: true });
    if (wg.guard.length) shapes.push({ path: polyPath(wg.guard, wg.type === 'cutlass'), fill: C.accent, shade: 1 });
    return { shapes, aura: glow };
  };
  const wl = weaponLayer();
  const weaponBack = !!(style.weapon && (style.weapon.hand ?? 'F') === 'B');
  if (wl && weaponBack) layers.push(wl);
  layers.push({ shapes: armShapes('B') });
  layers.push({ shapes: legShapes('B') });

  // robe flaps: back flap between back leg and body
  const robeFlap = (side: 'F' | 'B') => {
    if (!skirt || skirt.type === 'coat') return null;
    const sl = num(skirt.length, 30) * S;
    const col = C[skirt.color ?? 'top'] ?? skirt.color;
    const hp = side === 'F' ? sk.hipF : sk.hipB;
    const wst = add(hp, sk.up, 4);
    const thA = side === 'F' ? sk.thAF : sk.thAB, shA = side === 'F' ? sk.shAF : sk.shAB;
    const fw = num(skirt.width, 1);
    // raised legs (kicks) make the cloth drape more instead of following the leg
    const rel = thA + sk.rot;
    const grav = 0.25 + 0.6 * Math.max(0, Math.min(1, (Math.abs(rel) - 25) / 80));
    const pts = reg(flap(wst, thA, shA, sl, 6 * W * fw, 8.6 * W * fw, side === 'F' ? 0 : 2, grav));
    return { path: polyPath(pts, true), fill: side === 'B' ? backOf(col) : col, shade: 3, line: true } as Shape;
  };
  const fb = robeFlap('B');
  if (fb) layers.push({ shapes: [fb] });

  // ---------------- body
  const body: Shape[] = [];
  reg(sk.torsoPts, 1);
  const T = (v: number, u: number) => add(add(sk.hip, sk.fwd, v), sk.up, u);
  const torsoCol = top === 'bare' ? skinF : topC;
  // pelvis / pants seat
  body.push({ path: ellipsePath(add(sk.hip, sk.up, 2), (pantsT === 'trunks' ? 6.3 : 7.6) * W * (pantsT === 'gi' ? 1.1 : 1), (pantsT === 'trunks' || pantsT === 'shorts' ? 4.2 : 6.2) * W, Math.atan2(sk.fwd.y, sk.fwd.x)), fill: pantsT === 'trunks' || pantsT === 'shorts' ? pantsC : legSkinOr(pantsT, pantsC, C), shade: 2.5 });
  body.push({ path: polyPath(sk.torsoPts, true), fill: torsoCol, shade: 4.2, rim: 0.9 });
  const neckR = 2.9 * Math.pow(W, 0.8);
  body.push({ path: capsulePath(sk.neck, H(-0.05 * r, 0.45 * r), neckR * 1.05, neckR), fill: skinF, shade: 1.8 });
  const bodyAfter = (ctx: CanvasRenderingContext2D) => {
    // torso details
    const TL = sk.L.torso / 29;
    const line = (pts: P[], col: string, wdt: number) => { ctx.beginPath(); ctx.moveTo(pts[0].x, pts[0].y); for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i].x, pts[i].y); ctx.strokeStyle = col; ctx.lineWidth = wdt; ctx.stroke(); };
    const fillPts = (pts: P[], col: string, smooth = false) => { ctx.beginPath(); polyPath(pts, smooth)(ctx, 0, 0); ctx.fillStyle = col; ctx.fill(); };
    const clipTorso = () => { ctx.save(); ctx.beginPath(); polyPath(sk.torsoPts, true)(ctx, 0, 0); ctx.clip(); };
    const OL = C.outline;
    clipTorso();
    if (top === 'gi' || top === 'robe') {
      const vBot = T(3.5 * W, 12 * TL);
      const skinV = [T(-1.2, 32 * TL), T(6.5 * W, 31 * TL), vBot];
      fillPts(skinV, skinF);
      if (style.undershirt) fillPts(skinV, C[style.undershirt] ?? style.undershirt);
      // pec line
      if (!style.undershirt && !sk.fem) line([T(2.5 * W, 21 * TL), T(5 * W, 19.5 * TL), T(7.5 * W, 21 * TL)], rgba(shadeOf(skinF), 0.9), 0.5);
      ctx.lineCap = 'round';
      line([T(-2.5, 32.5 * TL), T(-0.5 * W, 20 * TL), vBot, T(5.5 * W, 2 * TL)], C.trim, 2.4 * S);
      line([T(8 * W, 30 * TL), vBot], C.trim, 2.4 * S);
      line([T(-2.5, 32.5 * TL), T(-0.5 * W, 20 * TL), vBot, T(5.5 * W, 2 * TL)], OL, 0.45);
      line([T(8 * W, 30 * TL), vBot], OL, 0.45);
      // fold lines
      line([T(-6 * W, 18 * TL), T(-3 * W, 10 * TL)], rgba(shadeOf(topC), 0.8), 0.5);
    } else if (top === 'bare' || top === 'tank' || top === 'vest') {
      const sh = rgba(shadeOf(skinF), 0.95);
      if (top !== 'bare') {
        // tank top body
        fillPts([T(-9 * W, 30 * TL), T(-2, 31 * TL), T(1.5 * W, 26 * TL), T(5.5 * W, 27 * TL), T(9.5 * W, 24 * TL), T(10 * W, -3), T(-9 * W, -3)], topC);
        if (top === 'vest') fillPts([T(1 * W, 30 * TL), T(9.6 * W, 26 * TL), T(9.6 * W, 3), T(4.5 * W, 3)], skinF);
      } else {
        line([T(1.5 * W, 23 * TL), T(5 * W, 18.5 * TL), T(8.6 * W, 20 * TL)], sh, 0.7);
        line([T(4.5 * W, 15 * TL), T(4.5 * W, 3 * TL)], sh, 0.5);
        for (const u of [13, 9, 5]) line([T(2.2 * W, u * TL), T(6.6 * W, (u + 0.6) * TL)], sh, 0.45);
        line([T(-4 * W, 24 * TL), T(-6 * W, 14 * TL)], sh, 0.5);
      }
    } else if (top === 'jacket' || top === 'coat') {
      fillPts([T(-0.5, 32 * TL), T(7.5 * W, 30 * TL), T(9.4 * W, 22 * TL), T(7.5 * W, 6 * TL), T(4.2 * W, 6 * TL), T(1.5 * W, 22 * TL)], C.top2);
      line([T(-0.5, 32 * TL), T(1.5 * W, 22 * TL), T(4.2 * W, 6 * TL), T(4.4 * W, -3)], C.trim, 1.8 * S);
      line([T(-0.5, 32 * TL), T(1.5 * W, 22 * TL), T(4.2 * W, 6 * TL), T(4.4 * W, -3)], OL, 0.4);
      // collar
      fillPts([T(-5 * W, 33.5 * TL), T(-0.8, 35 * TL), T(2.2 * W, 28 * TL), T(-0.8, 27 * TL)], C.trim, false);
      line([T(-5 * W, 33.5 * TL), T(-0.8, 35 * TL), T(2.2 * W, 28 * TL), T(-0.8, 27 * TL), T(-5 * W, 33.5 * TL)], OL, 0.4);
      for (const u of [18, 12]) { ctx.beginPath(); const p = T(6 * W, u * TL); ctx.arc(p.x, p.y, 0.8, 0, Math.PI * 2); ctx.fillStyle = C.accent; ctx.fill(); }
    } else if (top === 'qipao') {
      line([T(1 * W, 32 * TL), T(3.5 * W, 27 * TL), T(9.5 * W, 23 * TL)], C.trim, 1.5 * S);
      line([T(-1, 32 * TL), T(-2, 34 * TL), T(3, 34.5 * TL), T(4 * W, 31 * TL)], C.trim, 1.6 * S);
      line([T(3.5 * W, 27 * TL), T(9.5 * W, 23 * TL)], OL, 0.35);
      for (const u of [26, 24.5]) { ctx.beginPath(); const p = T(5.5 * W + (26 - u), u * TL); ctx.arc(p.x, p.y, 0.7, 0, Math.PI * 2); ctx.fillStyle = C.accent; ctx.fill(); }
      // embroidery
      ctx.beginPath(); const e = T(-2 * W, 16 * TL); ctx.arc(e.x, e.y, 3 * W, 0.5, 4.2); ctx.strokeStyle = rgba(C.accent, 0.8); ctx.lineWidth = 0.6; ctx.stroke();
    } else if (top === 'armor' || top === 'suit') {
      const glowC = C.visor;
      fillPts([T(-6 * W, 27 * TL), T(-1, 30 * TL), T(8 * W, 27 * TL), T(9.4 * W, 18 * TL), T(6 * W, 12 * TL), T(-6 * W, 14 * TL)], C.metal, true);
      line([T(-6 * W, 27 * TL), T(-1, 30 * TL), T(8 * W, 27 * TL), T(9.4 * W, 18 * TL), T(6 * W, 12 * TL), T(-6 * W, 14 * TL), T(-6 * W, 27 * TL)], OL, 0.5);
      if (top === 'suit') {
        ctx.save(); ctx.shadowColor = glowC; ctx.shadowBlur = 6;
        line([T(2 * W, 24 * TL), T(5.5 * W, 20 * TL), T(5.5 * W, 15 * TL)], glowC, 0.9);
        line([T(4.5 * W, 10 * TL), T(6 * W, 2 * TL)], glowC, 0.8);
        ctx.beginPath(); const c = T(3.2 * W, 21 * TL); ctx.arc(c.x, c.y, 2.2 * W, 0, Math.PI * 2); ctx.fillStyle = glowC; ctx.fill();
        ctx.restore();
        ctx.beginPath(); const c2 = T(3.2 * W, 21 * TL); ctx.arc(c2.x, c2.y, 2.2 * W, 0, Math.PI * 2); ctx.strokeStyle = OL; ctx.lineWidth = 0.5; ctx.stroke();
        ctx.beginPath(); ctx.arc(c2.x, c2.y, 1.1 * W, 0, Math.PI * 2); ctx.fillStyle = '#ffffff'; ctx.fill();
      }
      for (const u of [8, 4]) line([T(-6 * W, u * TL), T(7 * W, u * TL)], rgba(shadeOf(topC), 0.9), 0.5);
    }
    ctx.restore();
  };
  layers.push({ shapes: body, after: bodyAfter });

  // belt / sash
  const beltLayer = (): Layer | null => {
    if (!style.belt) return null;
    const bc = C[style.belt === true ? 'belt' : style.belt] ?? style.belt;
    const tl = sk.torsoPts;
    const TL = sk.L.torso / 29;
    const a0 = T(-8.2 * W, 0.5 * TL), a1 = T(8.4 * W, 0.5 * TL), a2 = T(8.0 * W, 5 * TL), a3 = T(-7.8 * W, 5 * TL);
    void tl;
    const shapes: Shape[] = [{ path: polyPath(reg([a0, a1, a2, a3]), false), fill: bc, shade: 1.6, line: true }];
    const knot = T(5.5 * W, 2.5 * TL);
    const tails = num(style.beltTails, 1);
    if (tails) {
      for (let i = 0; i < 2; i++) {
        const sp = waveSpine(knot, 10 + i * 18 + trail * 0.15, (10 + i * 2) * S, 8, phase + i, 4);
        shapes.push({ path: polyPath(reg(ribbonPts(sp, 2.2 * S, 1.8 * S)), false), fill: i ? backOf(bc) : bc, shade: 0.8, line: true });
      }
      shapes.push({ path: ellipsePath(knot, 2.2 * S, 1.8 * S), fill: bc, shade: 0.8 });
    }
    return { shapes };
  };

  // ---------------- front leg + front flap + belt
  layers.push({ shapes: legShapes('F') });
  const ff = robeFlap('F');
  if (ff) layers.push({ shapes: [ff] });
  const bl = beltLayer();
  if (bl) layers.push(bl);
  if (style.cape && style.cape.collar) {
    const cp = style.cape;
    const pts = reg([add(sk.shB, sk.up, 3), add(sk.neck, sk.fwd, 4), add(sk.shF, sk.fwd, 3), add(add(sk.shF, sk.up, -4), sk.fwd, 2)]);
    layers.push({ shapes: [{ path: polyPath(pts, true), fill: C[cp.color ?? 'cloth'] ?? cp.color, shade: 1.5 }] });
  }
  if (style.scarf) {
    const sc = style.scarf;
    const c = add(sk.neck, sk.up, -1);
    layers.push({ shapes: [{ path: ellipsePath(c, 5.6 * S, 3.6 * S, Math.atan2(sk.fwd.y, sk.fwd.x) - 0.15), fill: C[sc.color ?? 'cloth'] ?? sc.color, shade: 1.6 }] });
  }

  // ---------------- head
  const headLayerShapes: Shape[] = [];
  const helmet = hair.type === 'helmet';
  const headPts = Hp([...Array.from({ length: 13 }, (_, i) => { const a = (140 + i * (230 / 12)) * D2R; return [Math.cos(a) * 0.93, Math.sin(a)] as [number, number]; }), [0.93, 0.48], [0.62, 0.98], [0.2, 1.0], [-0.3, 0.74]]);
  reg(headPts, 1);
  headLayerShapes.push({ path: polyPath(headPts, true), fill: helmet ? C.metal : skinF, shade: 2.6, rim: 0.7 });
  const earP = H(-0.12 * r, 0.22 * r);
  if (!helmet) headLayerShapes.push({ path: ellipsePath(earP, 0.2 * r, 0.27 * r, sk.headTh), fill: skinF, shade: 0.8 });
  const face = pose.face ?? 'neutral';
  const eyesT = style.eyes ?? 'normal';
  const headAfter = (ctx: CanvasRenderingContext2D) => {
    ctx.save();
    ctx.translate(sk.head.x, sk.head.y); ctx.rotate(sk.headTh); ctx.scale(r, r);
    const OL = C.outline;
    ctx.lineCap = 'round'; ctx.lineJoin = 'round';
    if (!helmet) {
      // ear detail
      ctx.beginPath(); ctx.arc(-0.12, 0.22, 0.12, -1.2, 1.6); ctx.strokeStyle = rgba(shadeOf(skinF), 1); ctx.lineWidth = 0.06; ctx.stroke();
      // blush / cheek shade
      ctx.beginPath(); ctx.ellipse(0.55, 0.52, 0.16, 0.07, 0, 0, Math.PI * 2); ctx.fillStyle = rgba('#ff8080', sk.fem ? 0.35 : 0.12); ctx.fill();
    }
    const eye = (cx: number, cy: number, w: number, far: boolean) => {
      const ang = eyesT === 'fierce' || face === 'shout' || face === 'angry' ? 0.22 : eyesT === 'cute' ? -0.05 : 0.08;
      const hgt = (eyesT === 'cute' ? 0.3 : eyesT === 'fierce' ? 0.17 : 0.23) * (sk.fem ? 1.15 : 1);
      if (face === 'hurt' || face === 'ko' || face === 'closed') {
        ctx.beginPath();
        if (face === 'hurt') { ctx.moveTo(cx - w * 0.5, cy - hgt * 0.4); ctx.lineTo(cx + w * 0.1, cy); ctx.lineTo(cx - w * 0.4, cy + hgt * 0.35); }
        else { ctx.moveTo(cx - w * 0.5, cy); ctx.quadraticCurveTo(cx, cy + 0.08, cx + w * 0.5, cy); }
        ctx.strokeStyle = OL; ctx.lineWidth = 0.09; ctx.stroke();
        return;
      }
      if (eyesT === 'glow') {
        ctx.save(); ctx.shadowColor = C.eye; ctx.shadowBlur = 8;
        ctx.beginPath(); ctx.moveTo(cx - w * 0.55, cy - hgt * 0.15 - ang * 0.3); ctx.lineTo(cx + w * 0.5, cy + ang * 0.4); ctx.lineTo(cx - w * 0.3, cy + hgt * 0.5); ctx.closePath();
        ctx.fillStyle = C.eye; ctx.fill(); ctx.restore();
        ctx.beginPath(); ctx.moveTo(cx - w * 0.55, cy - hgt * 0.15 - ang * 0.3); ctx.lineTo(cx + w * 0.5, cy + ang * 0.4); ctx.strokeStyle = OL; ctx.lineWidth = 0.07; ctx.stroke();
        return;
      }
      // sclera
      ctx.beginPath(); ctx.moveTo(cx - w * 0.5, cy - hgt * 0.35 - ang * 0.3); ctx.quadraticCurveTo(cx, cy - hgt * 0.75, cx + w * 0.5, cy - hgt * 0.2 + ang * 0.25);
      ctx.lineTo(cx + w * 0.42, cy + hgt * 0.45); ctx.quadraticCurveTo(cx, cy + hgt * 0.62, cx - w * 0.45, cy + hgt * 0.35); ctx.closePath();
      ctx.fillStyle = '#ffffff'; ctx.fill();
      ctx.save(); ctx.clip();
      // iris
      const ix = cx + w * (far ? 0.12 : 0.1);
      const g = ctx.createLinearGradient(0, cy - hgt * 0.6, 0, cy + hgt * 0.6);
      g.addColorStop(0, shadeOf(C.eye)); g.addColorStop(1, lightOf(C.eye));
      ctx.beginPath(); ctx.ellipse(ix, cy + hgt * 0.05, w * 0.28, hgt * 0.62, 0, 0, Math.PI * 2); ctx.fillStyle = g; ctx.fill();
      ctx.beginPath(); ctx.ellipse(ix + w * 0.02, cy + hgt * 0.05, w * 0.12, hgt * 0.32, 0, 0, Math.PI * 2); ctx.fillStyle = '#0c0814'; ctx.fill();
      ctx.beginPath(); ctx.arc(ix - w * 0.08, cy - hgt * 0.2, hgt * 0.17, 0, Math.PI * 2); ctx.fillStyle = '#ffffff'; ctx.fill();
      ctx.restore();
      // upper lid
      ctx.beginPath(); ctx.moveTo(cx - w * 0.6, cy - hgt * 0.25 - ang * 0.35); ctx.quadraticCurveTo(cx, cy - hgt * 0.8, cx + w * 0.55, cy - hgt * 0.2 + ang * 0.3);
      ctx.strokeStyle = OL; ctx.lineWidth = 0.12; ctx.stroke();
      if (sk.fem) { ctx.beginPath(); ctx.moveTo(cx + w * 0.45, cy - hgt * 0.25 + ang * 0.3); ctx.lineTo(cx + w * 0.68, cy - hgt * 0.42 + ang * 0.3); ctx.lineWidth = 0.07; ctx.stroke(); }
    };
    const brow = (cx: number, cy: number, w: number) => {
      const ang = eyesT === 'fierce' || face === 'shout' || face === 'angry' || face === 'hurt' ? 0.22 : 0.02;
      ctx.beginPath(); ctx.moveTo(cx - w * 0.55, cy - 0.06 - ang * 0.2); ctx.lineTo(cx + w * 0.5, cy + ang * 0.6);
      ctx.strokeStyle = mix(C.hair, OL, 0.5); ctx.lineWidth = sk.fem ? 0.06 : 0.1; ctx.stroke();
    };
    if (!helmet || style.visor === false) {
      eye(0.36, 0.16, 0.36, false);
      eye(0.8, 0.15, 0.2, true);
      brow(0.36, -0.12, 0.42); brow(0.8, -0.12, 0.22);
      // nose
      ctx.beginPath(); ctx.moveTo(0.95, 0.3); ctx.lineTo(1.0, 0.44); ctx.lineTo(0.92, 0.46); ctx.strokeStyle = rgba(shadeOf(skinF), 1); ctx.lineWidth = 0.06; ctx.stroke();
      // mouth
      if (!style.mask) {
        ctx.beginPath();
        if (face === 'shout' || face === 'hurt' || face === 'ko') {
          ctx.moveTo(0.48, 0.66); ctx.quadraticCurveTo(0.68, 0.6, 0.82, 0.64); ctx.lineTo(0.75, 0.82); ctx.quadraticCurveTo(0.6, 0.86, 0.5, 0.74); ctx.closePath();
          ctx.fillStyle = '#5a1a26'; ctx.fill(); ctx.strokeStyle = OL; ctx.lineWidth = 0.06; ctx.stroke();
        } else if (face === 'smile') {
          ctx.moveTo(0.5, 0.66); ctx.quadraticCurveTo(0.66, 0.78, 0.82, 0.64); ctx.strokeStyle = OL; ctx.lineWidth = 0.07; ctx.stroke();
        } else {
          ctx.moveTo(0.54, 0.7); ctx.lineTo(0.8, 0.68); ctx.strokeStyle = sk.fem ? C.lips : OL; ctx.lineWidth = 0.07; ctx.stroke();
        }
      }
    }
    if (style.scar) { ctx.beginPath(); ctx.moveTo(0.5, -0.05); ctx.lineTo(0.75, 0.42); ctx.strokeStyle = rgba('#a03040', 0.9); ctx.lineWidth = 0.06; ctx.stroke(); }
    if (style.facePaint) { ctx.beginPath(); ctx.moveTo(0.25, 0.42); ctx.lineTo(0.62, 0.4); ctx.moveTo(0.3, 0.52); ctx.lineTo(0.6, 0.5); ctx.strokeStyle = C[style.facePaint] ?? style.facePaint; ctx.lineWidth = 0.07; ctx.stroke(); }
    ctx.restore();
  };
  layers.push({ shapes: headLayerShapes, after: headAfter });

  // mask (lower face)
  if (style.mask) {
    const m = Hp([[-0.1, 0.32], [0.55, 0.36], [1.02, 0.36], [0.98, 0.56], [0.64, 1.04], [0.18, 1.04], [-0.32, 0.8], [-0.3, 0.42]]);
    layers.push({ shapes: [{ path: polyPath(reg(m), true), fill: C.mask, shade: 1.6, line: true }] });
  }

  // ---------------- hair
  const hairShapes: Shape[] = [];
  const ht = hair.type ?? 'short';
  if (ht !== 'none' && ht !== 'bald' && !helmet) {
    const n = num(hair.spikes, ht === 'spiky' ? 8 : ht === 'wild' || ht === 'mane' ? 9 : ht === 'short' ? 7 : 6);
    const lmin = num(hair.min, ht === 'spiky' ? 0.7 : ht === 'wild' || ht === 'mane' ? 0.7 : ht === 'short' ? 0.22 : 0.3);
    const lmax = num(hair.max, ht === 'spiky' ? 1.5 : ht === 'wild' || ht === 'mane' ? 1.5 : ht === 'short' ? 0.5 : 0.55);
    const sweep = num(hair.sweep, ht === 'spiky' ? 28 : 40);
    const a0 = ht === 'mane' || ht === 'wild' ? 115 : ht === 'long' || ht === 'flowing' ? 130 : 150, a1 = num(hair.front, 318);
    const pts: [number, number][] = [];
    const hairWind = num(pose.hairWind, wind) * 14;
    // hairline back -> front (inner), then spikes front -> back (outer)
    pts.push([-0.28, 0.2], [-0.05, -0.08], [0.25, -0.42], [0.62, -0.42], [0.88, -0.38]);
    for (let i = n - 1; i >= 0; i--) {
      const t0 = i / n, t1 = (i + 1) / n;
      const aV = (a0 + (a1 - a0) * t1) * D2R;
      const aT = (a0 + (a1 - a0) * (t0 + t1) / 2);
      const ln = lmin + (lmax - lmin) * (0.45 + 0.55 * Math.sin(Math.PI * Math.min(1, (t0 + 0.15) * 1.05))) * (0.75 + 0.5 * hash(i + n * 7 + (hair.seed ?? 0)));
      const vR = 1.06;
      pts.push([Math.cos(aV) * vR * 0.95, Math.sin(aV) * vR]);
      const td = (aT - sweep * (0.6 + 0.4 * (1 - t0)) - hairWind) * D2R;
      const base: [number, number] = [Math.cos(aT * D2R) * 0.95, Math.sin(aT * D2R)];
      pts.push([base[0] + Math.cos(td) * ln, base[1] + Math.sin(td) * ln]);
    }
    const aE = a0 * D2R;
    pts.push([Math.cos(aE) * 1.04 * 0.95, Math.sin(aE) * 1.04]);
    if (ht === 'mane' || ht === 'wild' || ht === 'long' || ht === 'flowing') pts.push([-0.7, 0.95], [-0.45, 0.6]);
    else pts.push([-0.62, 0.72], [-0.42, 0.45]);
    const hp = reg(Hp(pts), 1);
    hairShapes.push({ path: mixedPath(hp.map((p, i) => ({ p, c: i >= 5 && (i - 5) % 2 === 1 }))), fill: hcol, shade: 3, rim: 0.9 });
    // bangs
    const bangs = num(hair.bangs, ht === 'spiky' ? 3 : ht === 'short' ? 2 : 3);
    for (let i = 0; i < bangs; i++) {
      const bx = 0.22 + i * 0.26, by = -0.62 - (i === 0 ? 0.1 : 0);
      const tipL = num(hair.bangLen, ht === 'long' || ht === 'flowing' || ht === 'ponytail' ? 0.55 : 0.42) * (1 - i * 0.15);
      const tip: [number, number] = [bx + 0.18 + i * 0.05, by + tipL + 0.15];
      const bp = Hp([[bx - 0.2, by - 0.15], [bx + 0.25, by - 0.05], tip]);
      hairShapes.push({ path: mixedPath(bp.map((p, k) => ({ p, c: k === 2 }))), fill: hcol, shade: 1.6, noOutline: false });
    }
    if (ht === 'topknot' || ht === 'bun') {
      const k = Hp([[-0.55, -1.05]]);
      hairShapes.push({ path: ellipsePath(k[0], 0.38 * r, 0.32 * r, sk.headTh), fill: hcol, shade: 1.5 });
    }
  }
  if (helmet) {
    // android helmet: crest + visor
    const crest = Hp([[-0.9, -0.2], [-0.6, -0.95], [0.1, -1.18], [0.65, -0.9], [0.95, -0.35], [0.98, -0.05], [0.5, -0.12], [-0.3, 0.1], [-0.55, 0.5], [-0.85, 0.4]]);
    hairShapes.push({ path: polyPath(reg(crest), true), fill: C.hair, shade: 2.4, rim: 0.8, line: true });
    const fin = Hp([[-0.5, -0.95], [-1.45, -1.3], [-1.6, -1.1], [-0.85, -0.55]]);
    hairShapes.push({ path: polyPath(reg(fin), false), fill: C.accent, shade: 0.8 });
  }
  const hairAfter = (ctx: CanvasRenderingContext2D) => {
    if (helmet && style.visor !== false) {
      const v = Hp([[0.12, 0.0], [1.02, -0.02], [1.0, 0.32], [0.2, 0.3]]);
      ctx.save(); ctx.shadowColor = C.visor; ctx.shadowBlur = 10;
      ctx.beginPath(); polyPath(v, false)(ctx, 0, 0); ctx.fillStyle = C.visor; ctx.fill(); ctx.restore();
      ctx.beginPath(); polyPath(v, false)(ctx, 0, 0); ctx.strokeStyle = C.outline; ctx.lineWidth = 0.6; ctx.stroke();
      const hl = Hp([[0.3, 0.08], [0.9, 0.06]]);
      ctx.beginPath(); ctx.moveTo(hl[0].x, hl[0].y); ctx.lineTo(hl[1].x, hl[1].y); ctx.strokeStyle = '#ffffff'; ctx.lineWidth = 0.5; ctx.stroke();
      // mouth grill
      const g = Hp([[0.55, 0.62], [0.9, 0.6]]);
      ctx.beginPath(); ctx.moveTo(g[0].x, g[0].y); ctx.lineTo(g[1].x, g[1].y); ctx.strokeStyle = shadeOf(C.metal); ctx.lineWidth = 0.6; ctx.stroke();
    }
    // hair strands
    if (!helmet && ht !== 'none' && ht !== 'bald') {
      const s1 = Hp([[-0.2, -0.75], [0.25, -0.9], [0.55, -0.7]]);
      ctx.beginPath(); ctx.moveTo(s1[0].x, s1[0].y); ctx.quadraticCurveTo(s1[1].x, s1[1].y, s1[2].x, s1[2].y);
      ctx.strokeStyle = rgba(lightOf(hcol), 0.75); ctx.lineWidth = 0.9; ctx.stroke();
    }
  };
  if (hairShapes.length) layers.push({ shapes: hairShapes, after: hairAfter });

  // headband / horns / hat
  const topShapes: Shape[] = [];
  if (style.headband) {
    const hb = style.headband;
    const band = Hp([[-0.98, -0.6], [-0.3, -0.8], [0.5, -0.78], [0.97, -0.6], [1.0, -0.38], [0.5, -0.53], [-0.3, -0.55], [-0.98, -0.36]]);
    topShapes.push({ path: polyPath(reg(band), true), fill: C[hb.color ?? 'top2'] ?? hb.color, shade: 1, line: true });
    topShapes.push({ path: ellipsePath(H(-0.98 * r, -0.48 * r), 2.0 * S, 1.6 * S), fill: C[hb.color ?? 'top2'] ?? hb.color, shade: 0.6 });
  }
  if (style.horns) {
    const hc = C.horn;
    const big = num(style.horns.size, 1);
    const horn = (bx: number, by: number, ang: number, l: number, w: number, curl: number) => {
      const pts: P[] = [];
      const base = H(bx * r, by * r);
      const N = 6;
      const spine: P[] = [];
      for (let i = 0; i <= N; i++) { const t = i / N; spine.push(add(base, dir(ang - (sk.headTh / D2R) + curl * t * t), l * t)); }
      const rp = ribbonPts(spine, w, 0.3);
      pts.push(...rp);
      return reg(pts);
    };
    topShapes.push({ path: polyPath(horn(-0.35, -0.8, 160, 14 * big * S, 4.2 * S, -45), true), fill: backOf(hc), shade: 1.2 });
    topShapes.push({ path: polyPath(horn(0.35, -0.85, 155, 17 * big * S, 5 * S, 55), true), fill: hc, shade: 1.6 });
  }
  if (style.hat) {
    const hc = C.hat;
    const brim = Hp([[-1.6, -0.45], [-0.9, -0.75], [0, -0.82], [0.9, -0.72], [1.65, -0.5], [1.5, -0.3], [0.6, -0.5], [-0.6, -0.5], [-1.4, -0.3]]);
    const crown = Hp([[-0.85, -0.6], [-0.75, -1.35], [0, -1.55], [0.75, -1.35], [0.85, -0.6]]);
    topShapes.push({ path: polyPath(reg(crown), true), fill: hc, shade: 2, line: true });
    topShapes.push({ path: polyPath(reg(brim), true), fill: hc, shade: 1.2, line: true });
    const band = Hp([[-0.83, -0.62], [0.83, -0.62], [0.8, -0.82], [-0.8, -0.82]]);
    topShapes.push({ path: polyPath(band, false), fill: C.accent, shade: 0.5, noOutline: true });
  }
  if (topShapes.length) layers.push({ shapes: topShapes });

  // ---------------- front arm (+weapon in front hand before it)
  if (wl && !weaponBack) layers.push(wl);
  layers.push({ shapes: armShapes('F') });
  if (style.shoulderPads) {
    const d = dir(sk.upAF);
    layers.push({ shapes: [{ path: ellipsePath(add(sk.shF, d, 1.5), 6.2 * W, 4.4 * W, Math.atan2(d.y, d.x) + Math.PI / 2), fill: C.metal, shade: 1.8, rim: 0.6, line: true }] });
  }
  // charge / energy in hands
  if (pose.charge) {
    const which = pose.charge === 'both' ? [sk.handF, sk.handB] : pose.charge === 'B' ? [sk.handB] : [sk.handF];
    const rr = num(pose.chargeSize, 7) * S;
    for (const p of which) reg([p], rr * 1.8);
    layers.push({ shapes: [], after: (ctx) => { for (const p of which) glowOrb(ctx, p, rr, C.energy, phase); } });
  }
  return layers;
}
function legSkinOr(pantsT: string, pantsC: string, C: Record<string, string>) { return pantsT === 'skirt' ? (C.top) : pantsC; }

function glowOrb(ctx: CanvasRenderingContext2D, p: P, r: number, col: string, phase: number) {
  ctx.save();
  ctx.globalCompositeOperation = 'lighter';
  const g = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, r * 1.8);
  g.addColorStop(0, rgba('#ffffff', 0.95)); g.addColorStop(0.25, rgba(lightOf(col), 0.9)); g.addColorStop(0.55, rgba(col, 0.55)); g.addColorStop(1, rgba(col, 0));
  ctx.fillStyle = g; ctx.beginPath(); ctx.arc(p.x, p.y, r * 1.8, 0, Math.PI * 2); ctx.fill();
  ctx.strokeStyle = rgba(lightOf(col), 0.8); ctx.lineWidth = 0.6;
  for (let i = 0; i < 3; i++) { ctx.beginPath(); ctx.arc(p.x, p.y, r * (0.8 + i * 0.25), phase + i * 2.1, phase + i * 2.1 + 1.6); ctx.stroke(); }
  ctx.restore();
}

const LIGHT: P = norm({ x: 0.55, y: -0.85 });
function drawLayer(ctx: CanvasRenderingContext2D, ly: Layer, C: Record<string, string>) {
  const OL = ly.outline ?? 1.15;
  ctx.lineJoin = 'round'; ctx.lineCap = 'round';
  if (ly.before) ly.before(ctx);
  if (ly.shapes.length) {
    ctx.save();
    if (ly.aura) { ctx.shadowColor = ly.aura; ctx.shadowBlur = 14 * RES; }
    ctx.strokeStyle = C.outline; ctx.lineWidth = OL * 2;
    ctx.fillStyle = C.outline;
    for (const s of ly.shapes) if (!s.noOutline) { ctx.beginPath(); s.path(ctx, 0, 0); ctx.stroke(); }
    ctx.restore();
    for (const s of ly.shapes) {
      ctx.save();
      ctx.beginPath(); s.path(ctx, 0, 0);
      if (s.flat) { ctx.fillStyle = s.fill; ctx.fill(); ctx.restore(); continue; }
      ctx.clip();
      ctx.fillStyle = shadeOf(s.fill); ctx.fill();
      const d = s.shade ?? 2;
      const h = s.rim ?? (lum(s.fill) < 0.2 ? 0.55 : 0);
      if (d > 0) {
        if (h > 0) { ctx.beginPath(); s.path(ctx, LIGHT.x * d, LIGHT.y * d); ctx.fillStyle = lightOf(s.fill); ctx.fill(); }
        ctx.save();
        ctx.beginPath(); s.path(ctx, LIGHT.x * d, LIGHT.y * d); ctx.clip();
        ctx.beginPath(); s.path(ctx, -LIGHT.x * h, -LIGHT.y * h); ctx.fillStyle = s.fill; ctx.fill();
        ctx.restore();
      } else { ctx.beginPath(); s.path(ctx, 0, 0); ctx.fillStyle = s.fill; ctx.fill(); }
      ctx.restore();
    }
    for (const s of ly.shapes) if (s.line) { ctx.beginPath(); s.path(ctx, 0, 0); ctx.strokeStyle = C.outline; ctx.lineWidth = 0.45; ctx.stroke(); }
  }
  if (ly.after) { ctx.save(); ly.after(ctx); ctx.restore(); }
}

export function renderPose(style: Record<string, any>, pose: Record<string, any>, active: ActiveSpec | undefined, factory: CanvasFactory, palette?: Record<string, any>): RigRender {
  style = style ?? {};
  pose = pose ?? {};
  const C = colorsOf(style, palette);
  const sk = solve(style, pose);
  const wg = weaponGeo(style, sk);
  const { hurt, hit } = computeBoxes(sk, active, wg);
  const bb = new BBox();
  const layers = buildLayers(style, sk, C, wg, bb);
  for (const b of [...hurt, ...hit]) { bb.addP({ x: b[0], y: b[1] }); bb.addP({ x: b[2], y: b[3] }); }
  const aura = pose.aura ? (C.aura) : null;
  const pad = PAD + (aura ? 5 : 0);
  const x1 = Math.floor(bb.x1 - pad), y1 = Math.floor(bb.y1 - pad), x2 = Math.ceil(bb.x2 + pad), y2 = Math.ceil(Math.max(bb.y2, 0) + pad);
  const width = (x2 - x1) * RES, height = (y2 - y1) * RES;
  const axisX = -x1 * RES, axisY = -y1 * RES;
  const made = factory(width, height);
  if (made) {
    const ctx = made.ctx;
    ctx.save();
    ctx.translate(axisX, axisY);
    ctx.scale(RES, RES);
    if (aura) {
      // aura flame behind the body
      ctx.save();
      ctx.globalCompositeOperation = 'lighter';
      const c = lerpP(sk.hip, sk.neck, 0.4);
      const g = ctx.createRadialGradient(c.x, c.y, 4, c.x, c.y, 62 * sk.S);
      g.addColorStop(0, rgba(aura, 0.5)); g.addColorStop(0.6, rgba(aura, 0.18)); g.addColorStop(1, rgba(aura, 0));
      ctx.fillStyle = g;
      ctx.beginPath();
      const ph = num(pose.phase, 0);
      for (let i = 0; i <= 24; i++) {
        const a = (i / 24) * Math.PI * 2;
        const rr = (34 + (Math.sin(a * 5 + ph * 3) * 0.5 + 0.5) * 16 + (Math.sin(a) < 0 ? 14 : 0)) * sk.S;
        const p = { x: c.x + Math.sin(a) * rr * 0.75, y: c.y - Math.cos(a) * rr * 1.05 };
        if (i === 0) ctx.moveTo(p.x, p.y); else ctx.lineTo(p.x, p.y);
      }
      ctx.fill();
      ctx.restore();
    }
    for (const ly of layers) {
      if (aura && !ly.aura && ly.shapes.length) ly.aura = aura;
      drawLayer(ctx, ly, C);
    }
    if (pose.flash) {
      ctx.globalCompositeOperation = 'source-atop';
      ctx.fillStyle = rgba(pose.flash === true ? '#ffffff' : pose.flash, 0.55);
      ctx.fillRect(x1, y1, x2 - x1, y2 - y1);
    }
    ctx.restore();
  }
  return { canvas: made ? made.canvas : null, width, height, axisX, axisY, scale: 1 / RES, hurt, hit };
}

// ------------------------------------------------------------------ effects
/**
 * effect: { type, size?, length?, color?, color2?, phase?, angle? }
 * types: orb, slash, beam, fire, darkfire, ice, lightning, kunai, shuriken, water, wave, pillar, shockwave, burst, petal
 * The axis is the effect centre (beam: its origin on the left; pillar/shockwave: ground centre).
 */
export function renderEffect(effect: Record<string, any>, factory: CanvasFactory, palette?: Record<string, any>): RigRender {
  const e = effect ?? {};
  const C = { ...DEFAULT_COLORS, ...(palette ? palette.colors ?? palette : {}) };
  const type = e.type ?? 'orb';
  const size = num(e.size, 14);
  const ph = num(e.phase, 0);
  const col: string = e.color ?? C.energy ?? '#7fd4ff';
  const col2: string = e.color2 ?? lightOf(col);
  // geometry: body box (hit) and drawing bbox
  let body: Box;
  let draw: Box;
  switch (type) {
    case 'beam': { const L = num(e.length, 200); body = [0, -size, L, size]; draw = [-size * 1.5, -size * 2.2, L + size * 1.8, size * 2.2]; break; }
    case 'pillar': { const h = num(e.length, 90); body = [-size, -h, size, 0]; draw = [-size * 2, -h - size, size * 2, size * 0.6]; break; }
    case 'shockwave': { body = [-size, -size * 1.4, size * 1.1, 0]; draw = [-size * 2.4, -size * 2.2, size * 1.8, size * 0.5]; break; }
    case 'slash': case 'water': case 'wave': { body = [-size * 0.35, -size, size * 0.55, size]; draw = [-size * 1.6, -size * 1.4, size * 1.2, size * 1.4]; break; }
    case 'kunai': case 'shuriken': { body = [-size * 0.6, -size * 0.45, size * 0.8, size * 0.45]; draw = [-size * 3.4, -size * 1.4, size * 1.6, size * 1.4]; break; }
    case 'ice': { body = [-size * 0.8, -size * 0.6, size * 1.1, size * 0.6]; draw = [-size * 2.6, -size * 1.6, size * 2, size * 1.6]; break; }
    case 'burst': { body = [-size * 0.6, -size * 0.6, size * 0.6, size * 0.6]; draw = [-size * 1.8, -size * 1.8, size * 1.8, size * 1.8]; break; }
    default: { body = [-size * 0.8, -size * 0.8, size * 0.9, size * 0.8]; draw = [-size * 3.2, -size * 1.9, size * 1.9, size * 1.9]; }
  }
  body = body.map((v) => Math.round(v)) as Box;
  const x1 = Math.floor(draw[0] - 4), y1 = Math.floor(draw[1] - 4), x2 = Math.ceil(draw[2] + 4), y2 = Math.ceil(draw[3] + 4);
  const width = (x2 - x1) * RES, height = (y2 - y1) * RES, axisX = -x1 * RES, axisY = -y1 * RES;
  const nohit = !!e.nohit;
  const out: RigRender = { canvas: null, width, height, axisX, axisY, scale: 1 / RES, hit: nohit ? [] : [body], hurt: nohit ? [] : [body] };
  const made = factory(width, height);
  if (!made) return out;
  out.canvas = made.canvas;
  const ctx = made.ctx;
  ctx.save();
  ctx.translate(axisX, axisY);
  ctx.scale(RES, RES);
  ctx.lineCap = 'round'; ctx.lineJoin = 'round';
  drawEffect(ctx, type, size, ph, col, col2, e);
  ctx.restore();
  return out;
}

function radial(ctx: CanvasRenderingContext2D, x: number, y: number, r: number, stops: [number, string][]) {
  const g = ctx.createRadialGradient(x, y, 0, x, y, r);
  for (const [o, c] of stops) g.addColorStop(o, c);
  ctx.fillStyle = g;
  ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI * 2); ctx.fill();
}
function glowStroke(ctx: CanvasRenderingContext2D, col: string, w: number, blur: number, fn: () => void) {
  ctx.save(); ctx.shadowColor = col; ctx.shadowBlur = blur * RES; ctx.strokeStyle = col; ctx.lineWidth = w; ctx.beginPath(); fn(); ctx.stroke(); ctx.restore();
}
function glowFill(ctx: CanvasRenderingContext2D, col: string, blur: number, fn: () => void) {
  ctx.save(); ctx.shadowColor = col; ctx.shadowBlur = blur * RES; ctx.fillStyle = col; ctx.beginPath(); fn(); ctx.fill(); ctx.restore();
}

function drawEffect(ctx: CanvasRenderingContext2D, type: string, s: number, ph: number, col: string, col2: string, e: Record<string, any>) {
  const W = '#ffffff';
  ctx.globalCompositeOperation = 'lighter';
  switch (type) {
    case 'orb': case 'fire': case 'darkfire': {
      const fire = type !== 'orb';
      // trail
      for (let i = 0; i < 6; i++) {
        const t = i / 6;
        const x = -s * (0.6 + t * 2.2), y = Math.sin(ph * 2 + i * 1.3) * s * 0.18 * t;
        radial(ctx, x, y, s * (0.85 - t * 0.55), [[0, rgba(col, 0.55 * (1 - t))], [1, rgba(col, 0)]]);
      }
      if (fire) {
        for (let i = 0; i < 9; i++) {
          const a = (i / 9) * Math.PI * 2 + ph;
          const fl = s * (1.0 + 0.5 * hash(i + ph * 10));
          ctx.save();
          ctx.translate(Math.cos(a) * s * 0.3, Math.sin(a) * s * 0.3);
          glowFill(ctx, rgba(col, 0.75), 6, () => {
            ctx.moveTo(0, -s * 0.45); ctx.quadraticCurveTo(-fl * 1.3, -s * 0.5 + Math.sin(a) * 4, -fl * 1.9, Math.sin(a * 2) * s * 0.4);
            ctx.quadraticCurveTo(-fl * 1.1, s * 0.5, 0, s * 0.45); ctx.closePath();
          });
          ctx.restore();
        }
      }
      radial(ctx, 0, 0, s * 1.9, [[0, rgba(col, 0.8)], [0.45, rgba(col, 0.35)], [1, rgba(col, 0)]]);
      radial(ctx, 0, 0, s * 0.95, [[0, rgba(W, 1)], [0.5, rgba(col2, 0.95)], [1, rgba(col, 0.2)]]);
      // swirl
      for (let i = 0; i < 3; i++) glowStroke(ctx, rgba(col2, 0.9), 1.1, 4, () => ctx.arc(0, 0, s * (0.75 + i * 0.22), ph * 2 + i * 2.1, ph * 2 + i * 2.1 + 2.2));
      break;
    }
    case 'slash': case 'water': case 'wave': {
      const water = type !== 'slash';
      const h = s;
      const crescent = (sc: number) => { ctx.moveTo(0, -h * sc); ctx.quadraticCurveTo(h * 0.95 * sc, 0, 0, h * sc); ctx.quadraticCurveTo(h * 0.35 * sc, 0, 0, -h * sc); ctx.closePath(); };
      // after images
      for (let i = 3; i >= 1; i--) { ctx.save(); ctx.translate(-i * s * 0.28, 0); ctx.globalAlpha = 0.18 * (4 - i); glowFill(ctx, col, 4, () => crescent(1 - i * 0.07)); ctx.restore(); }
      glowFill(ctx, rgba(col, 0.95), 10, () => crescent(1.05));
      ctx.save(); ctx.translate(h * 0.06, 0); glowFill(ctx, rgba(col2, 1), 3, () => crescent(0.85)); ctx.restore();
      ctx.save(); ctx.translate(h * 0.12, 0); ctx.fillStyle = W; ctx.beginPath(); crescent(0.62); ctx.fill(); ctx.restore();
      if (water) {
        for (let i = 0; i < 9; i++) {
          const y = (hash(i + 1) * 2 - 1) * h * 0.9, x = -s * (0.2 + hash(i + 5 + ph) * 1.1);
          radial(ctx, x, y, 1.2 + hash(i * 3) * 2.2, [[0, rgba(W, 0.95)], [1, rgba(col, 0)]]);
        }
        glowStroke(ctx, rgba(W, 0.8), 0.8, 3, () => { ctx.moveTo(-s * 0.6, -h * 0.6); ctx.quadraticCurveTo(-s * 0.2 + Math.sin(ph) * 3, 0, -s * 0.6, h * 0.6); });
      } else {
        for (let i = 0; i < 6; i++) { const y = (i / 5 - 0.5) * h * 1.6; glowStroke(ctx, rgba(col2, 0.6), 0.7, 2, () => { ctx.moveTo(-s * (0.3 + hash(i + ph) * 0.4), y); ctx.lineTo(-s * (1.0 + hash(i * 7 + ph) * 0.6), y * 0.9); }); }
      }
      break;
    }
    case 'beam': {
      const L = num(e.length, 200);
      const wob = (x: number, k: number) => Math.sin(x * 0.12 + ph * 3 + k) * s * 0.12;
      const band = (th: number, k: number) => {
        ctx.moveTo(0, -th);
        for (let x = 0; x <= L; x += 8) ctx.lineTo(x, -th - wob(x, k));
        ctx.arc(L, 0, th, -Math.PI / 2, Math.PI / 2);
        for (let x = L; x >= 0; x -= 8) ctx.lineTo(x, th + wob(x, k + 1));
        ctx.closePath();
      };
      glowFill(ctx, rgba(col, 0.55), 14, () => band(s * 1.25, 0));
      glowFill(ctx, rgba(col, 0.9), 6, () => band(s * 0.85, 2));
      ctx.fillStyle = rgba(col2, 1); ctx.beginPath(); band(s * 0.55, 4); ctx.fill();
      ctx.fillStyle = W; ctx.beginPath(); band(s * 0.28, 6); ctx.fill();
      // origin flare and head
      radial(ctx, 0, 0, s * 2.1, [[0, rgba(W, 1)], [0.35, rgba(col2, 0.8)], [1, rgba(col, 0)]]);
      radial(ctx, L, 0, s * 1.8, [[0, rgba(W, 1)], [0.4, rgba(col2, 0.7)], [1, rgba(col, 0)]]);
      // energy streaks
      for (let i = 0; i < 10; i++) {
        const x = ((hash(i + 2) * L + ph * 40) % L), y = (hash(i + 9) * 2 - 1) * s * 0.9;
        glowStroke(ctx, rgba(W, 0.7), 0.7, 2, () => { ctx.moveTo(x, y); ctx.lineTo(x + 14 + hash(i) * 20, y); });
      }
      break;
    }
    case 'ice': {
      const shard = (x: number, y: number, l: number, w: number, a: number) => {
        ctx.save(); ctx.translate(x, y); ctx.rotate(a);
        glowFill(ctx, rgba(col, 0.75), 6, () => { ctx.moveTo(l, 0); ctx.lineTo(0, -w); ctx.lineTo(-l * 0.5, 0); ctx.lineTo(0, w); ctx.closePath(); });
        ctx.fillStyle = rgba(W, 0.85); ctx.beginPath(); ctx.moveTo(l, 0); ctx.lineTo(0, -w); ctx.lineTo(-l * 0.1, 0); ctx.closePath(); ctx.fill();
        ctx.strokeStyle = rgba(W, 0.9); ctx.lineWidth = 0.5; ctx.beginPath(); ctx.moveTo(l, 0); ctx.lineTo(0, -w); ctx.lineTo(-l * 0.5, 0); ctx.lineTo(0, w); ctx.closePath(); ctx.stroke();
        ctx.restore();
      };
      for (let i = 0; i < 5; i++) { const t = i / 5; radial(ctx, -s * (0.8 + t * 1.6), Math.sin(i * 2 + ph) * s * 0.3, s * (0.5 - t * 0.3), [[0, rgba(col, 0.5)], [1, rgba(col, 0)]]); }
      radial(ctx, 0, 0, s * 1.5, [[0, rgba(col, 0.6)], [1, rgba(col, 0)]]);
      shard(-s * 0.3, -s * 0.45, s * 0.9, s * 0.25, -0.25 + Math.sin(ph) * 0.05);
      shard(-s * 0.3, s * 0.45, s * 0.9, s * 0.25, 0.25);
      shard(0, 0, s * 1.15, s * 0.4, 0);
      for (let i = 0; i < 6; i++) radial(ctx, -s * (0.6 + hash(i + ph * 3) * 1.6), (hash(i * 5) * 2 - 1) * s * 0.8, 1.4, [[0, rgba(W, 1)], [1, rgba(col, 0)]]);
      break;
    }
    case 'lightning': {
      radial(ctx, 0, 0, s * 1.8, [[0, rgba(col, 0.7)], [0.5, rgba(col, 0.25)], [1, rgba(col, 0)]]);
      radial(ctx, 0, 0, s * 0.8, [[0, rgba(W, 1)], [0.6, rgba(col2, 0.8)], [1, rgba(col, 0)]]);
      for (let k = 0; k < 6; k++) {
        const a0 = (k / 6) * Math.PI * 2 + ph * 1.7;
        glowStroke(ctx, rgba(col2, 1), 0.9, 5, () => {
          let x = 0, y = 0; ctx.moveTo(0, 0);
          for (let i = 1; i <= 5; i++) { const a = a0 + (hash(k * 10 + i + Math.floor(ph * 7)) - 0.5) * 1.3; x += Math.cos(a) * s * 0.33; y += Math.sin(a) * s * 0.33; ctx.lineTo(x, y); }
        });
      }
      for (let i = 0; i < 4; i++) glowStroke(ctx, rgba(W, 0.9), 0.6, 2, () => { let x = -s * 0.3, y = (i - 1.5) * s * 0.3; ctx.moveTo(x, y); for (let j = 0; j < 5; j++) { x -= s * 0.45; y += (hash(i * 9 + j + ph * 5) - 0.5) * s * 0.5; ctx.lineTo(x, y); } });
      break;
    }
    case 'kunai': case 'shuriken': {
      // motion streaks
      for (let i = 0; i < 3; i++) glowStroke(ctx, rgba(col, 0.6 - i * 0.15), 1.2 - i * 0.3, 3, () => { ctx.moveTo(-s * 0.4, (i - 1) * s * 0.25); ctx.lineTo(-s * (2.6 - i * 0.4), (i - 1) * s * 0.3); });
      ctx.globalCompositeOperation = 'source-over';
      if (type === 'kunai') {
        ctx.save(); ctx.shadowColor = col; ctx.shadowBlur = 5 * RES;
        ctx.fillStyle = '#dfe6f0'; ctx.strokeStyle = '#141018'; ctx.lineWidth = 0.6;
        ctx.beginPath(); ctx.moveTo(s * 0.9, 0); ctx.lineTo(0, -s * 0.28); ctx.lineTo(-s * 0.1, 0); ctx.lineTo(0, s * 0.28); ctx.closePath(); ctx.fill(); ctx.stroke();
        ctx.restore();
        ctx.fillStyle = '#7a8496'; ctx.beginPath(); ctx.moveTo(s * 0.9, 0); ctx.lineTo(0, s * 0.28); ctx.lineTo(-s * 0.1, 0); ctx.closePath(); ctx.fill();
        ctx.fillStyle = '#2a2230'; ctx.fillRect(-s * 0.55, -s * 0.08, s * 0.45, s * 0.16);
        ctx.strokeStyle = '#2a2230'; ctx.lineWidth = 0.8; ctx.beginPath(); ctx.arc(-s * 0.7, 0, s * 0.14, 0, Math.PI * 2); ctx.stroke();
      } else {
        ctx.save(); ctx.rotate(ph * 4); ctx.shadowColor = col; ctx.shadowBlur = 5 * RES;
        ctx.fillStyle = '#c9d2de'; ctx.strokeStyle = '#141018'; ctx.lineWidth = 0.6; ctx.beginPath();
        for (let i = 0; i < 8; i++) { const a = (i / 8) * Math.PI * 2; const rr = i % 2 ? s * 0.18 : s * 0.6; ctx.lineTo(Math.cos(a) * rr, Math.sin(a) * rr); }
        ctx.closePath(); ctx.fill(); ctx.stroke(); ctx.restore();
        ctx.fillStyle = '#2a2230'; ctx.beginPath(); ctx.arc(0, 0, s * 0.08, 0, Math.PI * 2); ctx.fill();
      }
      break;
    }
    case 'pillar': {
      const h = num(e.length, 90);
      glowFill(ctx, rgba(col, 0.55), 12, () => { ctx.moveTo(-s * 1.3, 0); for (let y = 0; y >= -h; y -= 6) ctx.lineTo(-s * (1.1 - (-y / h) * 0.4) + Math.sin(y * 0.2 + ph * 4) * 2, y); ctx.lineTo(0, -h - s * 0.8); for (let y = -h; y <= 0; y += 6) ctx.lineTo(s * (1.1 - (-y / h) * 0.4) + Math.sin(y * 0.25 + ph * 4 + 1) * 2, y); ctx.closePath(); });
      ctx.fillStyle = rgba(col2, 0.95); ctx.beginPath(); ctx.moveTo(-s * 0.6, 0); ctx.quadraticCurveTo(-s * 0.3, -h * 0.6, 0, -h); ctx.quadraticCurveTo(s * 0.3, -h * 0.6, s * 0.6, 0); ctx.fill();
      ctx.fillStyle = rgba(W, 0.9); ctx.beginPath(); ctx.moveTo(-s * 0.25, 0); ctx.quadraticCurveTo(0, -h * 0.5, 0, -h * 0.8); ctx.quadraticCurveTo(0, -h * 0.5, s * 0.25, 0); ctx.fill();
      radial(ctx, 0, 0, s * 2, [[0, rgba(W, 0.9)], [0.4, rgba(col, 0.6)], [1, rgba(col, 0)]]);
      break;
    }
    case 'shockwave': {
      for (let i = 0; i < 3; i++) {
        const x = -i * s * 0.6;
        glowFill(ctx, rgba(col, 0.6 - i * 0.15), 8, () => { ctx.moveTo(x - s, 0); ctx.quadraticCurveTo(x - s * 0.2, -s * (1.6 - i * 0.3) - Math.sin(ph * 3) * 3, x + s, 0); ctx.closePath(); });
      }
      ctx.globalCompositeOperation = 'source-over';
      for (let i = 0; i < 5; i++) {
        const x = (hash(i + 4) - 0.6) * s * 2, y = -hash(i + 8 + ph) * s * 1.4, rr = 1.5 + hash(i) * 2.5;
        ctx.fillStyle = '#6a5848'; ctx.strokeStyle = '#1a1210'; ctx.lineWidth = 0.5;
        ctx.beginPath(); ctx.moveTo(x - rr, y); ctx.lineTo(x, y - rr); ctx.lineTo(x + rr, y + rr * 0.3); ctx.lineTo(x - rr * 0.2, y + rr); ctx.closePath(); ctx.fill(); ctx.stroke();
      }
      break;
    }
    case 'burst': {
      const t = Math.min(1, Math.max(0, ph));
      const R = s * (0.6 + t * 1.1);
      radial(ctx, 0, 0, R, [[0, rgba(W, 1 - t * 0.7)], [0.4, rgba(col2, 0.8 * (1 - t))], [1, rgba(col, 0)]]);
      glowStroke(ctx, rgba(col2, 1 - t), 1.6 * (1 - t) + 0.3, 6, () => ctx.arc(0, 0, R * 0.85, 0, Math.PI * 2));
      for (let i = 0; i < 8; i++) { const a = (i / 8) * Math.PI * 2 + 0.3; glowStroke(ctx, rgba(W, 1 - t), 0.9, 3, () => { ctx.moveTo(Math.cos(a) * R * 0.5, Math.sin(a) * R * 0.5); ctx.lineTo(Math.cos(a) * R * 1.3, Math.sin(a) * R * 1.3); }); }
      break;
    }
    case 'petal': default: {
      // lotus / petal burst: trailing petals, a ring of petals and a hot core
      for (let i = 0; i < 7; i++) {
        const x = -s * (0.8 + hash(i + 1) * 1.8), y = (hash(i + 7) * 2 - 1) * s * 0.8, a = ph * 3 + i;
        ctx.save(); ctx.translate(x, y); ctx.rotate(a);
        glowFill(ctx, rgba(col, 0.7 - i * 0.07), 3, () => { ctx.moveTo(-s * 0.2, 0); ctx.quadraticCurveTo(0, -s * 0.16, s * 0.2, 0); ctx.quadraticCurveTo(0, s * 0.16, -s * 0.2, 0); });
        ctx.restore();
      }
      radial(ctx, 0, 0, s * 1.5, [[0, rgba(col, 0.55)], [1, rgba(col, 0)]]);
      for (let i = 0; i < 6; i++) {
        const a = (i / 6) * Math.PI * 2 + ph * 1.5;
        ctx.save(); ctx.rotate(a);
        glowFill(ctx, rgba(col, 0.9), 5, () => { ctx.moveTo(s * 0.15, 0); ctx.quadraticCurveTo(s * 0.55, -s * 0.38, s * 1.05, 0); ctx.quadraticCurveTo(s * 0.55, s * 0.38, s * 0.15, 0); });
        ctx.fillStyle = rgba(col2, 0.9); ctx.beginPath(); ctx.moveTo(s * 0.2, 0); ctx.quadraticCurveTo(s * 0.5, -s * 0.16, s * 0.85, 0); ctx.quadraticCurveTo(s * 0.5, s * 0.16, s * 0.2, 0); ctx.fill();
        ctx.restore();
      }
      radial(ctx, 0, 0, s * 0.55, [[0, rgba(W, 1)], [0.6, rgba(col2, 0.9)], [1, rgba(col, 0)]]);
    }
  }
  void angOf;
}
