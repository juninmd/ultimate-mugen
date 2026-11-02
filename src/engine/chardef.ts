import type { Animation } from '../mugen/air';
import type { CommandDef } from './commands';
import { compileNode, parseExprList, type Compiled, type Fn, type Node } from './expr';

/** Runtime sprite (drawable). */
export interface Sprite {
  image: CanvasImageSource | null;
  width: number;
  height: number;
  axisX: number;
  axisY: number;
  /** extra draw scale (e.g. hi-res procedurally rendered sprites use 0.5) */
  scale?: number;
}

export interface ParamVal { raw: string; nodes: Node[] | null; compiled?: Compiled<any>[] }

export interface StateController {
  type: string; // lowercase controller type
  label: string;
  triggerAll: Node[];
  triggers: Node[][];
  params: Map<string, ParamVal>;
  persistent: number;
  ignoreHitPause: boolean;
  /** compiled trigger functions (lazily filled by the VM) */
  _tAll?: Fn<any>[];
  _t?: Fn<any>[][];
  uid: number;
}

export interface StateDef {
  id: number;
  params: Map<string, ParamVal>;
  controllers: StateController[];
  owner?: CharacterDef;
}

export interface CharacterDef {
  id: string;
  name: string;
  displayName: string;
  author: string;
  source: 'mugen' | 'native';
  /** Localcoord width: coordinates are scaled by 320/localW to engine space */
  localW: number;
  localH: number;
  constants: Map<string, number>;
  sprites: Map<string, Sprite>;
  anims: Map<number, Animation>;
  commands: CommandDef[];
  states: Map<number, StateDef>;
  /** state numbers -3,-2,-1 merged lists */
  sounds: Map<string, AudioBuffer | ArrayBuffer | Uint8Array | ((ctx: AudioContext) => AudioBuffer)>;
  portrait?: Sprite;
  bigPortrait?: Sprite;
  /** list of {name, input} for move list & AI */
  movelist?: { name: string; input: string; command?: string }[];
  /** commands useful for AI: names of commands that trigger attacks */
  aiCommands?: string[];
  /** theme color used by UI */
  color?: string;
  series?: string;
  description?: string;
}

let uidCounter = 1;
export const nextUid = () => uidCounter++;

export function makeParam(raw: string): ParamVal {
  let nodes: Node[] | null = null;
  try { nodes = parseExprList(raw); } catch { nodes = null; }
  return { raw, nodes };
}

export function compileParam<H>(p: ParamVal, resolver: any): Compiled<H>[] {
  if (p.compiled) return p.compiled;
  const out: Compiled<H>[] = [];
  if (p.nodes) {
    for (const n of p.nodes) {
      try { out.push(compileNode<H>(n, resolver)); } catch (e) { out.push({ f: () => 0, float: false }); }
    }
  }
  p.compiled = out;
  return out;
}

/** Default constants (MUGEN defaults, 320x240 space). */
export function defaultConstants(): Map<string, number> {
  const m = new Map<string, number>();
  const set = (k: string, v: number) => m.set(k, v);
  set('data.life', 1000); set('data.power', 3000); set('data.attack', 100); set('data.defence', 100);
  set('data.fall.defence_up', 50); set('data.liedown.time', 60); set('data.airjuggle', 15);
  set('data.sparkno', 2); set('data.guard.sparkno', 40); set('data.ko.echo', 0); set('data.volume', 0);
  set('data.intpersistindex', 60); set('data.floatpersistindex', 40);
  set('size.xscale', 1); set('size.yscale', 1);
  set('size.ground.back', 15); set('size.ground.front', 16); set('size.air.back', 12); set('size.air.front', 12);
  set('size.height', 60); set('size.attack.dist', 160); set('size.proj.attack.dist', 90);
  set('size.proj.doscale', 0); set('size.head.pos.x', -5); set('size.head.pos.y', -90);
  set('size.mid.pos.x', -5); set('size.mid.pos.y', -60); set('size.shadowoffset', 0);
  set('size.draw.offset.x', 0); set('size.draw.offset.y', 0);
  set('velocity.walk.fwd.x', 2.4); set('velocity.walk.back.x', -2.2);
  set('velocity.run.fwd.x', 4.6); set('velocity.run.fwd.y', 0);
  set('velocity.run.back.x', -4.5); set('velocity.run.back.y', -3.8);
  set('velocity.jump.y', -8.4); set('velocity.jump.neu.x', 0); set('velocity.jump.back.x', -2.55); set('velocity.jump.fwd.x', 2.5);
  set('velocity.runjump.back.x', -2.55); set('velocity.runjump.back.y', -8.1); set('velocity.runjump.fwd.x', 4); set('velocity.runjump.fwd.y', -8.1);
  set('velocity.airjump.y', -8.1); set('velocity.airjump.neu.x', 0); set('velocity.airjump.back.x', -2.55); set('velocity.airjump.fwd.x', 2.5);
  set('velocity.air.gethit.groundrecover.x', -0.15); set('velocity.air.gethit.groundrecover.y', -3.5);
  set('velocity.air.gethit.airrecover.mul.x', 0.5); set('velocity.air.gethit.airrecover.mul.y', 0.2);
  set('velocity.air.gethit.airrecover.add.x', 0); set('velocity.air.gethit.airrecover.add.y', -4.5);
  set('velocity.air.gethit.airrecover.back', -1); set('velocity.air.gethit.airrecover.fwd', 0);
  set('velocity.air.gethit.airrecover.up', -2); set('velocity.air.gethit.airrecover.down', 1.5);
  set('movement.airjump.num', 1); set('movement.airjump.height', 35); set('movement.yaccel', 0.44);
  set('movement.stand.friction', 0.85); set('movement.crouch.friction', 0.82);
  set('movement.stand.friction.threshold', 2); set('movement.crouch.friction.threshold', 0.05);
  set('movement.air.gethit.groundlevel', 25); set('movement.air.gethit.groundrecover.ground.threshold', -20);
  set('movement.air.gethit.groundrecover.groundlevel', 10); set('movement.air.gethit.airrecover.threshold', -1);
  set('movement.air.gethit.airrecover.yaccel', 0.35); set('movement.air.gethit.trip.groundlevel', 15);
  set('movement.down.bounce.offset.x', 0); set('movement.down.bounce.offset.y', 20);
  set('movement.down.bounce.yaccel', 0.4); set('movement.down.bounce.groundlevel', 12);
  set('movement.down.friction.threshold', 0.05);
  return m;
}
