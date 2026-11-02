/**
 * Shared building blocks for the original launch roster: a generic pose library (driven by a
 * per-fighter stance), the standard animation set used by the engine's common states, and small
 * helpers to declare frames, anims and moves tersely.
 */
import type { NativeAnim, NativeFrame, NativeMove, ActiveSpec, HitSpec } from '../native/types';

export type Pose = Record<string, any>;
export const merge = (...p: Pose[]): Pose => Object.assign({}, ...p);

/** pose frame */
export function fr(pose: string, ticks: number, hit?: ActiveSpec | null, o: Partial<NativeFrame> = {}): NativeFrame {
  const f: NativeFrame = { pose, ticks, ...o };
  if (hit) f.hit = hit;
  return f;
}
/** effect frame */
export function fx(effect: Record<string, any>, ticks: number, o: Partial<NativeFrame> = {}): NativeFrame { return { effect, ticks, ...o }; }
export const anim = (frames: NativeFrame[], loop?: number): NativeAnim => (loop === undefined ? { frames } : { frames, loop });

/** effect loop: n frames with advancing phase */
export function fxLoop(base: Record<string, any>, n: number, ticks: number, step = 1.1): NativeAnim {
  return anim(Array.from({ length: n }, (_, i) => fx({ ...base, phase: i * step }, ticks)));
}
/** finite hit burst */
export function burst(color: string, size = 16): NativeAnim {
  return anim([0, 0.25, 0.5, 0.75, 0.95].map((p) => fx({ type: 'burst', size, color, phase: p, nohit: true }, 3)));
}

export interface Stance {
  armF: [number, number]; armB: [number, number];
  legF: [number, number]; legB: [number, number];
  lean: number; head: number;
  handF?: string; handB?: string;
  /** absolute blade angle (sword users) in neutral */
  blade?: number;
  breath?: number;
}

const DEF: Stance = { armF: [45, 100], armB: [25, 118], legF: [20, 24], legB: [-17, 12], lean: 6, head: -4, breath: 1 };

const P = (o: Pose) => o;

export function basePoses(st0: Partial<Stance> = {}): Record<string, Pose> {
  const st = { ...DEF, ...st0 } as Stance;
  const hands: Pose = {};
  if (st.handF) hands.handF = st.handF;
  if (st.handB) hands.handB = st.handB;
  const blade = (a: number | undefined) => (st.blade === undefined ? {} : { blade: a ?? st.blade });
  const stance: Pose = {
    shoulderF: st.armF[0], elbowF: st.armF[1], shoulderB: st.armB[0], elbowB: st.armB[1],
    thighF: st.legF[0], kneeF: st.legF[1], thighB: st.legB[0], kneeB: st.legB[1], lean: st.lean, head: st.head, ...hands, ...blade(undefined),
  };
  const guardArms: Pose = { shoulderF: 78, elbowF: 128, shoulderB: 68, elbowB: 136, ...blade(st.blade === undefined ? undefined : 160) };
  const crouchLegs: Pose = { thighF: 72, kneeF: 118, thighB: 36, kneeB: 122, lean: 16, head: -12 };
  const tuck: Pose = { thighF: 78, kneeF: 120, thighB: 50, kneeB: 118, ankleF: 30, ankleB: 35 };
  const p: Record<string, Pose> = {};
  const b = st.breath ?? 1;
  [0, 1, 2, 1].forEach((k, i) => {
    p[`idle${i}`] = merge(stance, { kneeF: st.legF[1] + k * 2.5 * b, kneeB: st.legB[1] + k * 2.5 * b, thighF: st.legF[0] + k * 1.2 * b, elbowF: st.armF[1] + k * 2 * b, elbowB: st.armB[1] + k * 2 * b, shoulderF: st.armF[0] - k * 1.5 * b, phase: i * 1.6, ...(st.blade === undefined ? {} : { blade: st.blade - k * 1.5 }) });
  });
  p.turn = merge(stance, { lean: 0, thighF: 8, kneeF: 8, thighB: -8, kneeB: 8, shoulderF: 20, shoulderB: 20 });
  for (let i = 0; i < 6; i++) {
    const t = (i / 6) * Math.PI * 2, s = Math.sin(t), c = Math.cos(t);
    p[`walk${i}`] = merge(stance, {
      thighF: 10 + 20 * s, kneeF: 12 + 26 * Math.max(0, c), thighB: -4 - 20 * s, kneeB: 12 + 26 * Math.max(0, -c),
      lean: st.lean + 3, shoulderF: st.armF[0] + 4 * s, shoulderB: st.armB[0] - 4 * s, phase: i, wind: 0.5,
    });
  }
  for (let i = 0; i < 4; i++) {
    const t = (i / 4) * Math.PI * 2 + 0.5, s = Math.sin(t), c = Math.cos(t);
    p[`run${i}`] = merge(stance, {
      thighF: 22 + 42 * s, kneeF: 30 + 70 * Math.max(0, c), thighB: 22 - 42 * s, kneeB: 30 + 70 * Math.max(0, -c), ankleF: 15, ankleB: 15,
      lean: 26, head: -20, shoulderF: 45 - 45 * s, elbowF: 95, shoulderB: 45 + 45 * s, elbowB: 95, wind: 1, phase: i * 1.5,
      ...(st.blade === undefined ? {} : { blade: -60 }),
    });
  }
  p.crouch0 = merge(stance, { thighF: 50, kneeF: 70, thighB: 15, kneeB: 70, lean: 12, head: -8 });
  p.crouch = merge(stance, crouchLegs, { shoulderF: st.armF[0] + 5, shoulderB: st.armB[0] + 5 });
  p.crouch1 = merge(p.crouch, { kneeF: 121, kneeB: 125, elbowF: st.armF[1] + 4, phase: 2 });
  p.jumpStart = merge(stance, { thighF: 45, kneeF: 78, thighB: 12, kneeB: 74, lean: 18, head: -10, shoulderF: 25, elbowF: 70, shoulderB: 10, elbowB: 70 });
  p.jumpUp = merge(stance, { thighF: 55, kneeF: 95, thighB: 5, kneeB: 70, ankleF: 30, ankleB: 40, lean: 0, head: -4, shoulderF: 55, elbowF: 100, shoulderB: 40, elbowB: 110, wind: 0.2 });
  p.jumpTop = merge(stance, tuck, { lean: 4, shoulderF: 60, elbowF: 105, shoulderB: 45, elbowB: 110, wind: 0.4 });
  p.jumpDown = merge(stance, { thighF: 25, kneeF: 40, thighB: -12, kneeB: 30, ankleF: 30, ankleB: 25, lean: 4, shoulderF: 60, elbowF: 85, shoulderB: 45, elbowB: 90, wind: 0.1, hairWind: -0.6 });
  for (let i = 0; i < 4; i++) p[`flip${i}`] = merge(stance, tuck, { rot: i * 90 + 45, lean: 20, head: 20, shoulderF: 70, elbowF: 110, shoulderB: 60, elbowB: 110, free: true, wind: 0.8, phase: i });
  p.land = merge(p.jumpStart, { lean: 12 });
  p.hop = merge(stance, { lean: -12, head: 6, thighF: 40, kneeF: 50, thighB: -15, kneeB: 40, ankleF: 30, ankleB: 30, ...guardArms, wind: 0.9 });
  p.guard = merge(stance, guardArms, { lean: -2, head: 10, thighF: st.legF[0] + 2, kneeF: st.legF[1] + 8, kneeB: st.legB[1] + 8 });
  p.guardHit = merge(p.guard, { lean: -8, head: 14, thighB: st.legB[0] - 6, phase: 1 });
  p.crouchGuard = merge(crouchLegs, guardArms, { lean: 8, head: 4, ...hands });
  p.crouchGuardHit = merge(p.crouchGuard, { lean: 2, head: 10 });
  p.airGuard = merge(p.jumpTop, guardArms, { head: 10 });
  p.airGuardHit = merge(p.airGuard, { lean: -8 });
  const hurtArms: Pose = { shoulderF: 15, elbowF: 35, shoulderB: -5, elbowB: 40, handF: 'open', handB: 'open' };
  p.hitH1 = merge(stance, hurtArms, { lean: -14, head: -18, thighF: 18, kneeF: 22, thighB: -22, kneeB: 18, face: 'hurt', ...blade(120) });
  p.hitH2 = merge(stance, hurtArms, { lean: -24, head: -24, thighF: 26, kneeF: 14, thighB: -26, kneeB: 24, shoulderF: -10, elbowF: 25, shoulderB: -25, face: 'hurt', ...blade(100) });
  p.hitH3 = merge(p.hitH2, { lean: -30, head: -28, thighF: 32, phase: 2 });
  p.hitL1 = merge(stance, hurtArms, { lean: 24, head: 18, thighF: 12, kneeF: 26, thighB: -14, kneeB: 26, shoulderF: -12, elbowF: 50, shoulderB: -22, elbowB: 45, face: 'hurt', ...blade(125) });
  p.hitL2 = merge(p.hitL1, { lean: 32, head: 24, kneeF: 32 });
  p.hitL3 = merge(p.hitL2, { lean: 38, head: 26, kneeB: 36 });
  p.hitCrouch = merge(crouchLegs, hurtArms, { lean: -12, head: -22, face: 'hurt', ...blade(110) });
  p.hitAir1 = merge(hurtArms, { rot: -25, lean: -10, head: -20, thighF: 40, kneeF: 50, thighB: 12, kneeB: 45, shoulderF: -40, elbowF: 25, shoulderB: -60, elbowB: 20, face: 'hurt', wind: 0.9, ...blade(60) });
  p.hitAir2 = merge(p.hitAir1, { rot: -40, phase: 2 });
  p.fall1 = merge(p.hitAir1, { rot: -62, thighF: 45, kneeF: 35, thighB: 22, kneeB: 45, phase: 1 });
  p.fall2 = merge(p.hitAir1, { rot: -82, thighF: 38, kneeF: 30, thighB: 15, kneeB: 40, shoulderF: 150, elbowF: 30, shoulderB: 170, elbowB: 20, phase: 3 });
  p.bounce = merge({ rot: -90, thighF: 35, kneeF: 40, thighB: 15, kneeB: 50, shoulderF: 140, elbowF: 30, shoulderB: 160, elbowB: 25, head: 10, face: 'hurt', handF: 'open', handB: 'open', wind: 0, ...blade(170) });
  p.lie = merge({ rot: -90, thighF: 10, kneeF: 18, thighB: 4, kneeB: 10, ankleF: 30, ankleB: 20, shoulderF: 30, elbowF: 40, shoulderB: 165, elbowB: 25, head: -4, face: 'ko', handF: 'open', handB: 'open', wind: 0, hairWind: 0, ...blade(175) });
  p.getUp1 = merge({ lean: -30, thighF: 82, kneeF: 30, thighB: 70, kneeB: 95, shoulderF: 40, elbowF: 70, shoulderB: -40, elbowB: 5, head: 10, handB: 'open', face: 'hurt', ...blade(100) });
  p.getUp2 = merge(crouchLegs, stance, { thighF: 72, kneeF: 118, thighB: 36, kneeB: 122, lean: 30, head: -14, shoulderF: 30, elbowF: 80, shoulderB: 10, elbowB: 70 });
  p.getUp3 = merge(p.crouch0);
  p.dead = merge(p.lie, { face: 'ko' });
  p.trip = merge(hurtArms, { rot: -45, lean: -10, thighF: 75, kneeF: 15, thighB: 55, kneeB: 30, shoulderF: 110, elbowF: 20, shoulderB: 150, elbowB: 30, face: 'hurt', ...blade(150) });
  p.recover0 = merge(p.flip1, { face: 'neutral' });
  p.win0 = merge(stance, { lean: 0, head: -8, thighF: 8, kneeF: 6, thighB: -8, kneeB: 6, shoulderF: 170, elbowF: 15, shoulderB: 20, elbowB: 120, face: 'smile', ...blade(175) });
  p.win1 = merge(p.win0, { shoulderF: 172, elbowF: 8, phase: 2, wind: 0.6 });
  p.lose = merge(stance, { thighF: 80, kneeF: 88, thighB: -5, kneeB: 100, ankleB: 70, lean: 26, head: 25, shoulderF: -5, elbowF: 15, shoulderB: 0, elbowB: 10, face: 'closed', handF: 'open', handB: 'open', ...blade(135) });
  p.intro0 = merge(stance, { lean: 0, head: 0, thighF: 6, kneeF: 4, thighB: -6, kneeB: 4, shoulderF: 40, elbowF: 125, shoulderB: 45, elbowB: 130, face: 'closed', ...blade(170) });
  p.intro1 = merge(p.intro0, { face: 'neutral', head: -4 });
  return p;
}

/** the standard animation set, built from basePoses names (+ optional overrides) */
export function standardAnims(over: Partial<Record<string, NativeAnim>> = {}): Record<string, NativeAnim> {
  const A: Record<string, NativeAnim> = {
    idle: anim([fr('idle0', 10), fr('idle1', 8), fr('idle2', 10), fr('idle3', 8)]),
    turn: anim([fr('turn', 3), fr('turn', 3)]),
    walk: anim([0, 1, 2, 3, 4, 5].map((i) => fr(`walk${i}`, 6))),
    walkBack: anim([5, 4, 3, 2, 1, 0].map((i) => fr(`walk${i}`, 7))),
    crouchStart: anim([fr('crouch0', 3), fr('crouch', 2)]),
    crouch: anim([fr('crouch', 14), fr('crouch1', 14)]),
    crouchEnd: anim([fr('crouch0', 3), fr('idle0', 2)]),
    crouchTurn: anim([fr('crouch', 3), fr('crouch', 3)]),
    jumpStart: anim([fr('jumpStart', 2), fr('jumpStart', 2)]),
    jumpUp: anim([fr('jumpUp', 6), fr('jumpTop', -1)]),
    jumpFwd: anim([fr('jumpUp', 4), fr('flip0', 4), fr('flip1', 4), fr('flip2', 4), fr('flip3', 4), fr('jumpTop', -1)]),
    jumpBack: anim([fr('jumpUp', 5), fr('jumpTop', -1)]),
    jumpDown: anim([fr('jumpDown', -1)]),
    land: anim([fr('land', 2), fr('jumpStart', 2), fr('idle0', 1)]),
    run: anim([0, 1, 2, 3].map((i) => fr(`run${i}`, 4))),
    hop: anim([fr('hop', 4), fr('hop', -1)]),
    guardStart: anim([fr('guard', 2)]),
    crouchGuardStart: anim([fr('crouchGuard', 2)]),
    airGuardStart: anim([fr('airGuard', 2)]),
    guard: anim([fr('guard', -1)]),
    crouchGuard: anim([fr('crouchGuard', -1)]),
    airGuard: anim([fr('airGuard', -1)]),
    guardEnd: anim([fr('guard', 3)]),
    guardHit: anim([fr('guardHit', 3), fr('guard', -1)]),
    crouchGuardHit: anim([fr('crouchGuardHit', 3), fr('crouchGuard', -1)]),
    airGuardHit: anim([fr('airGuardHit', 3), fr('airGuard', -1)]),
    hitLight: anim([fr('hitH1', 3), fr('hitH1', 6), fr('idle0', 3)]),
    hitMedium: anim([fr('hitH2', 3), fr('hitH1', 8), fr('idle0', 3)]),
    hitHeavy: anim([fr('hitH3', 3), fr('hitH2', 10), fr('hitH1', 4)]),
    hitLowLight: anim([fr('hitL1', 3), fr('hitL1', 6), fr('idle0', 3)]),
    hitLowMedium: anim([fr('hitL2', 3), fr('hitL1', 8), fr('idle0', 3)]),
    hitLowHeavy: anim([fr('hitL3', 3), fr('hitL2', 10), fr('hitL1', 4)]),
    hitCrouch: anim([fr('hitCrouch', 3), fr('hitCrouch', 8), fr('crouch', 3)]),
    hitAir: anim([fr('hitAir1', 4), fr('hitAir2', -1)]),
    fall: anim([fr('fall1', 6), fr('fall2', -1)]),
    bounce: anim([fr('bounce', 3), fr('bounce', -1)]),
    bounceAir: anim([fr('fall2', 3), fr('bounce', -1)]),
    lieDown: anim([fr('lie', -1)]),
    getUp: anim([fr('lie', 6), fr('getUp1', 8), fr('getUp2', 7), fr('getUp3', 5), fr('idle0', 3)]),
    dead: anim([fr('dead', -1)]),
    trip: anim([fr('trip', 4), fr('fall1', -1)]),
    airRecover: anim([fr('flip0', 3), fr('flip1', 3), fr('flip2', 3), fr('flip3', 3), fr('jumpDown', -1)]),
    win: anim([fr('win0', 14), fr('win1', 14)]),
    lose: anim([fr('lose', -1)]),
    draw: anim([fr('intro1', -1)]),
    intro: anim([fr('intro0', 40), fr('intro1', 10), fr('idle0', 10)]),
  };
  for (const [k, v] of Object.entries(over)) if (v) A[k] = v;
  return A;
}

// ---------------------------------------------------------------- moves
export function hit(at: number, damage: number, level: HitSpec['level'] = 'medium', o: Partial<HitSpec> = {}): HitSpec {
  return { at, damage, level, ...o };
}
type MoveOpts = Omit<NativeMove, 'id' | 'name' | 'input' | 'kind' | 'anim'>;
export const normal = (id: string, name: string, input: string, animName: string, o: MoveOpts = {}): NativeMove => ({ id, name, input, kind: 'normal', anim: animName, ...o });
export const special = (id: string, name: string, input: string, animName: string, o: MoveOpts = {}): NativeMove => ({ id, name, input, kind: 'special', anim: animName, ...o });
export const superMove = (id: string, name: string, input: string, animName: string, o: MoveOpts = {}): NativeMove => ({ id, name, input, kind: 'super', anim: animName, cost: 1000, simple: 'super', ...o });

/** limb hit helper */
export const at = (part: string, radius = 9): ActiveSpec => ({ part, radius });

// ---------------------------------------------------------------- normal move sets
export interface NormalNames { jab: string; strong: string; lkick: string; hkick: string; cjab: string; clk: string; sweep: string; ajab: string; akick: string }
const DEF_NAMES: NormalNames = { jab: 'Jab', strong: 'Straight', lkick: 'Snap Kick', hkick: 'Roundhouse', cjab: 'Low Jab', clk: 'Shin Kick', sweep: 'Sweep', ajab: 'Air Punch', akick: 'Air Kick' };

export interface NormalSet { poses: Record<string, Pose>; anims: Record<string, NativeAnim>; moves: NativeMove[] }

/** unarmed normals (punches/kicks). `B` = basePoses() of the fighter; `o.pose` overrides individual poses */
export function unarmedNormals(B: Record<string, Pose>, o: { names?: Partial<NormalNames>; dmg?: number; speed?: number; open?: boolean; pose?: Record<string, Pose>; reach?: number } = {}): NormalSet {
  const n = { ...DEF_NAMES, ...(o.names ?? {}) };
  const d = o.dmg ?? 1, sp = o.speed ?? 1;
  const T = (t: number) => Math.max(1, Math.round(t * sp));
  const D = (x: number) => Math.round(x * d);
  const hand = o.open ? { handF: 'open' } : {};
  const poses: Record<string, Pose> = {
    jab0: merge(B.idle0, { shoulderF: 62, elbowF: 80, lean: 9 }),
    jab1: merge(B.idle0, { shoulderF: 93, elbowF: 3, lean: 14, head: -12, shoulderB: 18, elbowB: 125, thighF: 26, kneeF: 22, ...hand }),
    str0: merge(B.idle0, { lean: 2, shoulderB: 5, elbowB: 125, shoulderF: 62, elbowF: 92 }),
    str1: merge(B.idle0, { lean: 20, head: -18, shoulderB: 96, elbowB: 2, shoulderF: 12, elbowF: 132, thighF: 36, kneeF: 30, thighB: -26, kneeB: 4, face: 'shout' }),
    lk0: merge(B.idle0, { thighF: 62, kneeF: 105, lean: -2, ankleF: 30 }),
    lk1: merge(B.idle0, { thighF: 88, kneeF: 4, ankleF: 50, lean: -12, head: 6, thighB: -6, kneeB: 6, shoulderF: 50, elbowF: 110 }),
    hk0: merge(B.idle0, { thighF: 85, kneeF: 125, ankleF: 30, lean: -10, head: 4, thighB: -4, kneeB: 6 }),
    hk1: merge(B.idle0, { thighF: 128, kneeF: 4, ankleF: 45, lean: -32, head: 26, thighB: -2, kneeB: 4, shoulderF: 30, elbowF: 100, shoulderB: -20, elbowB: 60, face: 'shout', wind: 0.8 }),
    cjab0: merge(B.crouch, { shoulderF: 60, elbowF: 80 }),
    cjab1: merge(B.crouch, { shoulderF: 88, elbowF: 3, lean: 20, head: -16, ...hand }),
    clk: merge(B.crouch, { thighF: 88, kneeF: 2, ankleF: 25, lean: 4, thighB: 40, kneeB: 128 }),
    sweep0: merge(B.crouch, { lean: 32, head: -24, shoulderB: 50, elbowB: 10, handB: 'open', thighF: 70, kneeF: 125 }),
    sweep1: merge(B.crouch, { thighF: 93, kneeF: 0, ankleF: 12, thighB: 42, kneeB: 140, lean: 44, head: -34, shoulderF: 30, elbowF: 40, shoulderB: 55, elbowB: 8, handB: 'open', wind: 0.9 }),
    ajab: merge(B.jumpTop, { shoulderF: 68, elbowF: 2, lean: 14, head: -8, ...hand }),
    akick: merge(B.jumpTop, { thighF: 52, kneeF: 0, ankleF: 62, thighB: 75, kneeB: 125, lean: -16, head: 10, shoulderF: 60, elbowF: 110, wind: 0.7 }),
    ...(o.pose ?? {}),
  };
  const anims: Record<string, NativeAnim> = {
    jab: anim([fr('jab0', T(3)), fr('jab1', 3, at('handF', 9)), fr('jab0', T(4)), fr('idle0', 2)]),
    strong: anim([fr('str0', T(4)), fr('str0', T(2)), fr('str1', 4, at('handB', 10)), fr('str1', 3), fr('str0', T(7)), fr('idle0', 3)]),
    lkick: anim([fr('lk0', T(3)), fr('lk1', 4, at('footF', 9)), fr('lk0', T(5)), fr('idle0', 2)]),
    hkick: anim([fr('hk0', T(5)), fr('hk0', T(3)), fr('hk1', 5, at('footF', 11)), fr('hk1', 3), fr('hk0', T(7)), fr('idle0', 4)]),
    cjab: anim([fr('cjab0', T(2)), fr('cjab1', 3, at('handF', 9)), fr('cjab0', T(4)), fr('crouch', 2)]),
    clk: anim([fr('crouch', T(3)), fr('clk', 4, at('footF', 9)), fr('crouch', T(6))]),
    sweep: anim([fr('sweep0', T(5)), fr('sweep1', 5, at('footF', 11)), fr('sweep1', 6), fr('sweep0', T(8)), fr('crouch', 4)]),
    ajab: anim([fr('jumpTop', 2), fr('ajab', 6, at('handF', 9)), fr('ajab', 6), fr('jumpTop', -1)]),
    akick: anim([fr('jumpTop', 4), fr('akick', 10, at('footF', 11)), fr('akick', 8), fr('jumpTop', -1)]),
  };
  const moves: NativeMove[] = [
    normal('jab', n.jab, 'x', 'jab', { hits: [hit(2, D(28), 'light')] }),
    normal('strong', n.strong, 'y', 'strong', { hits: [hit(3, D(62), 'heavy')] }),
    normal('lkick', n.lkick, 'a', 'lkick', { hits: [hit(2, D(34), 'light')] }),
    normal('hkick', n.hkick, 'b', 'hkick', { hits: [hit(3, D(72), 'heavy')] }),
    normal('cjab', n.cjab, 'x', 'cjab', { when: 'crouch', hits: [hit(2, D(24), 'light')] }),
    normal('clk', n.clk, 'a', 'clk', { when: 'crouch', hits: [hit(2, D(30), 'light', { guard: 'low' })] }),
    normal('sweep', n.sweep, 'b', 'sweep', { when: 'crouch', hits: [hit(2, D(64), 'heavy', { guard: 'low', launch: true, knockback: [-2, -4] })] }),
    normal('ajab', n.ajab, 'x', 'ajab', { when: 'air', hits: [hit(2, D(34), 'light', { guard: 'high' })] }),
    normal('akick', n.akick, 'b', 'akick', { when: 'air', hits: [hit(2, D(58), 'medium', { guard: 'high' })] }),
  ];
  return { poses, anims, moves };
}

/** sword normals (x/y/b/crouch x/crouch b/air slashes use the weapon, light kicks stay) */
export function swordNormals(B: Record<string, Pose>, o: { names?: Partial<NormalNames>; dmg?: number; speed?: number; reach?: number; pose?: Record<string, Pose> } = {}): NormalSet {
  const base = unarmedNormals(B, { names: o.names, dmg: o.dmg, speed: o.speed });
  const n = { ...DEF_NAMES, ...(o.names ?? {}) };
  const d = o.dmg ?? 1, sp = o.speed ?? 1;
  const T = (t: number) => Math.max(1, Math.round(t * sp));
  const D = (x: number) => Math.round(x * d);
  const W = (r = 4) => at('weapon', r);
  const poses: Record<string, Pose> = {
    ...base.poses,
    jab0: merge(B.idle0, { shoulderF: 40, elbowF: 120, blade: 175, lean: 0 }),
    jab1: merge(B.idle0, { shoulderF: 88, elbowF: 8, blade: 96, lean: 14, head: -10, thighF: 30, kneeF: 24, wind: 0.6 }),
    str0: merge(B.idle0, { shoulderF: 168, elbowF: 30, shoulderB: 150, elbowB: 40, blade: 200, lean: -8, head: -6 }),
    str1: merge(B.idle0, { shoulderF: 75, elbowF: 4, shoulderB: 50, elbowB: 40, blade: 55, lean: 22, head: -16, thighF: 40, kneeF: 34, thighB: -28, kneeB: 4, face: 'shout', wind: 0.9 }),
    hk0: merge(B.idle0, { shoulderF: 15, elbowF: 5, blade: 25, lean: 18, head: -10, thighF: 36, kneeF: 40 }),
    hk1: merge(B.idle0, { shoulderF: 128, elbowF: 6, blade: 122, lean: -6, head: -12, thighF: 26, kneeF: 14, thighB: -22, kneeB: 8, face: 'shout', wind: 0.8 }),
    hk2: merge(B.idle0, { shoulderF: 160, elbowF: 10, blade: 168, lean: -14, head: -14, thighF: 20, kneeF: 10, thighB: -22, kneeB: 8, wind: 0.8 }),
    cjab0: merge(B.crouch, { shoulderF: 40, elbowF: 110, blade: 150 }),
    cjab1: merge(B.crouch, { shoulderF: 86, elbowF: 4, blade: 94, lean: 20, head: -14 }),
    sweep0: merge(B.crouch, { shoulderF: 120, elbowF: 40, blade: 200, lean: 10 }),
    sweep1: merge(B.crouch, { shoulderF: 70, elbowF: 2, blade: 82, lean: 34, head: -24, thighF: 80, kneeF: 110, wind: 0.9 }),
    ajab: merge(B.jumpTop, { shoulderF: 80, elbowF: 4, blade: 80, lean: 12 }),
    akick: merge(B.jumpTop, { shoulderF: 60, elbowF: 0, blade: 45, lean: 20, head: -10, wind: 0.8 }),
    akick0: merge(B.jumpTop, { shoulderF: 165, elbowF: 20, blade: 200, lean: -6 }),
    ...(o.pose ?? {}),
  };
  const anims: Record<string, NativeAnim> = {
    ...base.anims,
    jab: anim([fr('jab0', T(3)), fr('jab1', 4, W()), fr('jab0', T(5)), fr('idle0', 2)]),
    strong: anim([fr('str0', T(5)), fr('str0', T(3)), fr('str1', 4, W(5)), fr('str1', 4), fr('str0', T(6), null, { pose: merge(poses.str1, { blade: 70, lean: 12 }) }), fr('idle0', 4)]),
    hkick: anim([fr('hk0', T(5)), fr('hk0', T(2)), fr('hk1', 4, W(5)), fr('hk2', 4, W(5)), fr('hk2', 4), fr('hk0', T(8)), fr('idle0', 4)]),
    cjab: anim([fr('cjab0', T(3)), fr('cjab1', 3, W()), fr('cjab0', T(5)), fr('crouch', 2)]),
    sweep: anim([fr('sweep0', T(6)), fr('sweep1', 5, W(5)), fr('sweep1', 6), fr('sweep0', T(8)), fr('crouch', 4)]),
    ajab: anim([fr('jumpTop', 2), fr('ajab', 6, W()), fr('ajab', 6), fr('jumpTop', -1)]),
    akick: anim([fr('akick0', 5), fr('akick', 9, W(5)), fr('akick', 8), fr('jumpTop', -1)]),
  };
  const moves: NativeMove[] = [
    normal('jab', n.jab, 'x', 'jab', { hits: [hit(2, D(32), 'light')] }),
    normal('strong', n.strong, 'y', 'strong', { hits: [hit(3, D(70), 'heavy')] }),
    normal('lkick', n.lkick, 'a', 'lkick', { hits: [hit(2, D(32), 'light')] }),
    normal('hkick', n.hkick, 'b', 'hkick', { hits: [hit(3, D(68), 'heavy', { launch: true, knockback: [-2, -6] })] }),
    normal('cjab', n.cjab, 'x', 'cjab', { when: 'crouch', hits: [hit(2, D(28), 'light')] }),
    normal('clk', n.clk, 'a', 'clk', { when: 'crouch', hits: [hit(2, D(30), 'light', { guard: 'low' })] }),
    normal('sweep', n.sweep, 'b', 'sweep', { when: 'crouch', hits: [hit(2, D(66), 'heavy', { guard: 'low', launch: true, knockback: [-2, -4] })] }),
    normal('ajab', n.ajab, 'x', 'ajab', { when: 'air', hits: [hit(2, D(36), 'light', { guard: 'high' })] }),
    normal('akick', n.akick, 'b', 'akick', { when: 'air', hits: [hit(2, D(62), 'medium', { guard: 'high' })] }),
  ];
  return { poses, anims, moves };
}
