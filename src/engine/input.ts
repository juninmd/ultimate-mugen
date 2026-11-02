/** Input bits (absolute directions; converted to relative B/F at command check time). */
export const K = {
  U: 1, D: 2, L: 4, R: 8,
  a: 16, b: 32, c: 64, x: 128, y: 256, z: 512, s: 1024, d: 2048, w: 4096,
} as const;
export type KeyName = keyof typeof K;
export const BUTTONS: KeyName[] = ['a', 'b', 'c', 'x', 'y', 'z', 's', 'd', 'w'];

export interface InputSource {
  /** Called once per tick; returns the bitmask held this tick. */
  poll(): number;
}

/** Ring buffer of input bitmasks. */
export class InputHistory {
  static SIZE = 256;
  buf = new Uint16Array(InputHistory.SIZE);
  /** facing at each tick (1/-1) */
  face = new Int8Array(InputHistory.SIZE);
  tick = -1;
  push(bits: number, facing: number) {
    this.tick++;
    this.buf[this.tick & 255] = bits;
    this.face[this.tick & 255] = facing;
  }
  at(t: number): number { return t < 0 || t > this.tick || this.tick - t >= InputHistory.SIZE - 1 ? 0 : this.buf[t & 255]; }
  faceAt(t: number): number { return t < 0 ? 1 : this.face[t & 255] || 1; }
}

/** Relative direction bits: U=1, D=2, B=4, F=8 */
export function relDirs(bits: number, facing: number): number {
  let r = bits & 3;
  const left = bits & K.L, right = bits & K.R;
  if (left && right) return r; // SOCD neutral
  if (facing >= 0) { if (left) r |= 4; if (right) r |= 8; }
  else { if (right) r |= 4; if (left) r |= 8; }
  return r;
}

// ------------------------------------------------------------ keyboard/gamepad/touch

export interface KeyMap { [code: string]: KeyName }

export const DEFAULT_P1_KEYS: KeyMap = {
  KeyW: 'U', KeyS: 'D', KeyA: 'L', KeyD: 'R',
  KeyU: 'x', KeyI: 'y', KeyO: 'z', KeyJ: 'a', KeyK: 'b', KeyL: 'c', Enter: 's',
  // arrows also map to P1 when P2 is CPU
  ArrowUp: 'U', ArrowDown: 'D', ArrowLeft: 'L', ArrowRight: 'R',
  KeyZ: 'a', KeyX: 'b', KeyC: 'c', KeyV: 'x', KeyB: 'y', KeyN: 'z',
};

export class KeyboardState {
  down = new Set<string>();
  pressedThisFrame = new Set<string>();
  constructor(target: Window | null) {
    if (!target) return;
    target.addEventListener('keydown', (e) => {
      const ke = e as KeyboardEvent;
      if (!this.down.has(ke.code)) this.pressedThisFrame.add(ke.code);
      this.down.add(ke.code);
      if (ke.code.startsWith('Arrow') || ke.code === 'Space') ke.preventDefault();
    });
    target.addEventListener('keyup', (e) => this.down.delete((e as KeyboardEvent).code));
    target.addEventListener('blur', () => this.down.clear());
  }
  endFrame() { this.pressedThisFrame.clear(); }
}

export class KeyboardSource implements InputSource {
  constructor(public kb: KeyboardState, public map: KeyMap) {}
  poll(): number {
    let b = 0;
    for (const code of this.kb.down) { const k = this.map[code]; if (k) b |= K[k]; }
    return b;
  }
}

export class GamepadSource implements InputSource {
  constructor(public index: number) {}
  poll(): number {
    const pads = typeof navigator !== 'undefined' && navigator.getGamepads ? navigator.getGamepads() : [];
    const p = pads[this.index];
    if (!p) return 0;
    let b = 0;
    const ax = p.axes[0] ?? 0, ay = p.axes[1] ?? 0;
    const pr = (i: number) => !!p.buttons[i]?.pressed;
    if (ay < -0.5 || pr(12)) b |= K.U;
    if (ay > 0.5 || pr(13)) b |= K.D;
    if (ax < -0.5 || pr(14)) b |= K.L;
    if (ax > 0.5 || pr(15)) b |= K.R;
    // Standard layout: face buttons 0..3, bumpers 4,5, triggers 6,7
    if (pr(2)) b |= K.x; if (pr(3)) b |= K.y; if (pr(5)) b |= K.z;
    if (pr(0)) b |= K.a; if (pr(1)) b |= K.b; if (pr(7)) b |= K.c;
    if (pr(9)) b |= K.s;
    return b;
  }
}

export class MergedSource implements InputSource {
  constructor(public sources: InputSource[]) {}
  poll() { let b = 0; for (const s of this.sources) b |= s.poll(); return b; }
}

/** Virtual (touch) pad writes its bits here. */
export class VirtualSource implements InputSource {
  bits = 0;
  /** Pending one-shot macro (sequence of bitmasks, one per tick) for "simple mode" special buttons */
  macro: number[] = [];
  poll() {
    if (this.macro.length) return this.macro.shift()! | (this.bits & ~15);
    return this.bits;
  }
}
