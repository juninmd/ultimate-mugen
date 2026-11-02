import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, unarmedNormals } from './kit';

const B = basePoses({ armF: [80, 30], armB: [70, 105], legF: [30, 60], legB: [-10, 30], lean: 4, head: -6, handF: 'open', handB: 'open', breath: 0.8 });
const JADE = '#5aff9a';
const N = unarmedNormals(B, {
  dmg: 0.95, speed: 0.9, open: true,
  names: { jab: 'Willow Palm', strong: 'Tiger Palm', lkick: 'Sparrow Kick', hkick: 'Heaven Axe Kick', cjab: 'Root Palm', clk: 'Lotus Tap', sweep: 'Dragon Tail', ajab: 'Falling Leaf', akick: 'Swallow Kick' },
  pose: {
    lk1: merge(B.idle0, { thighF: 100, kneeF: 2, ankleF: 60, lean: -18, head: 10, thighB: -4, kneeB: 4, shoulderF: 100, elbowF: 20, shoulderB: 115, elbowB: 60 }),
    hk0: merge(B.idle0, { thighF: 150, kneeF: 10, ankleF: 30, lean: -20, head: 14, thighB: -4, kneeB: 4, shoulderF: 60, elbowF: 40, shoulderB: 115, elbowB: 60 }),
    hk1: merge(B.idle0, { thighF: 105, kneeF: 0, ankleF: 20, lean: 6, head: -12, thighB: -6, kneeB: 4, shoulderF: 40, elbowF: 50, shoulderB: 115, elbowB: 60, face: 'shout', wind: 0.8 }),
    akick: merge(B.jumpTop, { thighF: 95, kneeF: 0, ankleF: 30, thighB: 30, kneeB: 120, lean: -20, head: 10, shoulderF: 110, elbowF: 20, shoulderB: 115, elbowB: 60, wind: 0.8 }),
  },
});

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  lp0: merge(B.idle0, { shoulderF: -20, elbowF: 100, shoulderB: 115, elbowB: 60, lean: -6, charge: 'F', chargeSize: 5 }),
  lp1: merge(B.idle0, { shoulderF: 94, elbowF: 0, shoulderB: 120, elbowB: 60, lean: 16, head: -12, thighF: 42, kneeF: 36, thighB: -30, kneeB: 2, face: 'shout', charge: 'F', chargeSize: 7 }),
  ca0: merge(B.crouch, { lean: 20 }),
  ca1: merge(B.jumpUp, { thighF: 165, kneeF: 0, ankleF: 20, thighB: -10, kneeB: 40, lean: -24, head: 20, shoulderF: 60, elbowF: 30, shoulderB: -60, elbowB: 20, face: 'shout', aura: true, wind: 1 }),
  ca2: merge(B.jumpUp, { thighF: 120, kneeF: 0, ankleF: 20, thighB: 20, kneeB: 80, lean: -10, head: 10, shoulderF: 120, elbowF: 20, shoulderB: -40, elbowB: 20, aura: true, wind: 1, phase: 2 }),
  pw0: merge(B.crouch0, { lean: 20, shoulderF: 100, elbowF: 40, shoulderB: 115, elbowB: 60 }),
  pw1: merge(B.jumpTop, { thighF: 92, kneeF: 0, ankleF: 10, thighB: 50, kneeB: 130, lean: -16, head: 12, shoulderF: 170, elbowF: 10, shoulderB: -50, elbowB: 10, aura: true, face: 'shout', wind: 1 }),
  hp1: merge(B.idle0, { thighF: 110, kneeF: 0, ankleF: 30, lean: -16, head: 8, thighB: -4, kneeB: 6, shoulderF: 90, elbowF: 20, shoulderB: 115, elbowB: 60, aura: true, face: 'shout', wind: 1 }),
  hp2: merge(B.idle0, { thighF: 75, kneeF: 0, ankleF: 30, lean: -6, head: 2, thighB: -4, kneeB: 6, shoulderF: 120, elbowF: 20, shoulderB: 115, elbowB: 60, aura: true, wind: 1, phase: 2 }),
  hp3: merge(B.idle0, { thighF: 140, kneeF: 0, ankleF: 30, lean: -26, head: 14, thighB: -4, kneeB: 6, shoulderF: 70, elbowF: 20, shoulderB: 120, elbowB: 60, aura: true, wind: 1, phase: 3 }),
  win0: merge(B.intro0, { shoulderF: 85, elbowF: 95, shoulderB: 80, elbowB: 100, handB: 'fist', head: 4, face: 'closed', thighF: 90, kneeF: 120, ankleF: 40 }),
  win1: merge(B.intro0, { shoulderF: 85, elbowF: 95, shoulderB: 80, elbowB: 100, handB: 'fist', head: 2, face: 'smile', thighF: 92, kneeF: 120, ankleF: 40, phase: 2 }),
};

const char: NativeChar = {
  format: 'umf/1',
  id: 'lian',
  name: 'Lian Feng',
  series: 'Jade Lantern Tales',
  author: 'Ultimate MUGEN',
  description: 'The youngest master of the Jade Lantern school, whose kicks are said to fold the air like silk.',
  color: '#2aa86a',
  winQuote: 'Balance is everything. You lost yours three kicks ago.',
  stats: { life: 950, power: 3000, attack: 98, defense: 96, walk: [2.6, 2.4], run: [5.8, 5], jump: [-9, -2.6, 0, 2.8], height: 98, width: [15, 14] },
  art: {
    type: 'rig',
    style: {
      female: true, top: 'qipao', sleeves: 'none', pants: 'tight', belt: false, gloves: 'wrap', boots: 'shoe',
      skirt: { type: 'robe', length: 26, color: 'top', width: 0.9 },
      hair: { type: 'topknot', spikes: 6, min: 0.2, max: 0.4, bangs: 3, bangLen: 0.55, seed: 4 },
      eyes: 'cute',
      colors: { skin: '#f6d6bc', hair: '#1e1a22', eye: '#2a8a5a', top: '#1f8a5a', top2: '#f0c040', trim: '#f0c040', pants: '#f4ece0', accent: '#f0c040', boot: '#1e1a22', wrap: '#f0c040', energy: JADE, aura: JADE, lips: '#d05060' },
    },
    poses,
  },
  palettes: [
    { top: '#c8203a', trim: '#f4ece0', accent: '#f4ece0', pants: '#1e1a22', wrap: '#f4ece0', energy: '#ff8aa8', aura: '#ff8aa8', eye: '#c8203a' },
    { top: '#2a3ab8', trim: '#e0e0f0', accent: '#e0e0f0', pants: '#e0e0f0', hair: '#5a3020', wrap: '#e0e0f0', energy: '#8ab0ff', aura: '#8ab0ff', eye: '#2a3ab8' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    lotus: anim([fr('lp0', 7), fr('lp1', 4), fr('lp1', 14, null, { pose: merge(poses.lp1, { charge: false }) }), fr('jab0', 5), fr('idle0', 3)]),
    petalFx: fxLoop({ type: 'petal', size: 13, color: JADE }, 4, 3, 0.9),
    petalHit: burst(JADE, 16),
    crane: anim([fr('ca0', 3), fr('ca1', 4, at('footF', 12)), fr('ca2', 4, at('footF', 12)), fr('flip0', 4), fr('flip1', 4), fr('jumpDown', -1)]),
    phoenix: anim([fr('pw0', 5), fr('pw1', 6, at('footF', 13)), fr('pw1', -1, null, { pose: merge(poses.pw1, { aura: false, phase: 2 }) })]),
    super: anim([fr('pw0', 8, null, { pose: merge(poses.pw0, { aura: true }) }), fr('hp1', 3, at('footF', 13)), fr('hp2', 3, at('footF', 13)), fr('hp3', 3, at('footF', 13)), fr('hp1', 3, at('footF', 13), { pose: merge(poses.hp1, { phase: 4 }) }), fr('hp2', 3, at('footF', 13), { pose: merge(poses.hp2, { phase: 5 }) }), fr('hp3', 3, at('footF', 13), { pose: merge(poses.hp3, { phase: 6 }) }), fr('ca1', 5, at('footF', 14)), fr('ca2', 8), fr('jumpDown', -1)]),
  },
  moves: [
    ...N.moves,
    special('lotus', 'Lotus Palm', '~D, DF, F, x', 'lotus', { simple: 'sp1', projectile: { at: 2, anim: 'petalFx', hitAnim: 'petalHit', speed: [3.6, 0], offset: [40, -66], damage: 64, removeTime: 26, knockback: [-5, 0] } }),
    special('crane', 'Crane Ascension', 'F, D, DF, y', 'crane', { simple: 'sp2', invuln: [1, 3], moves: [{ at: 2, vel: [1.8, -9] }], hits: [hit(2, 64, 'heavy', { launch: true }), hit(3, 46, 'heavy', { launch: true })] }),
    special('phoenix', 'Phoenix Wing Kick', '~D, DB, B, a', 'phoenix', { simple: 'sp3', fx: 'aura', moves: [{ at: 2, vel: [6.2, -4.2] }], hits: [hit(2, 78, 'heavy', { knockback: [-6, -3], launch: true })] }),
    superMove('petals', 'Hundred Petal Storm', '~D, DF, F, D, DF, F, x', 'super', {
      invuln: [1, 2],
      moves: [{ at: 2, vel: [3, 0] }, { at: 4, vel: [3, 0] }, { at: 6, vel: [3, 0] }, { at: 8, vel: [2, -8] }],
      hits: [2, 3, 4, 5, 6, 7].map((f) => hit(f, 30, 'light', { hitstun: 22, knockback: [-1.2, 0] })).concat([hit(8, 96, 'heavy', { launch: true, knockback: [-3, -8], shake: 10 })]),
    }),
  ],
};
export default char;
