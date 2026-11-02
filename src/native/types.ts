/**
 * UMF — Ultimate Mugen Format: a simple, declarative character format.
 * A whole fighter is one JSON/TS object: stats, art, named animations and a move list.
 * It is compiled into the same state machine used for M.U.G.E.N characters.
 */
export type Box = [number, number, number, number];

/** Which body part is "active" (hitbox source) on a frame */
export interface ActiveSpec {
  part: string; // e.g. 'handR', 'footL', 'weapon', 'body', 'aura'
  radius?: number; // hitbox radius around the part's end point
  box?: Box; // or an explicit box (local coords, facing right, feet at 0,0)
}

export interface NativeFrame {
  /** pose name (from art.poses) or inline pose object (rig art) */
  pose?: string | Record<string, any>;
  /** effect frame (projectiles/fx): rendered by the rig fx renderer */
  effect?: Record<string, any>;
  /** sheet art: sprite name */
  sprite?: string;
  ticks: number; // -1 = hold forever
  hit?: ActiveSpec;
  hurt?: Box[] | 'none';
  offset?: [number, number];
}

export interface NativeAnim {
  frames: NativeFrame[];
  loop?: number; // loop start frame index (default 0); frames loop forever
}

export type GuardType = 'high' | 'low' | 'mid' | 'air' | 'unblockable';
export type HitLevel = 'light' | 'medium' | 'heavy';

export interface HitSpec {
  /** frame (1-based) on which the hit becomes active */
  at: number;
  damage: number;
  guardDamage?: number;
  level?: HitLevel;
  guard?: GuardType;
  /** knockback velocity [x, y] on ground (x negative = away) */
  knockback?: [number, number];
  airKnockback?: [number, number];
  /** launch / knockdown */
  launch?: boolean;
  hitstop?: number;
  hitstun?: number;
  spark?: number;
  shake?: number;
}

export interface ProjectileSpec {
  at: number; // frame (1-based) that spawns it
  anim: string;
  hitAnim?: string;
  speed: [number, number];
  accel?: [number, number];
  offset: [number, number];
  hits?: number;
  damage: number;
  level?: HitLevel;
  launch?: boolean;
  knockback?: [number, number];
  removeTime?: number;
  priority?: number;
}

export interface NativeMove {
  id: string;
  name: string;
  /** MUGEN command syntax, e.g. "x", "D,DF,F,x", "F,F", "~D,DF,F,x+y" */
  input: string;
  /** simple-mode (mobile) binding: which special button triggers it */
  simple?: 'sp1' | 'sp2' | 'sp3' | 'super';
  kind: 'normal' | 'special' | 'super';
  when?: 'stand' | 'crouch' | 'air';
  anim: string;
  hits?: HitSpec[];
  /** velocity changes: at frame N set vel */
  moves?: { at: number; vel: [number | null, number | null] }[];
  projectile?: ProjectileSpec | ProjectileSpec[];
  /** power cost (1000 = one bar) */
  cost?: number;
  /** invulnerable frames range [from, to] (1-based frames) */
  invuln?: [number, number];
  /** air move physics: keep momentum */
  airPhysics?: boolean;
  /** extra description for movelist */
  note?: string;
  /** teleport behind opponent at frame N */
  teleport?: number;
  /** effect played on start: 'aura', 'flash' */
  fx?: string;
  /** sound cue on start */
  voice?: string;
}

export interface NativeStats {
  life?: number;
  power?: number;
  attack?: number;
  defense?: number;
  walk?: [number, number];
  run?: [number, number];
  jump?: [number, number, number, number]; // y, back x, neutral x, fwd x
  gravity?: number;
  airJumps?: number;
  height?: number;
  width?: [number, number];
}

export interface NativeChar {
  format: 'umf/1';
  id: string;
  name: string;
  /** original universe/tagline */
  series: string;
  author: string;
  description?: string;
  color: string;
  stats?: NativeStats;
  art: { type: 'rig'; style: Record<string, any>; poses: Record<string, Record<string, any>> } | { type: 'sheet'; image: string; sprites: Record<string, [number, number, number, number, number, number]> };
  anims: Record<string, NativeAnim>;
  moves: NativeMove[];
  /** palettes: alternate color sets (rig style overrides) */
  palettes?: Record<string, any>[];
  winQuote?: string;
}
