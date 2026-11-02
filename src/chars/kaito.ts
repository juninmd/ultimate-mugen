import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, unarmedNormals } from './kit';

const B = basePoses({ armF: [42, 102], armB: [22, 120], legF: [22, 26], legB: [-18, 12], lean: 6, head: -4 });
const ORB = '#ffb347';
const N = unarmedNormals(B, { names: { jab: 'Snap Jab', strong: 'Reverse Punch', lkick: 'Front Snap Kick', hkick: 'Crescent Roundhouse', cjab: 'Low Jab', clk: 'Shin Kick', sweep: 'Storm Sweep', ajab: 'Air Jab', akick: 'Diving Kick' } });

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  // ---- specials
  gale0: merge(B.idle0, { shoulderF: -28, elbowF: 105, shoulderB: -38, elbowB: 110, lean: -6, head: -2, thighF: 30, kneeF: 34, charge: 'F', chargeSize: 5, handF: 'open', handB: 'open' }),
  gale1: merge(B.idle0, { shoulderF: 92, elbowF: 4, shoulderB: 86, elbowB: 10, handF: 'open', handB: 'open', lean: 16, head: -14, thighF: 38, kneeF: 30, thighB: -30, kneeB: 4, face: 'shout', charge: 'F', chargeSize: 8 }),
  gale2: merge(B.idle0, { shoulderF: 90, elbowF: 6, shoulderB: 84, elbowB: 12, handF: 'open', handB: 'open', lean: 14, head: -12, thighF: 38, kneeF: 30, thighB: -30, kneeB: 4 }),
  rise0: merge(B.crouch, { shoulderF: 20, elbowF: 70, lean: 22, face: 'shout' }),
  rise1: merge(B.jumpUp, { thighF: 30, kneeF: 25, thighB: -14, kneeB: 70, ankleF: 45, ankleB: 40, lean: 4, head: -14, shoulderF: 172, elbowF: 6, shoulderB: 20, elbowB: 120, face: 'shout', wind: 1, hairWind: -1.2, aura: true }),
  rise2: merge(B.jumpUp, { thighF: 50, kneeF: 70, thighB: 0, kneeB: 80, ankleF: 40, lean: 8, head: -10, shoulderF: 160, elbowF: 20, shoulderB: 20, elbowB: 120, wind: 0.6, hairWind: -1 }),
  tk0: merge(B.crouch0, { lean: 18, head: -12, shoulderF: 60, elbowF: 100 }),
  tk1: merge(B.jumpTop, { thighF: 94, kneeF: 0, ankleF: 5, thighB: 62, kneeB: 132, lean: -22, head: 18, shoulderF: 40, elbowF: 110, shoulderB: -30, elbowB: 60, wind: 1, face: 'shout', y: -10 }),
  // ---- super
  sup0: merge(B.idle0, { shoulderF: -30, elbowF: 110, shoulderB: -40, elbowB: 110, lean: -8, head: -4, thighF: 34, kneeF: 40, thighB: -24, kneeB: 20, charge: 'both', chargeSize: 7, handF: 'open', handB: 'open', aura: true, face: 'shout', wind: 0.8 }),
  sup1: merge(B.idle0, { shoulderF: 92, elbowF: 2, shoulderB: 88, elbowB: 6, handF: 'open', handB: 'open', lean: 18, head: -16, thighF: 42, kneeF: 32, thighB: -34, kneeB: 2, face: 'shout', aura: true, wind: 1 }),
  bar1: merge(N.poses.jab1, { aura: true, wind: 1, face: 'shout' }),
  bar2: merge(N.poses.str1, { aura: true, wind: 1 }),
  bar3: merge(N.poses.lk1, { aura: true, wind: 1, face: 'shout' }),
  // ---- win / intro
  win0: merge(B.intro0, { shoulderF: 95, elbowF: 85, shoulderB: 20, elbowB: 130, head: -6, face: 'neutral', wind: 0.7 }),
  win1: merge(B.intro0, { shoulderF: 98, elbowF: 80, shoulderB: 20, elbowB: 130, head: -8, face: 'smile', wind: 0.9, phase: 2 }),
};

const char: NativeChar = {
  format: 'umf/1',
  id: 'kaito',
  name: 'Kaito Arashi',
  series: 'Storm Road Chronicles',
  author: 'Ultimate MUGEN',
  description: 'A wandering karateka who follows the storms, trading his teal gi for nothing but the next good fight.',
  color: '#2f8f9d',
  winQuote: 'The wind carried me here. Your stance just carried you down.',
  stats: { life: 1000, power: 3000, attack: 100, defense: 100, walk: [2.4, 2.2], run: [5.2, 4.6], jump: [-8.4, -2.5, 0, 2.6], height: 100, width: [16, 15] },
  art: {
    type: 'rig',
    style: {
      build: 1.06, top: 'gi', sleeves: 'none', pants: 'gi', belt: true, gloves: 'wrap', boots: 'barefoot',
      hair: { type: 'spiky', spikes: 7, min: 0.45, max: 0.95, sweep: 40, seed: 2 },
      headband: { color: 'top2', tail: 30 },
      colors: { skin: '#e2a97e', hair: '#3a2418', eye: '#5a3a1c', top: '#2f8f9d', top2: '#f2b233', trim: '#1d4e58', pants: '#2a2f3a', belt: '#141418', wrap: '#efe7d6' },
    },
    poses,
  },
  palettes: [
    { top: '#b8433a', trim: '#6a1e1a', top2: '#f4f0e6', pants: '#3a2a2a', hair: '#1a1a22' },
    { top: '#e8e2d2', trim: '#a8a090', top2: '#3a68d8', pants: '#e8e2d2', hair: '#c0c6d0', skin: '#f0c8a0' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    gale: anim([fr('gale0', 7), fr('gale0', 5), fr('gale1', 4), fr('gale2', 16), fr('jab0', 5), fr('idle0', 3)]),
    galeOrb: fxLoop({ type: 'orb', size: 11, color: ORB }, 4, 3),
    galeHit: burst(ORB, 18),
    rise: anim([fr('rise0', 3), fr('rise1', 4, at('handF', 13), { pose: merge(poses.rise1, { shoulderF: 128, elbowF: 16, lean: 12, head: -10 }) }), fr('rise1', 4, at('handF', 12)), fr('rise2', 10), fr('jumpDown', -1)]),
    tempest: anim([fr('tk0', 6), fr('tk1', 4, at('footF', 12)), fr('tk1', 4, at('footF', 12)), fr('tk1', 4), fr('tk0', 8), fr('idle0', 3)]),
    super: anim([fr('sup0', 12), fr('tk0', 3), fr('bar1', 3, at('handF', 12)), fr('bar2', 3, at('handB', 12)), fr('bar3', 3, at('footF', 12)), fr('bar1', 3, at('handF', 12)), fr('bar2', 3, at('handB', 12)), fr('bar3', 3, at('footF', 12)), fr('rise1', 4, at('handF', 14)), fr('rise2', 10), fr('jumpDown', -1)]),
  },
  moves: [
    ...N.moves,
    special('gale', 'Gale Fist', '~D, DF, F, x', 'gale', { simple: 'sp1', projectile: { at: 3, anim: 'galeOrb', hitAnim: 'galeHit', speed: [5.5, 0], offset: [44, -68], damage: 70, level: 'medium' } }),
    special('rise', 'Rising Tempest', 'F, D, DF, y', 'rise', { simple: 'sp2', invuln: [1, 3], moves: [{ at: 2, vel: [2.4, -8.6] }], hits: [hit(2, 70, 'heavy', { launch: true }), hit(3, 40, 'heavy', { launch: true })] }),
    special('tempest', 'Thunderstep Kick', '~D, DB, B, a', 'tempest', { simple: 'sp3', moves: [{ at: 2, vel: [7.5, 0] }, { at: 3, vel: [6.5, 0] }, { at: 4, vel: [2, 0] }], hits: [hit(2, 64, 'heavy', { knockback: [-6, 0] })] }),
    superMove('typhoon', 'Typhoon Barrage', '~D, DF, F, D, DF, F, x', 'super', {
      invuln: [1, 3],
      moves: [{ at: 2, vel: [6, 0] }, { at: 3, vel: [4, 0] }, { at: 4, vel: [4, 0] }, { at: 5, vel: [4, 0] }, { at: 6, vel: [4, 0] }, { at: 7, vel: [4, 0] }, { at: 8, vel: [4, 0] }, { at: 9, vel: [2, -8] }],
      hits: [3, 4, 5, 6, 7, 8].map((f) => hit(f, 32, 'light', { hitstun: 22, knockback: [-1.5, 0] })).concat([hit(9, 90, 'heavy', { launch: true, knockback: [-3, -8], shake: 10 })]),
    }),
  ],
};
export default char;
