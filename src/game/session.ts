import { Match } from '../engine/match';
import { Renderer } from '../engine/render';
import { Hud } from '../ui/hud';
import { commonDef } from '../engine/common';
import { buildFx } from '../engine/fx';
import { AIController } from '../engine/ai';
import type { InputSource } from '../engine/input';
import type { WebAudioSink } from '../engine/audio';
import type { CharEntry, StageEntry } from './registry';

export interface SessionConfig {
  p1: CharEntry; p2: CharEntry; stage: StageEntry;
  p1Input: InputSource | null; p2Input: InputSource | null; // null = CPU
  aiLevel: number;
  training?: boolean;
  rounds?: number;
}

let fxBank: ReturnType<typeof buildFx> | null = null;

/** A running fight: fixed-step simulation at 60 Hz + rendering. */
export class FightSession {
  match: Match;
  renderer: Renderer;
  hud = new Hud();
  paused = false;
  debug = false;
  speed = 1;
  private acc = 0;
  private last = 0;
  private raf = 0;
  onMatchOver?: (winnerSide: number) => void;
  onFrame?: () => void;

  constructor(public canvas: HTMLCanvasElement, public cfg: SessionConfig, public audio: WebAudioSink | null) {
    fxBank ??= buildFx();
    const stage = cfg.stage.build();
    this.match = new Match(commonDef(), fxBank, stage);
    this.match.audio = audio;
    this.match.training = !!cfg.training;
    if (cfg.rounds) this.match.roundsToWin = cfg.rounds;
    const d1 = cfg.p1.load(1);
    const d2 = cfg.p2.load(cfg.p1.id === cfg.p2.id ? 2 : 1);
    const ai1 = cfg.p1Input ? null : new AIController(cfg.aiLevel);
    const ai2 = cfg.p2Input ? null : new AIController(cfg.training ? 0 : cfg.aiLevel);
    const f1 = this.match.addPlayer(d1, 0, cfg.p1Input ?? ai1, ai1 ? cfg.aiLevel : 0);
    const f2 = this.match.addPlayer(d2, 1, cfg.p2Input ?? (cfg.training ? { poll: () => 0 } : ai2), ai2 ? cfg.aiLevel : 0, cfg.p1.id === cfg.p2.id ? 2 : 1);
    ai1?.attach(f1); ai2?.attach(f2);
    this.renderer = new Renderer(canvas);
    this.match.onEvent = (t, d) => {
      if (t === 'ko') audio?.synth('ko');
      if (t === 'matchover') setTimeout(() => this.onMatchOver?.(d), 600);
    };
    this.match.startRound();
    audio?.startMusic(parseInt(stage.music ?? '0') || 0);
  }

  start() {
    this.last = performance.now();
    const loop = (now: number) => {
      this.raf = requestAnimationFrame(loop);
      const dt = Math.min(100, now - this.last);
      this.last = now;
      if (!this.paused) {
        this.acc += dt * this.speed;
        const step = 1000 / 60;
        let n = 0;
        while (this.acc >= step && n < 5) { this.match.tick(); this.acc -= step; n++; }
        if (n === 5) this.acc = 0;
      }
      this.render();
      this.onFrame?.();
    };
    this.raf = requestAnimationFrame(loop);
  }
  /** advance N ticks synchronously (used by tests/screenshots) */
  advance(n: number) { for (let i = 0; i < n; i++) this.match.tick(); this.render(); }
  render() {
    this.renderer.fit(this.match);
    this.renderer.draw(this.match, { debug: this.debug });
    this.hud.draw(this.renderer.ctx, this.match, this.renderer.scale, this.renderer.offX, this.renderer.offY);
  }
  stop() { cancelAnimationFrame(this.raf); this.audio?.stopMusic(); }
}
