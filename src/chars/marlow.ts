import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, swordNormals } from './kit';

const B = basePoses({ armF: [62, 30], armB: [-10, 120], legF: [24, 26], legB: [-18, 14], lean: 6, head: -6, blade: 115, handB: 'open' });
const SEA = '#3ac8ff';
const N = swordNormals(B, {
  speed: 0.95,
  names: { jab: 'Quick Thrust', strong: 'Broadside Slash', lkick: 'Boarding Kick', hkick: 'Mast Splitter', cjab: 'Deck Cut', clk: 'Shin Kick', sweep: 'Keelhaul Sweep', ajab: 'Rigging Slash', akick: 'Plunder Dive' },
  pose: {
    jab1: merge(B.idle0, { shoulderF: 92, elbowF: 0, blade: 92, lean: 18, head: -12, thighF: 38, kneeF: 30, thighB: -26, kneeB: 4, wind: 0.6 }),
  },
});

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  rs0: merge(B.idle0, { shoulderF: 30, elbowF: 140, blade: 250, lean: -6, head: -6, glow: true }),
  rs1: merge(B.idle0, { shoulderF: 100, elbowF: 0, blade: 110, lean: 20, head: -14, thighF: 40, kneeF: 32, thighB: -30, kneeB: 4, glow: true, face: 'shout', wind: 1 }),
  rs2: merge(B.idle0, { shoulderF: 95, elbowF: 6, blade: 80, lean: 14, head: -10, thighF: 40, kneeF: 32, thighB: -30, kneeB: 4 }),
  ws0: merge(B.crouch, { shoulderF: 10, elbowF: 0, blade: 15, lean: 26, glow: true }),
  ws1: merge(B.jumpUp, { thighF: 35, kneeF: 30, thighB: -12, kneeB: 70, ankleF: 45, shoulderF: 168, elbowF: 4, blade: 175, lean: -4, head: -16, face: 'shout', glow: true, aura: true, wind: 1 }),
  ws2: merge(B.jumpTop, { shoulderF: 150, elbowF: 20, blade: 150 }),
  al0: merge(B.crouch0, { shoulderF: 40, elbowF: 140, blade: 95, lean: 20, head: -10, shoulderB: -40, elbowB: 30 }),
  al1: merge(B.run1, { shoulderF: 92, elbowF: 0, blade: 92, lean: 32, head: -24, thighF: 70, kneeF: 50, thighB: -34, kneeB: 10, shoulderB: -50, elbowB: 10, glow: true, aura: true, face: 'shout', wind: 1 }),
  mf0: merge(B.idle0, { shoulderF: 175, elbowF: 10, blade: 182, shoulderB: 120, elbowB: 60, lean: -8, head: -14, aura: true, glow: true, face: 'shout', wind: 1 }),
  mf1: merge(B.idle0, { shoulderF: 80, elbowF: 0, blade: 60, shoulderB: -40, elbowB: 20, lean: 28, head: -18, thighF: 46, kneeF: 44, thighB: -34, kneeB: 2, aura: true, glow: true, face: 'shout', wind: 1 }),
  win0: merge(B.intro0, { shoulderF: 150, elbowF: 30, blade: 160, shoulderB: 20, elbowB: 120, handB: 'open', head: -10, face: 'smile', wind: 0.8 }),
  win1: merge(B.intro0, { shoulderF: 152, elbowF: 28, blade: 162, shoulderB: 20, elbowB: 120, handB: 'open', head: -12, face: 'smile', wind: 1, phase: 2 }),
  intro0: merge(B.intro0, { shoulderF: 20, elbowF: 30, blade: 30, shoulderB: 60, elbowB: 130, handB: 'open', face: 'smile' }),
};

const char: NativeChar = {
  format: 'umf/1',
  id: 'marlow',
  name: 'Captain Marlow Reyes',
  series: 'Saltwind Corsairs',
  author: 'Ultimate MUGEN',
  description: 'A grinning sky-sea captain whose cutlass was quenched in a storm and still carries the tide inside it.',
  color: '#1e5aa8',
  winQuote: 'Plunder? Nah. I just came for the view... and your treasure map.',
  stats: { life: 1000, power: 3000, attack: 102, defense: 100, walk: [2.4, 2.2], run: [5.4, 4.6], jump: [-8.4, -2.5, 0, 2.6], height: 104, width: [16, 15] },
  art: {
    type: 'rig',
    style: {
      top: 'coat', sleeves: 'long', pants: 'tight', belt: 'cloth', beltTails: 1, gloves: 'glove', boots: 'tall', hat: true, scar: true,
      skirt: { type: 'coat', length: 44, color: 'top' },
      hair: { type: 'ponytail', length: 20, spikes: 6, min: 0.3, max: 0.6, bangs: 2, seed: 21 },
      weapon: { type: 'cutlass', length: 40 },
      colors: { skin: '#d8a078', hair: '#3a2416', eye: '#2a6ab0', top: '#1e3a6a', top2: '#f0ece0', trim: '#d8a830', pants: '#4a3424', cloth: '#b82a2a', glove: '#5a3a24', boot: '#2a1c14', hat: '#1e1a22', weapon: '#d8e2ec', hilt: '#3a2416', accent: '#d8a830', energy: SEA, aura: SEA },
    },
    poses,
  },
  palettes: [
    { top: '#7a1a24', cloth: '#1a1a22', trim: '#e0e0e8', accent: '#e0e0e8', hat: '#2a1a1a', pants: '#2a2a32', hair: '#d8c090', energy: '#ff7a4a', aura: '#ff7a4a' },
    { top: '#f0ece0', cloth: '#2a8a6a', trim: '#2a8a6a', accent: '#d8a830', hat: '#f0ece0', pants: '#2a3a4a', hair: '#1a1a22', energy: '#6affd0', aura: '#6affd0' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    riptide: anim([fr('rs0', 8), fr('rs0', 4, null, { pose: merge(poses.rs0, { phase: 2 }) }), fr('rs1', 4, at('weapon', 4)), fr('rs2', 14), fr('idle0', 5)]),
    riptideFx: fxLoop({ type: 'water', size: 22, color: SEA }, 4, 3, 1),
    splash: burst(SEA, 20),
    spout: anim([fr('ws0', 4), fr('ws1', 4, at('weapon', 6), { pose: merge(poses.ws1, { shoulderF: 118, elbowF: 4, blade: 105, lean: 10 }) }), fr('ws1', 4, at('weapon', 6), { pose: merge(poses.ws1, { phase: 2 }) }), fr('ws2', 10), fr('jumpDown', -1)]),
    lunge: anim([fr('al0', 7), fr('al1', 4, at('weapon', 6)), fr('al1', 5, null, { pose: merge(poses.al1, { phase: 2 }) }), fr('rs2', 10), fr('idle0', 4)]),
    super: anim([fr('mf0', 14), fr('mf0', 10, null, { pose: merge(poses.mf0, { phase: 2 }) }), fr('mf1', 6), fr('mf1', 30, null, { pose: merge(poses.mf1, { phase: 3 }) }), fr('rs2', 8), fr('idle0', 4)]),
    maelstromFx: fxLoop({ type: 'wave', size: 46, color: SEA }, 4, 3, 1),
  },
  moves: [
    ...N.moves,
    special('riptide', 'Riptide Slash', '~D, DF, F, x', 'riptide', { simple: 'sp1', projectile: { at: 3, anim: 'riptideFx', hitAnim: 'splash', speed: [5.6, 0], offset: [50, -56], damage: 74 } }),
    special('spout', 'Waterspout', 'F, D, DF, y', 'spout', { simple: 'sp2', invuln: [1, 3], moves: [{ at: 2, vel: [2, -8.4] }], hits: [hit(2, 72, 'heavy', { launch: true }), hit(3, 42, 'heavy', { launch: true })] }),
    special('lunge', 'Anchor Lunge', '~D, DB, B, a', 'lunge', { simple: 'sp3', fx: 'aura', moves: [{ at: 2, vel: [10, 0] }, { at: 3, vel: [6, 0] }], hits: [hit(2, 76, 'heavy', { knockback: [-6, 0] })] }),
    superMove('maelstrom', 'Maelstrom Fury', '~D, DF, F, D, DF, F, x', 'super', { invuln: [1, 3], projectile: { at: 3, anim: 'maelstromFx', hitAnim: 'splash', speed: [3.6, 0], offset: [56, -50], damage: 52, hits: 6, level: 'heavy', launch: true, priority: 6, removeTime: 90 } }),
  ],
};
export default char;
