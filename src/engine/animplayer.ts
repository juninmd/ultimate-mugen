import type { Animation, AnimFrame } from '../mugen/air';

/** Lightweight animation cursor used by explods/projectiles. */
export class AnimPlayer {
  elem = 0; elemTime = 0; elapsed = 0;
  constructor(public anim: Animation) {}
  frame(): AnimFrame | null { return this.anim.frames[this.elem] ?? null; }
  /** returns true when a non-looping pass completed (time >= total) */
  done(): boolean { return this.anim.totalTime >= 0 && this.elapsed >= this.anim.totalTime; }
  advance() {
    const a = this.anim;
    if (!a.frames.length) return;
    if (a.totalTime >= 0 && this.elapsed >= a.totalTime) {
      // restart loop
      let t = 0;
      for (let i = 0; i < a.loopStart; i++) t += Math.max(0, a.frames[i].time);
      this.elapsed = t;
    }
    const fr = a.frames[this.elem];
    this.elapsed++;
    if (fr.time < 0) return;
    this.elemTime++;
    if (this.elemTime >= fr.time) {
      this.elemTime = 0;
      this.elem++;
      if (this.elem >= a.frames.length) this.elem = a.loopStart;
    }
  }
}
