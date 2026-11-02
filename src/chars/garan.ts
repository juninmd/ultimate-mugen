import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, unarmedNormals } from './kit';

const B = basePoses({ armF: [35, 70], armB: [20, 80], legF: [22, 30], legB: [-18, 18], lean: 14, head: -12, handF: 'open', handB: 'open', breath: 1.2 });
const HELL = '#b84aff';
const N = unarmedNormals(B, {
  dmg: 1.12, speed: 1.12, open: true,
  names: { jab: 'Claw Jab', strong: 'Demon Rake', lkick: 'Hoof Kick', hkick: 'Cinder Roundhouse', cjab: 'Low Claw', clk: 'Ash Kick', sweep: 'Grave Sweep', ajab: 'Talon Dive', akick: 'Ruin Kick' },
});

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  ho0: merge(B.idle0, { shoulderF: -30, elbowF: 100, shoulderB: 150, elbowB: 30, lean: -6, head: -8, charge: 'F', chargeSize: 7, face: 'shout' }),
  ho1: merge(B.idle0, { shoulderF: 96, elbowF: 0, shoulderB: -20, elbowB: 40, lean: 18, head: -14, thighF: 40, kneeF: 34, thighB: -30, kneeB: 4, face: 'shout', charge: 'F', chargeSize: 9 }),
  ho2: merge(B.idle0, { shoulderF: 92, elbowF: 4, shoulderB: -20, elbowB: 40, lean: 14, head: -10, thighF: 40, kneeF: 34, thighB: -30, kneeB: 4 }),
  da0: merge(B.crouch, { shoulderF: 0, elbowF: 30, lean: 30, face: 'shout', aura: true }),
  da1: merge(B.jumpUp, { thighF: 40, kneeF: 30, thighB: -14, kneeB: 70, ankleF: 45, lean: 4, head: -18, shoulderF: 165, elbowF: 10, shoulderB: 30, elbowB: 100, face: 'shout', aura: true, wind: 1, hairWind: -1.2 }),
  da2: merge(B.jumpUp, { thighF: 55, kneeF: 80, thighB: 0, kneeB: 90, lean: 8, shoulderF: 150, elbowF: 30, shoulderB: 20, elbowB: 110, wind: 0.6 }),
  bd0: merge(B.crouch0, { lean: 20, shoulderF: 160, elbowF: 50, shoulderB: 150, elbowB: 60 }),
  bd1: merge(B.jumpUp, { shoulderF: 175, elbowF: 20, shoulderB: 170, elbowB: 25, lean: -10, head: -6, aura: true, wind: 0.6 }),
  bd2: merge(B.jumpDown, { shoulderF: 70, elbowF: 0, shoulderB: 50, elbowB: 10, lean: 30, head: -20, aura: true, face: 'shout', hairWind: -1 }),
  ig0: merge(B.idle0, { shoulderF: 170, elbowF: 20, shoulderB: 165, elbowB: 25, lean: -12, head: -20, thighF: 36, kneeF: 44, thighB: -26, kneeB: 20, aura: true, face: 'shout', wind: 1 }),
  ig1: merge(B.crouch, { shoulderF: 70, elbowF: 0, shoulderB: 60, elbowB: 10, lean: 40, head: -26, aura: true, face: 'shout', wind: 1 }),
  win0: merge(B.intro0, { shoulderF: 100, elbowF: 120, shoulderB: 20, elbowB: 70, head: -16, lean: -6, face: 'shout', charge: 'F', chargeSize: 8, wind: 0.6 }),
  win1: merge(B.intro0, { shoulderF: 100, elbowF: 125, shoulderB: 20, elbowB: 70, head: -18, lean: -6, face: 'smile', charge: 'F', chargeSize: 9, wind: 0.9, phase: 2 }),
};

const PILLAR = { type: 'pillar', size: 16, length: 120, color: HELL };
const char: NativeChar = {
  format: 'umf/1',
  id: 'garan',
  name: 'Garan the Ashen Oni',
  series: 'Ashen Gate Legends',
  author: 'Ultimate MUGEN',
  description: 'A thousand-year oni who crawled out of a sealed shrine gate, wreathed in flames that burn violet and cold.',
  color: '#8a2ad8',
  winQuote: 'Kneel, little mortal. The gate is open, and it is hungry.',
  stats: { life: 1080, power: 3000, attack: 108, defense: 100, walk: [2.0, 1.9], run: [4.8, 4.2], jump: [-7.9, -2.3, 0, 2.4], height: 112, width: [18, 17] },
  art: {
    type: 'rig',
    style: {
      scale: 1.08, build: 1.3, torsoShape: 'heavy', top: 'bare', pants: 'hakama', belt: 'accent', beltTails: 1, gloves: 'gauntlet', boots: 'wrap', shoulderPads: true,
      hair: { type: 'mane', spikes: 9, min: 0.7, max: 1.4, sweep: 50, bangs: 2, seed: 13 },
      horns: { size: 1.1 }, eyes: 'glow',
      colors: { skin: '#a8384a', hair: '#efe8f4', eye: '#ffd23a', pants: '#2a1838', accent: '#d8a830', glove: '#2a2430', metal: '#4a4058', wrap: '#5a4a5a', horn: '#f0e6cc', energy: HELL, aura: HELL },
    },
    poses,
  },
  palettes: [
    { skin: '#4a6ab8', hair: '#1a1a24', pants: '#e8e0d0', accent: '#c02a2a', eye: '#ff4a2a', energy: '#ff6a2a', aura: '#ff6a2a', metal: '#6a6a78' },
    { skin: '#5a8a4a', hair: '#f0d050', pants: '#1a2a1a', accent: '#e0e0e0', eye: '#ff2a8a', energy: '#5aff8a', aura: '#5aff8a', metal: '#3a4a3a' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    hellorb: anim([fr('ho0', 9), fr('ho0', 5, null, { pose: merge(poses.ho0, { phase: 2 }) }), fr('ho1', 4), fr('ho2', 15), fr('idle0', 5)]),
    hellFx: fxLoop({ type: 'darkfire', size: 14, color: HELL }, 4, 3, 0.9),
    hellHit: burst(HELL, 20),
    ascent: anim([fr('da0', 4), fr('da1', 4, at('handF', 14), { pose: merge(poses.da1, { shoulderF: 128, elbowF: 16, lean: 12, head: -10 }) }), fr('da1', 4, at('handF', 13), { pose: merge(poses.da1, { phase: 2 }) }), fr('da2', 10), fr('jumpDown', -1)]),
    brimstone: anim([fr('bd0', 6), fr('bd1', 10), fr('bd2', 6, at('handF', 14)), fr('bd2', -1, null, { pose: merge(poses.bd2, { aura: false }) })]),
    super: anim([fr('ig0', 14), fr('ig0', 10, null, { pose: merge(poses.ig0, { phase: 2 }) }), fr('ig1', 6), fr('ig1', 6, null, { pose: merge(poses.ig1, { phase: 2 }) }), fr('ig1', 6, null, { pose: merge(poses.ig1, { phase: 3 }) }), fr('ig1', 20, null, { pose: merge(poses.ig1, { phase: 4 }) }), fr('idle0', 5)]),
    pillarFx: fxLoop(PILLAR, 4, 3, 1.1),
  },
  moves: [
    ...N.moves,
    special('hellorb', 'Hellflame Orb', '~D, DF, F, x', 'hellorb', { simple: 'sp1', projectile: { at: 3, anim: 'hellFx', hitAnim: 'hellHit', speed: [4.6, 0], offset: [48, -66], damage: 80 } }),
    special('ascent', 'Demon Ascent', 'F, D, DF, y', 'ascent', { simple: 'sp2', invuln: [1, 3], moves: [{ at: 2, vel: [2, -8] }], hits: [hit(2, 80, 'heavy', { launch: true }), hit(3, 46, 'heavy', { launch: true })] }),
    special('brimstone', 'Brimstone Drop', '~D, DB, B, a', 'brimstone', { simple: 'sp3', moves: [{ at: 2, vel: [3.8, -6.6] }], hits: [hit(3, 92, 'heavy', { guard: 'high', shake: 10, knockback: [-4, -4], launch: true })] }),
    superMove('gate', 'Inferno Gate', '~D, DF, F, D, DF, F, x', 'super', {
      invuln: [1, 4],
      projectile: [3, 4, 5].map((f, i) => ({ at: f, anim: 'pillarFx', hitAnim: 'hellHit', speed: [0, 0] as [number, number], offset: [46 + i * 46, 0] as [number, number], damage: 50, hits: 3, removeTime: 30, level: 'heavy' as const, launch: true, knockback: [-2, -8] as [number, number], priority: 6 })),
    }),
  ],
};
export default char;
