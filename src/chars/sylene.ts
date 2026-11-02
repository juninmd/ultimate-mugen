import type { NativeChar } from '../native/types';
import { anim, at, basePoses, burst, fr, fxLoop, hit, merge, special, standardAnims, superMove, unarmedNormals } from './kit';

const B = basePoses({ armF: [70, 40], armB: [30, 60], legF: [14, 14], legB: [-12, 10], lean: 2, head: -4, handF: 'open', handB: 'open', breath: 0.9 });
const ICE = '#8fe8ff';
const N = unarmedNormals(B, {
  dmg: 0.95, open: true,
  names: { jab: 'Frost Palm', strong: 'Rime Burst', lkick: 'Skate Kick', hkick: 'Aurora Heel', cjab: 'Low Palm', clk: 'Ice Step', sweep: 'Hoarfrost Sweep', ajab: 'Snowfall Palm', akick: 'Glacial Kick' },
  pose: {
    str1: merge(B.idle0, { shoulderF: 95, elbowF: 0, shoulderB: 90, elbowB: 6, handF: 'open', handB: 'open', lean: 14, head: -10, thighF: 30, kneeF: 24, thighB: -24, kneeB: 4, charge: 'F', chargeSize: 8, face: 'shout' }),
  },
});
N.anims.strong.frames[2].hit = at('handF', 13);

const poses: Record<string, Record<string, any>> = {
  ...B,
  ...N.poses,
  fl0: merge(B.idle0, { shoulderF: 150, elbowF: 40, shoulderB: 20, elbowB: 70, lean: -8, head: -6, charge: 'F', chargeSize: 6, wind: 0.6 }),
  fl1: merge(B.idle0, { shoulderF: 96, elbowF: 0, shoulderB: -20, elbowB: 40, lean: 14, head: -10, thighF: 32, kneeF: 24, thighB: -26, kneeB: 4, face: 'shout', wind: 0.9 }),
  gs0: merge(B.crouch, { shoulderF: 30, elbowF: 30, shoulderB: 20, elbowB: 30, lean: 24, charge: 'both', chargeSize: 5 }),
  gs1: merge(B.idle0, { shoulderF: 170, elbowF: 0, shoulderB: 160, elbowB: 10, lean: -6, head: -16, thighF: 20, kneeF: 10, face: 'shout', aura: true, wind: 1, hairWind: -1 }),
  bg0: merge(B.crouch0, { lean: 20, shoulderF: 60, elbowF: 40 }),
  bg1: merge(B.crouch, { thighF: 92, kneeF: 0, ankleF: 30, thighB: 30, kneeB: 140, lean: -18, head: 10, shoulderF: 140, elbowF: 20, shoulderB: -60, elbowB: 10, aura: true, wind: 1 }),
  aw0: merge(B.idle0, { shoulderF: 175, elbowF: 0, shoulderB: 170, elbowB: 5, lean: -10, head: -20, charge: 'both', chargeSize: 8, aura: true, face: 'shout', wind: 1, hairWind: -1.2 }),
  aw1: merge(B.idle0, { shoulderF: 94, elbowF: 0, shoulderB: 88, elbowB: 4, lean: 16, head: -12, thighF: 36, kneeF: 30, thighB: -30, kneeB: 2, aura: true, face: 'shout', wind: 1 }),
  win0: merge(B.intro0, { shoulderF: 110, elbowF: 70, shoulderB: 20, elbowB: 120, head: -4, face: 'smile', charge: 'F', chargeSize: 5, wind: 0.6 }),
  win1: merge(B.intro0, { shoulderF: 112, elbowF: 66, shoulderB: 20, elbowB: 120, head: -6, face: 'smile', charge: 'F', chargeSize: 6, wind: 0.9, phase: 2 }),
  intro0: merge(B.intro0, { shoulderF: 30, elbowF: 100, shoulderB: 35, elbowB: 110, face: 'closed' }),
};

const char: NativeChar = {
  format: 'umf/1',
  id: 'sylene',
  name: 'Sylene Vashti',
  series: 'Glacier Crown Saga',
  author: 'Ultimate MUGEN',
  description: 'The exiled heir of a frozen kingdom, who turns every breath of the battlefield into a blade of ice.',
  color: '#5ab8e8',
  winQuote: 'Hush now. The cold only hurts for a moment.',
  stats: { life: 940, power: 3000, attack: 98, defense: 94, walk: [2.2, 2.1], run: [5, 4.4], jump: [-8.6, -2.5, 0, 2.6], height: 100, width: [14, 14] },
  art: {
    type: 'rig',
    style: {
      female: true, top: 'robe', sleeves: 'wide', pants: 'tight', belt: 'top2', beltTails: 1, gloves: 'glove', boots: 'tall',
      skirt: { type: 'robe', length: 42, color: 'top', width: 1.1 },
      hair: { type: 'long', length: 40, spikes: 6, min: 0.25, max: 0.55, bangs: 3, bangLen: 0.6, seed: 9 },
      horns: { size: 0.45 }, eyes: 'cute', undershirt: 'top2',
      colors: { skin: '#f8e0d4', hair: '#d8f0ff', eye: '#3ac0f0', top: '#eef6fc', top2: '#4a8ad0', trim: '#3a6ab8', pants: '#2a3a68', glove: '#2a3a68', boot: '#eef6fc', horn: '#a8ecff', energy: ICE, aura: ICE, lips: '#c06080' },
    },
    poses,
  },
  palettes: [
    { top: '#2a2a3a', top2: '#c03a8a', trim: '#c03a8a', hair: '#f0f0f8', pants: '#1a1a24', glove: '#1a1a24', boot: '#2a2a3a', horn: '#ff9ad8', eye: '#c03a8a', energy: '#ff8ad8', aura: '#ff8ad8' },
    { top: '#f4ecd8', top2: '#2a9a6a', trim: '#2a9a6a', hair: '#e0c080', pants: '#3a2a1a', glove: '#3a2a1a', boot: '#f4ecd8', horn: '#b0ffb0', eye: '#2a9a6a', energy: '#9affc8', aura: '#9affc8' },
  ],
  anims: {
    ...standardAnims(),
    ...N.anims,
    lance: anim([fr('fl0', 8), fr('fl0', 4, null, { pose: merge(poses.fl0, { phase: 2 }) }), fr('fl1', 4), fr('fl1', 14, null, { pose: merge(poses.fl1, { phase: 2 }) }), fr('jab0', 5), fr('idle0', 3)]),
    lanceFx: fxLoop({ type: 'ice', size: 13, color: ICE }, 4, 3, 0.8),
    iceHit: burst(ICE, 18),
    spire: anim([fr('gs0', 5), fr('gs0', 3, null, { pose: merge(poses.gs0, { phase: 1 }) }), fr('gs1', 6), fr('gs1', 14, null, { pose: merge(poses.gs1, { phase: 2 }) }), fr('gs0', 6), fr('idle0', 3)]),
    spireFx: fxLoop({ type: 'pillar', size: 14, length: 110, color: ICE }, 4, 3, 1.1),
    glide: anim([fr('bg0', 6), fr('bg1', 4, at('footF', 12)), fr('bg1', 6, at('footF', 12), { pose: merge(poses.bg1, { phase: 2 }) }), fr('bg1', 4, null, { pose: merge(poses.bg1, { phase: 3, aura: false }) }), fr('crouch', 8)]),
    super: anim([fr('aw0', 12), fr('aw0', 12, null, { pose: merge(poses.aw0, { phase: 2 }) }), fr('aw1', 4), fr('aw1', 44, null, { pose: merge(poses.aw1, { phase: 3 }) }), fr('fl1', 8), fr('idle0', 4)]),
    winterBeam: fxLoop({ type: 'beam', size: 15, length: 230, color: ICE }, 4, 3, 1.3),
  },
  moves: [
    ...N.moves,
    special('lance', 'Frost Lance', '~D, DF, F, x', 'lance', { simple: 'sp1', projectile: { at: 3, anim: 'lanceFx', hitAnim: 'iceHit', speed: [5.2, 0], offset: [44, -70], damage: 70 } }),
    special('spire', 'Glacier Spire', 'F, D, DF, y', 'spire', { simple: 'sp2', invuln: [1, 3], projectile: { at: 3, anim: 'spireFx', hitAnim: 'iceHit', speed: [0, 0], offset: [34, 0], damage: 48, hits: 2, removeTime: 22, level: 'heavy', launch: true, knockback: [-2, -8], priority: 3 } }),
    special('glide', 'Blizzard Glide', '~D, DB, B, a', 'glide', { simple: 'sp3', fx: 'aura', moves: [{ at: 2, vel: [7, 0] }, { at: 3, vel: [6, 0] }, { at: 4, vel: [2, 0] }], hits: [hit(2, 66, 'heavy', { guard: 'low', launch: true, knockback: [-2, -5] })] }),
    superMove('winter', 'Absolute Winter', '~D, DF, F, D, DF, F, x', 'super', { invuln: [1, 3], projectile: { at: 3, anim: 'winterBeam', hitAnim: 'iceHit', speed: [0, 0], offset: [44, -68], damage: 46, hits: 7, removeTime: 44, level: 'heavy', launch: true, priority: 6 } }),
  ],
};
export default char;
