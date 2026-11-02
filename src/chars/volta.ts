import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, unarmedNormals } from './kit';

const B = basePoses({ armF: [50, 90], armB: [30, 105], legF: [16, 20], legB: [-16, 12], lean: 6, head: -4, breath: 0.6 });
const VOLT = '#ffe14a';
const N = unarmedNormals(B, {
  dmg: 1.02,
  names: { jab: 'Piston Jab', strong: 'Rail Cross', lkick: 'Servo Kick', hkick: 'Turbine Kick', cjab: 'Low Piston', clk: 'Hydraulic Tap', sweep: 'Gyro Sweep', ajab: 'Drop Hammer', akick: 'Thruster Kick' },
});

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  ab0: merge(B.idle0, { shoulderF: 30, elbowF: 120, shoulderB: 20, elbowB: 110, lean: -4, charge: 'F', chargeSize: 6, wind: 0.4 }),
  ab1: merge(B.idle0, { shoulderF: 95, elbowF: 0, shoulderB: 10, elbowB: 120, handF: 'open', lean: 14, head: -10, thighF: 34, kneeF: 28, thighB: -26, kneeB: 4, charge: 'F', chargeSize: 9 }),
  ab2: merge(B.idle0, { shoulderF: 92, elbowF: 4, shoulderB: 10, elbowB: 120, handF: 'open', lean: 12, head: -8, thighF: 34, kneeF: 28, thighB: -26, kneeB: 4 }),
  tu0: merge(B.crouch, { shoulderF: 10, elbowF: 70, lean: 26, charge: 'F', chargeSize: 6 }),
  tu1: merge(B.jumpUp, { thighF: 28, kneeF: 20, thighB: -14, kneeB: 70, ankleF: 50, lean: 2, head: -14, shoulderF: 170, elbowF: 4, shoulderB: 20, elbowB: 120, aura: true, charge: 'F', chargeSize: 9, wind: 1 }),
  tu2: merge(B.jumpUp, { thighF: 50, kneeF: 80, thighB: 0, kneeB: 90, lean: 8, shoulderF: 150, elbowF: 25, shoulderB: 20, elbowB: 120 }),
  od0: merge(B.crouch0, { lean: 26, head: -16, shoulderF: 40, elbowF: 130, shoulderB: 20, elbowB: 120 }),
  od1: merge(B.run1, { lean: 38, head: -28, shoulderF: 60, elbowF: 140, shoulderB: 30, elbowB: 130, thighF: 60, kneeF: 50, thighB: -30, kneeB: 10, aura: true, wind: 1 }),
  tc0: merge(B.idle0, { shoulderF: 175, elbowF: 10, shoulderB: 165, elbowB: 15, lean: -8, head: -14, charge: 'both', chargeSize: 10, aura: true, wind: 1 }),
  tc1: merge(B.idle0, { shoulderF: 96, elbowF: 0, shoulderB: 92, elbowB: 4, handF: 'open', handB: 'open', lean: 18, head: -12, thighF: 40, kneeF: 30, thighB: -32, kneeB: 2, aura: true, wind: 1 }),
  win0: merge(B.intro0, { shoulderF: 120, elbowF: 140, shoulderB: 20, elbowB: 110, head: 4, face: 'neutral', charge: 'F', chargeSize: 4 }),
  win1: merge(B.intro0, { shoulderF: 120, elbowF: 140, shoulderB: 20, elbowB: 110, head: 2, face: 'neutral', charge: 'F', chargeSize: 6, phase: 2 }),
};

const char: NativeChar = {
  format: 'umf/1',
  id: 'volta',
  name: 'VX-9 Volta',
  series: 'Neon Grid Protocol',
  author: 'Ultimate MUGEN',
  description: 'A prototype combat android that escaped its lab and now fights to discover what its own heart is for.',
  color: '#2ac8e8',
  winQuote: 'Combat log saved. Conclusion: you require a firmware update.',
  stats: { life: 1000, power: 3000, attack: 100, defense: 104, walk: [2.3, 2.1], run: [5.6, 4.8], jump: [-8.2, -2.4, 0, 2.5], height: 104, width: [16, 15] },
  art: {
    type: 'rig',
    style: {
      scale: 1.03, build: 1.08, top: 'suit', sleeves: 'long', pants: 'tight', belt: 'metal', beltTails: 0, gloves: 'gauntlet', boots: 'greave', shoulderPads: true,
      hair: { type: 'helmet' },
      colors: { skin: '#c8d0dc', hair: '#e8ecf4', eye: '#59f0ff', top: '#2a2e3a', top2: '#59f0ff', trim: '#59f0ff', pants: '#2a2e3a', glove: '#e8ecf4', boot: '#e8ecf4', metal: '#b8c2d2', accent: '#ffcc22', visor: '#59f0ff', energy: VOLT, aura: '#59f0ff' },
    },
    poses,
  },
  palettes: [
    { hair: '#2a2a32', top: '#e8e8f0', pants: '#e8e8f0', glove: '#2a2a32', boot: '#2a2a32', metal: '#4a4a58', visor: '#ff3a4a', accent: '#ff3a4a', energy: '#ff6a4a', aura: '#ff3a4a' },
    { hair: '#f0c030', top: '#3a2a4a', pants: '#3a2a4a', glove: '#f0c030', boot: '#f0c030', metal: '#d8b860', visor: '#7aff6a', accent: '#7aff6a', energy: '#9aff6a', aura: '#7aff6a' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    arcbolt: anim([fr('ab0', 8), fr('ab0', 4, null, { pose: merge(poses.ab0, { phase: 2 }) }), fr('ab1', 4), fr('ab2', 14), fr('idle0', 5)]),
    boltFx: fxLoop({ type: 'lightning', size: 12, color: VOLT }, 4, 2, 0.7),
    boltHit: burst(VOLT, 18),
    tesla: anim([fr('tu0', 3), fr('tu1', 4, at('handF', 13), { pose: merge(poses.tu1, { shoulderF: 128, elbowF: 16, lean: 12, head: -10 }) }), fr('tu1', 4, at('handF', 12), { pose: merge(poses.tu1, { phase: 2 }) }), fr('tu2', 10), fr('jumpDown', -1)]),
    overclock: anim([fr('od0', 6), fr('od1', 4, at('shoulder', 14)), fr('od1', 4, at('shoulder', 14), { pose: merge(poses.od1, { phase: 1 }) }), fr('od1', 4, null, { pose: merge(poses.od1, { phase: 2 }) }), fr('od0', 8), fr('idle0', 3)]),
    super: anim([fr('tc0', 12), fr('tc0', 12, null, { pose: merge(poses.tc0, { phase: 2 }) }), fr('tc1', 4), fr('tc1', 20, null, { pose: merge(poses.tc1, { phase: 3 }) }), fr('ab2', 8), fr('idle0', 4)]),
    coreFx: fxLoop({ type: 'lightning', size: 30, color: VOLT }, 6, 2, 0.6),
    coreHit: burst(VOLT, 34),
  },
  moves: [
    ...N.moves,
    special('arcbolt', 'Arc Bolt', '~D, DF, F, x', 'arcbolt', { simple: 'sp1', projectile: { at: 3, anim: 'boltFx', hitAnim: 'boltHit', speed: [6.6, 0], offset: [46, -68], damage: 66, hits: 2 } }),
    special('tesla', 'Tesla Uppercut', 'F, D, DF, y', 'tesla', { simple: 'sp2', invuln: [1, 3], moves: [{ at: 2, vel: [2, -8.6] }], hits: [hit(2, 72, 'heavy', { launch: true }), hit(3, 42, 'heavy', { launch: true })] }),
    special('overclock', 'Overclock Dash', '~D, DB, B, a', 'overclock', { simple: 'sp3', fx: 'aura', moves: [{ at: 2, vel: [8.5, 0] }, { at: 3, vel: [7.5, 0] }, { at: 4, vel: [3, 0] }], hits: [hit(2, 46, 'medium'), hit(3, 40, 'heavy', { knockback: [-6, -3], launch: true })] }),
    superMove('core', 'Thunder Core Overload', '~D, DF, F, D, DF, F, x', 'super', { invuln: [1, 3], projectile: { at: 3, anim: 'coreFx', hitAnim: 'coreHit', speed: [2.6, 0], offset: [50, -60], damage: 44, hits: 7, level: 'heavy', launch: true, priority: 7, removeTime: 120 } }),
  ],
};
export default char;
