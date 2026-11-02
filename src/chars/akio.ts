import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, unarmedNormals } from './kit';

const B = basePoses({ armF: [55, 95], armB: [35, 110], legF: [18, 22], legB: [-20, 14], lean: 4, head: -2 });
const KI = '#4fe3ff';
const N = unarmedNormals(B, {
  dmg: 1.0,
  names: { jab: 'Quick Strike', strong: 'Star Hammer', lkick: 'Low Snap', hkick: 'Spinning Hook Kick', cjab: 'Crouch Jab', clk: 'Toe Kick', sweep: 'Orbit Sweep', ajab: 'Falling Fist', akick: 'Comet Kick' },
  pose: { hk1: merge(B.idle0, { thighF: 135, kneeF: 15, ankleF: 30, lean: -36, head: 28, thighB: -4, kneeB: 6, shoulderF: 20, elbowF: 90, shoulderB: -30, elbowB: 60, face: 'shout', wind: 0.9 }) },
});

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  sb0: merge(B.idle0, { shoulderF: -25, elbowF: 110, shoulderB: -35, elbowB: 110, handF: 'open', handB: 'open', lean: -8, thighF: 30, kneeF: 36, charge: 'both', chargeSize: 6, face: 'shout' }),
  sb1: merge(B.idle0, { shoulderF: 94, elbowF: 2, shoulderB: 90, elbowB: 8, handF: 'open', handB: 'open', lean: 16, head: -14, thighF: 40, kneeF: 30, thighB: -32, kneeB: 2, face: 'shout', charge: 'F', chargeSize: 9, wind: 0.8 }),
  sb2: merge(B.idle0, { shoulderF: 90, elbowF: 6, shoulderB: 86, elbowB: 12, handF: 'open', handB: 'open', lean: 12, head: -10, thighF: 40, kneeF: 30, thighB: -32, kneeB: 2 }),
  cu0: merge(B.crouch, { shoulderF: 15, elbowF: 80, lean: 24, charge: 'F', chargeSize: 5 }),
  cu1: merge(B.jumpUp, { thighF: 25, kneeF: 15, thighB: -15, kneeB: 80, ankleF: 50, ankleB: 40, lean: 6, head: -16, shoulderF: 168, elbowF: 6, shoulderB: 25, elbowB: 120, face: 'shout', wind: 1, hairWind: -1.3, aura: true, charge: 'F', chargeSize: 7 }),
  cu2: merge(B.jumpUp, { thighF: 55, kneeF: 80, thighB: 5, kneeB: 90, ankleF: 40, lean: 10, head: -10, shoulderF: 150, elbowF: 25, shoulderB: 20, elbowB: 120, wind: 0.6 }),
  mh0: merge(B.crouch0, { lean: 14, head: -10, shoulderF: 50, elbowF: 100 }),
  mh1: merge(B.jumpUp, { thighF: 165, kneeF: 2, ankleF: 20, thighB: -10, kneeB: 40, lean: -20, head: 12, shoulderF: 30, elbowF: 90, shoulderB: -20, elbowB: 70, wind: 0.8, face: 'shout' }),
  mh2: merge(B.jumpDown, { thighF: 70, kneeF: 2, ankleF: 30, thighB: -20, kneeB: 50, lean: 22, head: -14, shoulderF: 40, elbowF: 90, shoulderB: -40, elbowB: 50, aura: true, wind: 0.4, hairWind: -0.8, face: 'shout' }),
  nc0: merge(B.idle0, { shoulderF: -35, elbowF: 115, shoulderB: -45, elbowB: 115, handF: 'open', handB: 'open', lean: -10, head: -8, thighF: 36, kneeF: 44, thighB: -26, kneeB: 20, charge: 'both', chargeSize: 9, aura: true, face: 'shout', wind: 1 }),
  nc1: merge(B.idle0, { shoulderF: 94, elbowF: 0, shoulderB: 90, elbowB: 4, handF: 'open', handB: 'open', lean: 20, head: -18, thighF: 46, kneeF: 34, thighB: -36, kneeB: 2, aura: true, face: 'shout', wind: 1 }),
  win0: merge(B.intro0, { shoulderF: 172, elbowF: 6, shoulderB: 15, elbowB: 100, head: -12, face: 'smile', aura: true, wind: 0.8 }),
  win1: merge(B.intro0, { shoulderF: 174, elbowF: 4, shoulderB: 15, elbowB: 100, head: -12, face: 'smile', aura: true, wind: 1, phase: 2 }),
};

const char: NativeChar = {
  format: 'umf/1',
  id: 'akio',
  name: 'Akio Hoshi',
  series: 'Starfall Dojo',
  author: 'Ultimate MUGEN',
  description: 'A hot-blooded prodigy whose crimson hair stands on end whenever his inner starlight boils over.',
  color: '#e0303a',
  winQuote: 'That was a warm-up! Come on, get up and show me your real power!',
  stats: { life: 980, power: 3000, attack: 102, defense: 98, walk: [2.6, 2.3], run: [5.6, 4.8], jump: [-8.8, -2.6, 0, 2.8], height: 100, width: [16, 15] },
  art: {
    type: 'rig',
    style: {
      build: 1.04, top: 'tank', sleeves: 'none', pants: 'baggy', belt: 'top2', beltTails: 0, gloves: 'glove', boots: 'boot',
      hair: { type: 'spiky', spikes: 9, min: 0.8, max: 1.65, sweep: 30, bangs: 3, seed: 11 },
      eyes: 'fierce',
      colors: { skin: '#f0c49c', hair: '#e0303a', eye: '#2a9ad0', top: '#1c1c26', top2: '#e0303a', trim: '#e0303a', pants: '#5a5e6e', glove: '#e0303a', boot: '#2a2a36', energy: KI, aura: KI },
    },
    poses,
  },
  palettes: [
    { hair: '#f0d040', top: '#f4f2ea', top2: '#2a5ad0', pants: '#2a5ad0', glove: '#2a5ad0', energy: '#ffe14f', aura: '#ffe14f' },
    { hair: '#3a3a48', top: '#5a2a8a', top2: '#f0f0f0', pants: '#24242e', glove: '#f0f0f0', energy: '#c070ff', aura: '#c070ff' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    starburst: anim([fr('sb0', 8), fr('sb0', 5, null, { pose: merge(poses.sb0, { phase: 2 }) }), fr('sb1', 4), fr('sb2', 15), fr('jab0', 5), fr('idle0', 3)]),
    starOrb: fxLoop({ type: 'orb', size: 12, color: KI }, 4, 3),
    starHit: burst(KI, 18),
    comet: anim([fr('cu0', 3), fr('cu1', 4, at('handF', 13), { pose: merge(poses.cu1, { shoulderF: 128, elbowF: 16, lean: 12, head: -10 }) }), fr('cu1', 4, at('handF', 12), { pose: merge(poses.cu1, { phase: 2 }) }), fr('cu2', 10), fr('jumpDown', -1)]),
    meteor: anim([fr('mh0', 5), fr('mh1', 8), fr('mh2', 6, at('footF', 13)), fr('mh2', -1, null, { pose: merge(poses.mh2, { aura: false }) })]),
    super: anim([fr('nc0', 12), fr('nc0', 12, null, { pose: merge(poses.nc0, { phase: 2 }) }), fr('nc1', 4), fr('nc1', 44, null, { pose: merge(poses.nc1, { phase: 3 }) }), fr('sb2', 8), fr('idle0', 4)]),
    novaBeam: fxLoop({ type: 'beam', size: 15, length: 240, color: KI }, 4, 3, 1.3),
    novaHit: burst(KI, 24),
  },
  moves: [
    ...N.moves,
    special('starburst', 'Star Burst', '~D, DF, F, x', 'starburst', { simple: 'sp1', projectile: { at: 3, anim: 'starOrb', hitAnim: 'starHit', speed: [6, 0], offset: [46, -68], damage: 72 } }),
    special('comet', 'Comet Uppercut', 'F, D, DF, y', 'comet', { simple: 'sp2', invuln: [1, 3], moves: [{ at: 2, vel: [2.2, -9] }], hits: [hit(2, 76, 'heavy', { launch: true }), hit(3, 44, 'heavy', { launch: true })] }),
    special('meteor', 'Meteor Heel', '~D, DB, B, a', 'meteor', { simple: 'sp3', moves: [{ at: 2, vel: [3.6, -6.4] }], hits: [hit(3, 84, 'heavy', { guard: 'high', shake: 8 })] }),
    superMove('nova', 'Nova Cannon', '~D, DF, F, D, DF, F, x', 'super', { invuln: [1, 3], projectile: { at: 3, anim: 'novaBeam', hitAnim: 'novaHit', speed: [0, 0], offset: [44, -68], damage: 48, hits: 7, removeTime: 44, level: 'heavy', launch: true, priority: 6 } }),
  ],
};
export default char;
