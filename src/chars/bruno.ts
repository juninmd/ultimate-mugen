import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, unarmedNormals } from './kit';

const B = basePoses({ armF: [40, 80], armB: [30, 90], legF: [18, 22], legB: [-16, 16], lean: 10, head: -8, breath: 1.3 });
const DUST = '#ffb060';
const N = unarmedNormals(B, {
  dmg: 1.22, speed: 1.25, open: true,
  names: { jab: 'Knife-Edge Chop', strong: 'Hammer Fist', lkick: 'Big Boot', hkick: 'Thunder Boot', cjab: 'Gut Chop', clk: 'Shin Stomp', sweep: 'Ring Sweep', ajab: 'Flying Chop', akick: 'Dropkick' },
  pose: {
    str0: merge(B.idle0, { shoulderF: 170, elbowF: 50, shoulderB: 160, elbowB: 60, lean: -6, head: -6 }),
    str1: merge(B.idle0, { shoulderF: 80, elbowF: 10, shoulderB: 70, elbowB: 20, lean: 26, head: -18, thighF: 34, kneeF: 30, thighB: -24, kneeB: 4, face: 'shout' }),
    akick: merge(B.jumpTop, { thighF: 85, kneeF: 0, ankleF: 80, thighB: 80, kneeB: 5, ankleB: 80, lean: -40, head: 20, shoulderF: 30, elbowF: 60, rot: 10, wind: 0.8 }),
  },
});
N.anims.strong.frames[2].hit = at('handF', 12);
N.anims.akick.frames[1].hit = at('footF', 14);

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  br0: merge(B.crouch0, { lean: 30, head: -20, shoulderF: 60, elbowF: 120, shoulderB: 50, elbowB: 120 }),
  br1: merge(B.run1, { lean: 42, head: -30, shoulderF: 30, elbowF: 130, shoulderB: 20, elbowB: 120, thighF: 60, kneeF: 50, thighB: -30, kneeB: 10, face: 'shout', wind: 1 }),
  sl0: merge(B.crouch, { shoulderF: 60, elbowF: 40, lean: 10 }),
  sl1: merge(B.jumpUp, { shoulderF: 100, elbowF: 0, shoulderB: -80, elbowB: 10, handF: 'open', lean: -6, head: -10, thighF: 30, kneeF: 30, thighB: -10, kneeB: 50, face: 'shout', wind: 1 }),
  sl2: merge(B.jumpUp, { shoulderF: 120, elbowF: 0, shoulderB: -60, elbowB: 10, handF: 'open', lean: -10, head: -8, thighF: 20, kneeF: 40, thighB: -15, kneeB: 50, wind: 1, phase: 2 }),
  qs0: merge(B.idle0, { thighF: 100, kneeF: 100, ankleF: 10, lean: -8, head: -4, shoulderF: 120, elbowF: 50, shoulderB: 110, elbowB: 60, thighB: -4, kneeB: 6 }),
  qs1: merge(B.crouch0, { thighF: 55, kneeF: 70, lean: 22, head: -16, shoulderF: 30, elbowF: 70, shoulderB: 20, elbowB: 70, face: 'shout' }),
  av0: merge(B.crouch, { lean: 26, aura: true, face: 'shout' }),
  av1: merge(B.jumpUp, { shoulderF: 170, elbowF: 10, shoulderB: 165, elbowB: 15, thighF: 20, kneeF: 30, thighB: -10, kneeB: 40, aura: true, wind: 1, face: 'shout' }),
  av2: merge(B.jumpTop, { rot: 50, lean: 10, shoulderF: 150, elbowF: 20, shoulderB: 140, elbowB: 30, aura: true, free: true, hipH: 30 }),
  av3: merge({ rot: 90, thighF: 5, kneeF: 10, thighB: -5, kneeB: 10, shoulderF: 150, elbowF: 10, shoulderB: 140, elbowB: 10, head: -20, handF: 'open', handB: 'open', aura: true, face: 'shout', wind: 0 }),
  win0: merge(B.intro0, { shoulderF: 120, elbowF: 100, shoulderB: 110, elbowB: 110, head: -10, lean: -4, face: 'shout', wind: 0.4 }),
  win1: merge(B.intro0, { shoulderF: 130, elbowF: 110, shoulderB: 120, elbowB: 115, head: -14, lean: -6, face: 'smile', phase: 2 }),
};

const char: NativeChar = {
  format: 'umf/1',
  id: 'bruno',
  name: 'Bruno "Bulldozer" Volk',
  series: 'Iron Ring Federation',
  author: 'Ultimate MUGEN',
  description: 'A three-time heavyweight ring champion who treats every street corner like a title match.',
  color: '#d4882a',
  winQuote: 'HAHA! Ring the bell! Somebody ring the bell!',
  stats: { life: 1150, power: 3000, attack: 110, defense: 108, walk: [1.9, 1.7], run: [4.4, 3.8], jump: [-7.6, -2.2, 0, 2.2], height: 116, width: [20, 18] },
  art: {
    type: 'rig',
    style: {
      scale: 1.1, build: 1.45, torsoShape: 'heavy', top: 'bare', pants: 'trunks', belt: 'accent', beltTails: 0, gloves: 'wrap', boots: 'tall', scar: true,
      hair: { type: 'short', spikes: 6, min: 0.2, max: 0.45, bangs: 1, bangLen: 0.25, seed: 7 },
      eyes: 'fierce', facePaint: 'top2',
      colors: { skin: '#d89a70', hair: '#3a2214', eye: '#3a2a20', pants: '#c02a2a', top: '#c02a2a', top2: '#c02a2a', boot: '#2a2630', accent: '#f0c030', wrap: '#f0ece0', energy: DUST, aura: '#ff6a2a' },
    },
    poses,
  },
  palettes: [
    { pants: '#2a5ad0', top: '#2a5ad0', top2: '#2a5ad0', boot: '#e8e8f0', accent: '#d8d8e0', hair: '#e0c070', skin: '#f0c8a8' },
    { pants: '#1a1a22', top: '#1a1a22', top2: '#40c040', boot: '#40c040', accent: '#40c040', hair: '#1a1a22', skin: '#9a6040' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    bullrush: anim([fr('br0', 8), fr('br1', 4, at('head', 15)), fr('br1', 4, at('head', 15), { pose: merge(poses.br1, { phase: 1 }) }), fr('br1', 4, null, { pose: merge(poses.br1, { phase: 2 }) }), fr('br0', 10), fr('idle0', 4)]),
    lariat: anim([fr('sl0', 4), fr('sl1', 4, at('handF', 14)), fr('sl2', 4, at('handF', 14)), fr('sl1', 6, null, { pose: merge(poses.sl1, { phase: 3 }) }), fr('jumpDown', -1)]),
    quake: anim([fr('qs0', 10), fr('qs0', 4, null, { pose: merge(poses.qs0, { phase: 1 }) }), fr('qs1', 4), fr('qs1', 14), fr('idle0', 5)]),
    quakeFx: fxLoop({ type: 'shockwave', size: 16, color: DUST }, 4, 3, 1.2),
    quakeHit: burst(DUST, 18),
    super: anim([fr('av0', 10), fr('av1', 8, at('body', 18)), fr('av2', 8), fr('av3', 6, null, { hit: { part: 'body', box: [-34, -40, 44, 0] } }), fr('av3', -1, null, { hit: { part: 'body', box: [-34, -40, 44, 0] }, pose: merge(poses.av3, { phase: 2 }) })]),
  },
  moves: [
    ...N.moves,
    special('quake', 'Quake Stomp', '~D, DF, F, x', 'quake', { simple: 'sp1', projectile: { at: 3, anim: 'quakeFx', hitAnim: 'quakeHit', speed: [4.6, 0], offset: [34, 0], damage: 76, level: 'heavy', knockback: [-3, -5], launch: true } }),
    special('lariat', 'Skyhook Lariat', 'F, D, DF, y', 'lariat', { simple: 'sp2', invuln: [1, 3], moves: [{ at: 2, vel: [1.6, -6.6] }], hits: [hit(2, 90, 'heavy', { launch: true }), hit(3, 46, 'heavy', { launch: true })] }),
    special('bullrush', 'Bulldozer Rush', '~D, DB, B, a', 'bullrush', { simple: 'sp3', fx: 'aura', moves: [{ at: 2, vel: [7.5, 0] }, { at: 3, vel: [7.5, 0] }, { at: 4, vel: [4, 0] }], hits: [hit(2, 88, 'heavy', { knockback: [-7, -3], launch: true, shake: 8 })] }),
    superMove('avalanche', 'Avalanche Drop', '~D, DF, F, D, DF, F, x', 'super', { invuln: [1, 3], moves: [{ at: 2, vel: [3.4, -10] }], hits: [hit(2, 70, 'heavy', { launch: true, knockback: [-1, -9] }), hit(4, 150, 'heavy', { launch: true, shake: 18, knockback: [-4, -5] })] }),
  ],
};
export default char;
