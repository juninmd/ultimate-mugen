import type { AudioSink } from './match';
import type { Fighter } from './fighter';

/** WebAudio sink: plays MUGEN .snd samples (decoded lazily) or synthesized built-in sounds. */
export class WebAudioSink implements AudioSink {
  ctx: AudioContext | null = null;
  master!: GainNode;
  cache = new WeakMap<object, Map<string, AudioBuffer | 'pending' | 'bad'>>();
  channels = new Map<string, AudioBufferSourceNode>();
  volume = 0.7;
  musicGain: GainNode | null = null;
  music: { stop: () => void } | null = null;

  ensure() {
    if (this.ctx) { if (this.ctx.state === 'suspended') this.ctx.resume(); return this.ctx; }
    const AC = (window as any).AudioContext || (window as any).webkitAudioContext;
    if (!AC) return null;
    this.ctx = new AC();
    this.master = this.ctx!.createGain();
    this.master.gain.value = this.volume;
    this.master.connect(this.ctx!.destination);
    return this.ctx;
  }

  play(f: Fighter | null, group: number, index: number, own: boolean, volume: number, channel: number, builtin?: string) {
    const ctx = this.ensure();
    if (!ctx) return;
    if (own && f && group >= 0) {
      const def = f.def;
      const key = `${group},${index}`;
      const raw = def.sounds.get(key);
      if (raw instanceof Uint8Array) {
        let c = this.cache.get(def);
        if (!c) { c = new Map(); this.cache.set(def, c); }
        const st = c.get(key);
        if (st instanceof AudioBuffer) { this.playBuffer(st, volume, f, channel); return; }
        if (st === 'pending' || st === 'bad') return;
        c.set(key, 'pending');
        const copy = raw.slice().buffer;
        ctx.decodeAudioData(copy).then((b) => { c!.set(key, b); this.playBuffer(b, volume, f, channel); }).catch(() => c!.set(key, 'bad'));
        return;
      }
    }
    // common sounds: map MUGEN common.snd numbering to synth
    let kind = builtin ?? 'hit';
    if (!builtin) {
      if (group === 5) kind = index >= 2 ? 'hitheavy' : 'hit';
      else if (group === 6) kind = 'guard';
      else if (group === 7) kind = 'land';
      else if (group === 52 || group === 40) kind = 'swing';
      else if (!own) kind = 'swing';
      else return;
    }
    this.synth(kind, volume);
  }
  stop(f: Fighter, channel: number) {
    const k = `${f.id}:${channel}`;
    const s = this.channels.get(k);
    if (s) { try { s.stop(); } catch { /* */ } this.channels.delete(k); }
  }
  playBuffer(b: AudioBuffer, volume: number, f: Fighter, channel: number) {
    const ctx = this.ctx!;
    const src = ctx.createBufferSource();
    src.buffer = b;
    const g = ctx.createGain();
    g.gain.value = Math.min(1.5, volume);
    src.connect(g).connect(this.master);
    if (channel >= 0) { this.stop(f, channel); this.channels.set(`${f.id}:${channel}`, src); }
    src.start();
  }

  noise(dur: number) {
    const ctx = this.ctx!;
    const b = ctx.createBuffer(1, Math.ceil(ctx.sampleRate * dur), ctx.sampleRate);
    const d = b.getChannelData(0);
    for (let i = 0; i < d.length; i++) d[i] = (Math.random() * 2 - 1) * (1 - i / d.length) ** 2;
    return b;
  }

  synth(kind: string, vol = 1) {
    const ctx = this.ctx!;
    const t = ctx.currentTime;
    const out = ctx.createGain();
    out.gain.value = vol;
    out.connect(this.master);
    const thump = (f0: number, f1: number, dur: number, g0: number) => {
      const o = ctx.createOscillator(); const g = ctx.createGain();
      o.type = 'sine'; o.frequency.setValueAtTime(f0, t); o.frequency.exponentialRampToValueAtTime(f1, t + dur);
      g.gain.setValueAtTime(g0, t); g.gain.exponentialRampToValueAtTime(0.001, t + dur);
      o.connect(g).connect(out); o.start(t); o.stop(t + dur);
    };
    const burst = (dur: number, freq: number, q: number, g0: number, type: BiquadFilterType = 'bandpass') => {
      const s = ctx.createBufferSource(); s.buffer = this.noise(dur);
      const f = ctx.createBiquadFilter(); f.type = type; f.frequency.value = freq; f.Q.value = q;
      const g = ctx.createGain(); g.gain.setValueAtTime(g0, t); g.gain.exponentialRampToValueAtTime(0.001, t + dur);
      s.connect(f).connect(g).connect(out); s.start(t);
    };
    switch (kind) {
      case 'hit': thump(180, 60, 0.12, 0.9); burst(0.09, 2200, 0.8, 0.8); break;
      case 'hitheavy': thump(140, 40, 0.25, 1.2); burst(0.18, 1400, 0.6, 1.0); burst(0.08, 4000, 1, 0.4); break;
      case 'guard': burst(0.06, 5200, 6, 0.6); thump(900, 500, 0.06, 0.3); break;
      case 'swing': burst(0.12, 1800, 1.5, 0.25, 'bandpass'); break;
      case 'land': thump(90, 40, 0.1, 0.4); break;
      case 'super': {
        const o = ctx.createOscillator(); const g = ctx.createGain();
        o.type = 'sawtooth'; o.frequency.setValueAtTime(120, t); o.frequency.exponentialRampToValueAtTime(900, t + 0.5);
        g.gain.setValueAtTime(0.25, t); g.gain.exponentialRampToValueAtTime(0.001, t + 0.7);
        const f = ctx.createBiquadFilter(); f.type = 'lowpass'; f.frequency.value = 2400;
        o.connect(f).connect(g).connect(out); o.start(t); o.stop(t + 0.7);
        burst(0.6, 3000, 0.5, 0.35);
        break;
      }
      case 'ko': thump(220, 30, 0.8, 1.2); burst(0.5, 800, 0.4, 0.6); break;
      case 'select': thump(880, 660, 0.05, 0.25); break;
      case 'confirm': thump(660, 1320, 0.12, 0.3); break;
    }
  }

  /** Simple procedural battle music loop (drums + bass + arpeggio). */
  startMusic(seed = 0) {
    const ctx = this.ensure();
    if (!ctx) return;
    this.stopMusic();
    const g = ctx.createGain(); g.gain.value = 0.18; g.connect(this.master);
    this.musicGain = g;
    const bpm = 138 + (seed % 3) * 8;
    const step = 60 / bpm / 4;
    const scales = [[0, 3, 5, 7, 10, 12], [0, 2, 3, 7, 8, 12], [0, 3, 5, 6, 7, 10]];
    const sc = scales[seed % scales.length];
    const root = 41 + (seed % 5);
    const midi = (n: number) => 440 * Math.pow(2, (n - 69) / 12);
    const prog = [0, 0, 5, 3];
    let next = ctx.currentTime + 0.1, i = 0, alive = true;
    const tick = () => {
      if (!alive) return;
      while (next < ctx.currentTime + 0.25) {
        const bar = Math.floor(i / 16) % 4, s = i % 16;
        const r = root + prog[bar];
        if (s % 4 === 0) this.drum(next, 'kick', g);
        if (s % 8 === 4) this.drum(next, 'snare', g);
        if (s % 2 === 0) this.drum(next, 'hat', g);
        if (s % 2 === 0) this.tone(next, midi(r - 12 + (s % 8 === 6 ? 12 : 0)), step * 1.8, 'sawtooth', 0.35, g, 500);
        const arp = sc[(s * 3 + bar) % sc.length];
        if (s % 2 === 1 || s % 3 === 0) this.tone(next, midi(r + 12 + arp), step * 0.9, 'square', 0.08, g, 2600);
        next += step; i++;
      }
      setTimeout(tick, 60);
    };
    tick();
    this.music = { stop: () => { alive = false; g.gain.setTargetAtTime(0, ctx.currentTime, 0.2); } };
  }
  stopMusic() { this.music?.stop(); this.music = null; }
  private tone(t: number, f: number, d: number, type: OscillatorType, v: number, out: AudioNode, cut: number) {
    const ctx = this.ctx!;
    const o = ctx.createOscillator(); const g = ctx.createGain(); const fl = ctx.createBiquadFilter();
    o.type = type; o.frequency.value = f; fl.type = 'lowpass'; fl.frequency.value = cut;
    g.gain.setValueAtTime(v, t); g.gain.exponentialRampToValueAtTime(0.001, t + d);
    o.connect(fl).connect(g).connect(out); o.start(t); o.stop(t + d + 0.02);
  }
  private drum(t: number, k: string, out: AudioNode) {
    const ctx = this.ctx!;
    if (k === 'kick') {
      const o = ctx.createOscillator(); const g = ctx.createGain();
      o.frequency.setValueAtTime(150, t); o.frequency.exponentialRampToValueAtTime(40, t + 0.15);
      g.gain.setValueAtTime(1, t); g.gain.exponentialRampToValueAtTime(0.001, t + 0.18);
      o.connect(g).connect(out); o.start(t); o.stop(t + 0.2);
    } else {
      const s = ctx.createBufferSource(); s.buffer = this.noise(k === 'hat' ? 0.04 : 0.15);
      const f = ctx.createBiquadFilter(); f.type = 'highpass'; f.frequency.value = k === 'hat' ? 7000 : 1500;
      const g = ctx.createGain(); g.gain.value = k === 'hat' ? 0.25 : 0.6;
      s.connect(f).connect(g).connect(out); s.start(t);
    }
  }
}
