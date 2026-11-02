import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, swordNormals } from './kit';

const B = basePoses({ armF: [52, 42], armB: [38, 95], legF: [24, 28], legB: [-20, 14], lean: 8, head: -6, blade: 128 });
const RED = '#ff3a5c';
const N = swordNormals(B, {
  speed: 1.1, dmg: 1.08,
  names: { jab: 'Hilt Cut', strong: 'Moonfall Cleave', lkick: 'Push Kick', hkick: 'Rising Edge', cjab: 'Low Slice', clk: 'Ankle Kick', sweep: 'Reaper Sweep', ajab: 'Sky Slice', akick: 'Eclipse Drop' },
});

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  // specials
  cr0: merge(B.idle0, { shoulderF: 172, elbowF: 25, shoulderB: 150, elbowB: 45, blade: 205, lean: -10, head: -6, glow: true, wind: 0.6 }),
  cr1: merge(B.idle0, { shoulderF: 72, elbowF: 2, shoulderB: 45, elbowB: 40, blade: 62, lean: 24, head: -16, thighF: 44, kneeF: 36, thighB: -30, kneeB: 4, face: 'shout', glow: true, wind: 1 }),
  cr2: merge(B.idle0, { shoulderF: 60, elbowF: 4, shoulderB: 40, elbowB: 40, blade: 40, lean: 20, head: -12, thighF: 44, kneeF: 36, thighB: -30, kneeB: 4 }),
  mr0: merge(B.crouch, { shoulderF: 10, elbowF: 10, blade: 20, lean: 26, head: -16, glow: true }),
  mr1: merge(B.jumpUp, { thighF: 35, kneeF: 30, thighB: -10, kneeB: 70, ankleF: 45, shoulderF: 165, elbowF: 8, shoulderB: 120, elbowB: 40, blade: 172, lean: -6, head: -16, face: 'shout', glow: true, aura: true, wind: 1 }),
  mr2: merge(B.jumpTop, { shoulderF: 150, elbowF: 20, blade: 150, lean: 0, glow: true }),
  ss0: merge(B.crouch0, { shoulderF: -20, elbowF: 20, shoulderB: 10, elbowB: 60, blade: -75, lean: 26, head: -16, wind: 0.6 }),
  ss1: merge(B.run1, { shoulderF: 92, elbowF: 2, shoulderB: -30, elbowB: 30, blade: 92, lean: 30, head: -22, thighF: 70, kneeF: 50, thighB: -30, kneeB: 20, face: 'shout', glow: true, aura: true, wind: 1 }),
  ss2: merge(B.idle0, { shoulderF: 110, elbowF: 5, blade: 125, lean: 6, thighF: 40, kneeF: 40, wind: 0.8 }),
  sp0: merge(B.idle0, { shoulderF: 178, elbowF: 5, shoulderB: 165, elbowB: 25, blade: 180, lean: -6, head: -14, thighF: 34, kneeF: 40, thighB: -26, kneeB: 18, aura: true, glow: true, face: 'shout', wind: 1 }),
  sp1: merge(B.idle0, { shoulderF: 60, elbowF: 0, shoulderB: 45, elbowB: 30, blade: 45, lean: 30, head: -20, thighF: 50, kneeF: 50, thighB: -34, kneeB: 4, aura: true, glow: true, face: 'shout', wind: 1 }),
  win0: merge(B.intro0, { shoulderF: 30, elbowF: 40, blade: 200, head: -10, face: 'neutral', wind: 0.6 }),
  win1: merge(B.intro0, { shoulderF: 32, elbowF: 38, blade: 202, head: -12, face: 'neutral', wind: 0.9, phase: 2 }),
  intro0: merge(B.intro0, { shoulderF: 30, elbowF: 40, blade: 200, face: 'closed', wind: 0.8 }),
  intro1: merge(B.intro0, { shoulderF: 60, elbowF: 40, blade: 140, face: 'neutral', wind: 0.8 }),
};

const char: NativeChar = {
  format: 'umf/1',
  id: 'shion',
  name: 'Shion Kurogane',
  series: 'Requiem of the Pale Moon',
  author: 'Ultimate MUGEN',
  description: 'A silent exile in a black battle-robe who swings a moon-forged greatsword heavier than any man should lift.',
  color: '#c8203c',
  winQuote: 'The moon does not hurry. Neither does my blade.',
  stats: { life: 1000, power: 3000, attack: 106, defense: 96, walk: [2.2, 2.0], run: [5, 4.4], jump: [-8.2, -2.4, 0, 2.5], height: 104, width: [17, 15] },
  art: {
    type: 'rig',
    style: {
      build: 1.0, top: 'robe', sleeves: 'wide', pants: 'hakama', belt: 'cloth', beltTails: 1, gloves: 'glove', boots: 'tall',
      skirt: { type: 'robe', length: 38, color: 'top', width: 1.05 },
      hair: { type: 'long', length: 34, spikes: 7, min: 0.35, max: 0.8, sweep: 50, bangs: 3, seed: 5 },
      eyes: 'fierce',
      weapon: { type: 'greatsword', length: 68, width: 12, glowColor: 'energy', twoHand: true },
      colors: { skin: '#f0d0b8', hair: '#d8dce8', eye: '#d0203c', top: '#1b1a22', top2: '#c8203c', trim: '#c8203c', pants: '#3a3846', cloth: '#c8203c', glove: '#4a1420', boot: '#141218', weapon: '#3a3e4c', weapon2: '#e8eef8', hilt: '#c8203c', accent: '#b8bcc8', energy: RED, aura: RED, undershirt: '#2a2832' },
      undershirt: 'undershirt',
    },
    poses,
  },
  palettes: [
    { top: '#e8e4ec', pants: '#c8c4cc', trim: '#3048b0', cloth: '#3048b0', hair: '#1a1820', eye: '#3048b0', energy: '#5aa0ff', aura: '#5aa0ff', weapon: '#c8ccd8', hilt: '#3048b0', undershirt: '#c8c4cc' },
    { top: '#2a1838', pants: '#1a1024', trim: '#e0b030', cloth: '#e0b030', hair: '#f0e0c0', eye: '#e0b030', energy: '#ffcc33', aura: '#ffcc33', hilt: '#e0b030', undershirt: '#3a2448' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    crescent: anim([fr('cr0', 8), fr('cr0', 4, null, { pose: merge(poses.cr0, { phase: 2 }) }), fr('cr1', 4, at('weapon', 5)), fr('cr2', 14), fr('idle0', 5)]),
    crescentFx: fxLoop({ type: 'slash', size: 26, color: RED }, 4, 3, 0.9),
    crescentHit: burst(RED, 20),
    moonrise: anim([fr('mr0', 4), fr('mr1', 4, at('weapon', 6), { pose: merge(poses.mr1, { shoulderF: 118, elbowF: 4, blade: 105, lean: 10 }) }), fr('mr1', 4, at('weapon', 6), { pose: merge(poses.mr1, { phase: 2 }) }), fr('mr2', 10), fr('jumpDown', -1)]),
    shadowstep: anim([fr('ss0', 7), fr('ss1', 4, at('weapon', 6)), fr('ss1', 5, null, { pose: merge(poses.ss1, { phase: 2 }) }), fr('ss2', 10), fr('idle0', 4)]),
    super: anim([fr('sp0', 14), fr('sp0', 10, null, { pose: merge(poses.sp0, { phase: 2 }) }), fr('sp1', 6), fr('sp1', 30, null, { pose: merge(poses.sp1, { phase: 3 }) }), fr('cr2', 8), fr('idle0', 4)]),
    requiemFx: fxLoop({ type: 'slash', size: 50, color: RED }, 4, 3, 0.8),
    requiemHit: burst(RED, 30),
  },
  moves: [
    ...N.moves,
    special('crescent', 'Crescent Reaver', '~D, DF, F, x', 'crescent', { simple: 'sp1', projectile: { at: 3, anim: 'crescentFx', hitAnim: 'crescentHit', speed: [6, 0], offset: [52, -52], damage: 82, level: 'heavy' } }),
    special('moonrise', 'Moonrise Cleave', 'F, D, DF, y', 'moonrise', { simple: 'sp2', invuln: [1, 3], moves: [{ at: 2, vel: [2, -8.4] }], hits: [hit(2, 74, 'heavy', { launch: true }), hit(3, 40, 'heavy', { launch: true })] }),
    special('shadowstep', 'Shadowstep Rend', '~D, DB, B, a', 'shadowstep', { simple: 'sp3', fx: 'aura', moves: [{ at: 2, vel: [10, 0] }, { at: 3, vel: [6, 0] }], hits: [hit(2, 78, 'heavy', { knockback: [-5, 0] })] }),
    superMove('requiem', 'Pale Moon Requiem', '~D, DF, F, D, DF, F, x', 'super', { invuln: [1, 3], projectile: { at: 3, anim: 'requiemFx', hitAnim: 'requiemHit', speed: [4.5, 0], offset: [56, -52], damage: 58, hits: 5, level: 'heavy', launch: true, priority: 6, removeTime: 70 } }),
  ],
};
export default char;
