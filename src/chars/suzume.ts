import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fx, fxLoop, hit, merge, special, standardAnims, superMove, unarmedNormals } from './kit';

const B = basePoses({ armF: [60, 70], armB: [20, 125], legF: [26, 40], legB: [-22, 26], lean: 14, head: -10, breath: 0.8 });
const VIO = '#c86bff';
const N = unarmedNormals(B, {
  dmg: 0.9, speed: 0.85,
  names: { jab: 'Kunai Flick', strong: 'Reverse Edge', lkick: 'Shin Snap', hkick: 'Crescent Heel', cjab: 'Low Cut', clk: 'Ankle Tap', sweep: 'Shadow Sweep', ajab: 'Falling Cut', akick: 'Hawk Dive' },
  pose: { hk1: merge(B.idle0, { thighF: 150, kneeF: 2, ankleF: 30, lean: -38, head: 30, thighB: 0, kneeB: 6, shoulderF: 20, elbowF: 80, shoulderB: -40, elbowB: 50, face: 'shout', wind: 1 }) },
});

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  kv0: merge(B.idle0, { shoulderF: 160, elbowF: 60, shoulderB: 40, elbowB: 100, lean: -6, head: -6 }),
  kv1: merge(B.idle0, { shoulderF: 92, elbowF: 0, shoulderB: -20, elbowB: 60, lean: 18, head: -12, thighF: 40, kneeF: 34, thighB: -30, kneeB: 4, handF: 'open', noWeapon: true, wind: 0.8 }),
  sf0: merge(B.crouch, { lean: 20 }),
  sf1: merge(B.jumpUp, { thighF: 160, kneeF: 4, ankleF: 30, thighB: -10, kneeB: 50, lean: -30, head: 20, shoulderF: -30, elbowF: 40, shoulderB: -50, elbowB: 30, face: 'shout', wind: 1 }),
  sf2: merge(B.flip0, { rot: -120 }),
  sf3: merge(B.flip0, { rot: -230 }),
  sf4: merge(B.flip0, { rot: -320 }),
  tp0: merge(B.crouch0, { shoulderF: 100, elbowF: 120, shoulderB: 90, elbowB: 130, handF: 'open', lean: 10, head: 10, face: 'closed' }),
  tp2: merge(B.idle0, { shoulderF: 95, elbowF: 5, lean: 24, head: -16, thighF: 44, kneeF: 40, thighB: -34, kneeB: 6, weapon: 90, face: 'shout', wind: 1 }),
  ks0: merge(B.jumpTop, { shoulderF: 165, elbowF: 40, shoulderB: 150, elbowB: 50, lean: -10, aura: true, face: 'shout' }),
  ks1: merge(B.jumpTop, { shoulderF: 90, elbowF: 0, shoulderB: 30, elbowB: 90, lean: 16, handF: 'open', noWeapon: true, aura: true, wind: 1 }),
  ks2: merge(B.jumpTop, { shoulderF: 40, elbowF: 100, shoulderB: 92, elbowB: 0, lean: 18, handB: 'open', noWeapon: true, aura: true, wind: 1, phase: 2 }),
  win0: merge(B.intro0, { shoulderF: 100, elbowF: 140, shoulderB: 90, elbowB: 150, handF: 'open', handB: 'open', head: 6, face: 'closed', wind: 0.8, noWeapon: true }),
  win1: merge(B.intro0, { shoulderF: 100, elbowF: 140, shoulderB: 90, elbowB: 150, handF: 'open', handB: 'open', head: 6, face: 'closed', wind: 1, phase: 2, noWeapon: true }),
};

const KUNAI = { type: 'kunai', size: 12, color: VIO };
const char: NativeChar = {
  format: 'umf/1',
  id: 'suzume',
  name: 'Suzume Kagerou',
  series: 'Crimson Lotus Shinobi',
  author: 'Ultimate MUGEN',
  description: 'A soft-spoken kunoichi of a vanished clan who strikes from the blind spot between two heartbeats.',
  color: '#8a3ac8',
  winQuote: 'You were watching my blade. You should have watched my shadow.',
  stats: { life: 920, power: 3000, attack: 96, defense: 94, walk: [2.8, 2.5], run: [6.2, 5.2], jump: [-9.2, -2.8, 0, 3.0], height: 96, width: [14, 14], airJumps: 1 },
  art: {
    type: 'rig',
    style: {
      female: true, scale: 0.96, top: 'gi', sleeves: 'short', pants: 'tight', belt: 'top2', beltTails: 1, gloves: 'long', boots: 'tall',
      skirt: { type: 'robe', length: 20, color: 'top', width: 0.9 },
      hair: { type: 'ponytail', length: 34, spikes: 6, min: 0.25, max: 0.5, bangs: 3, bangLen: 0.5, seed: 3 },
      scarf: { color: 'cloth', length: 42 }, mask: true,
      weapon: { type: 'kunai', length: 12, angle: 100 },
      undershirt: 'mask',
      colors: { skin: '#f4d2b8', hair: '#3a2456', eye: '#b04ad8', top: '#5a3e96', top2: '#e04a6a', trim: '#e04a6a', pants: '#3a2e5e', cloth: '#e04a6a', glove: '#2e2448', boot: '#2e2448', mask: '#2e2448', weapon: '#e8eef8', hilt: '#e04a6a', accent: '#e04a6a', energy: VIO, aura: VIO, legs: '#3a2e5e' },
    },
    poses,
  },
  palettes: [
    { top: '#e8e4ec', pants: '#2a2a36', cloth: '#2a8ad8', top2: '#2a8ad8', trim: '#2a8ad8', hair: '#d8c8f0', mask: '#2a2a36', glove: '#2a2a36', boot: '#2a2a36', energy: '#4fc8ff', aura: '#4fc8ff' },
    { top: '#2a4a2a', pants: '#1a241a', cloth: '#e0a020', top2: '#e0a020', trim: '#e0a020', hair: '#c83020', mask: '#1a241a', glove: '#1a241a', boot: '#1a241a', energy: '#ffb030', aura: '#ffb030' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    volley: anim([fr('kv0', 6), fr('kv1', 3), fr('kv1', 14), fr('jab0', 5), fr('idle0', 3)]),
    kunaiFx: fxLoop(KUNAI, 2, 3, 0.5),
    kunaiHit: burst(VIO, 12),
    swallow: anim([fr('sf0', 3), fr('sf1', 5, at('footF', 12)), fr('sf2', 4), fr('sf3', 4), fr('sf4', 4), fr('jumpDown', -1)]),
    shadow: anim([fr('tp0', 6), fx({ type: 'burst', size: 26, color: VIO, phase: 0.3, nohit: true }, 5, { hurt: 'none' }), fx({ type: 'burst', size: 26, color: VIO, phase: 0.7, nohit: true }, 4, { hurt: 'none' }), fr('tp2', 5, at('weapon', 8)), fr('tp2', 8), fr('idle0', 4)]),
    super: anim([fr('ks0', 10), fr('ks1', 4), fr('ks2', 4), fr('ks1', 4, null, { pose: merge(poses.ks1, { phase: 3 }) }), fr('ks2', 4, null, { pose: merge(poses.ks2, { phase: 4 }) }), fr('ks1', 4, null, { pose: merge(poses.ks1, { phase: 5 }) }), fr('ks2', 10), fr('jumpTop', -1)]),
    stormKunai: fxLoop({ ...KUNAI, size: 14 }, 2, 3, 0.5),
  },
  moves: [
    ...N.moves,
    special('volley', 'Kunai Volley', '~D, DF, F, x', 'volley', { simple: 'sp1', projectile: { at: 2, anim: 'kunaiFx', hitAnim: 'kunaiHit', speed: [9, 0], offset: [44, -70], damage: 54, level: 'light' } }),
    special('swallow', 'Swallow Flip', 'F, D, DF, y', 'swallow', { simple: 'sp2', invuln: [1, 2], moves: [{ at: 2, vel: [1.5, -9] }, { at: 3, vel: [-1, null] }], hits: [hit(2, 82, 'heavy', { launch: true })] }),
    special('shadow', 'Shadow Step', '~D, DB, B, a', 'shadow', { simple: 'sp3', teleport: 3, invuln: [2, 3], hits: [hit(4, 70, 'heavy', { knockback: [-5, 0] })] }),
    superMove('storm', 'Thousand Kunai Storm', '~D, DF, F, D, DF, F, x', 'super', {
      moves: [{ at: 1, vel: [0, -6] }],
      projectile: [2, 3, 4, 5, 6].map((f, i) => ({ at: f, anim: 'stormKunai', hitAnim: 'kunaiHit', speed: [8, 2.4 - i * 0.2] as [number, number], offset: [36, -66 + i * 3] as [number, number], damage: 46, level: 'medium' as const, launch: i === 4 })),
    }),
  ],
};
export default char;
