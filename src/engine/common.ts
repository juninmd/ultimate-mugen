import { COMMON_CNS } from './common-states';
import { parseStates } from '../mugen/cns';
import { defaultConstants, type CharacterDef } from './chardef';

let cached: CharacterDef | null = null;
export function commonDef(): CharacterDef {
  if (cached) return cached;
  const states = new Map();
  parseStates(COMMON_CNS, states);
  cached = {
    id: 'common', name: 'common', displayName: 'common', author: 'Ultimate MUGEN', source: 'native',
    localW: 320, localH: 240, constants: defaultConstants(), sprites: new Map(), anims: new Map(), commands: [], states, sounds: new Map(),
  };
  return cached;
}

export function flatStage(): import('./match').StageInfo {
  return { name: 'Flat', boundLeft: -200, boundRight: 200, boundHigh: -60, boundLow: 0, p1x: -70, p2x: 70, zoffset: 210, verticalFollow: 0.2, tension: 60 };
}
