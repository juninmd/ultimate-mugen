import type { Animation, AnimFrame } from '../mugen/air';
import { computeTotal } from '../mugen/air';
import { defaultConstants, type CharacterDef, type Sprite } from './chardef';
import { makeCanvas } from './canvas';

/** Procedurally generated common effects (replaces fightfx.sff): sparks, guard sparks, super flash, dust. */
type Painter = (ctx: CanvasRenderingContext2D, size: number, t: number) => void;

function frameSprite(size: number, t: number, paint: Painter): Sprite {
  const S = 3;
  const c = makeCanvas(size * S, size * S);
  if (c) {
    c.ctx.translate(size * S / 2, size * S / 2);
    c.ctx.scale(S, S);
    paint(c.ctx, size, t);
  }
  return { image: c?.canvas ?? null, width: size * S, height: size * S, axisX: size * S / 2, axisY: size * S / 2, scale: 1 / S };
}

function star(ctx: CanvasRenderingContext2D, n: number, r1: number, r2: number, rot = 0) {
  ctx.beginPath();
  for (let i = 0; i < n * 2; i++) {
    const r = i % 2 ? r2 : r1;
    const a = rot + (i / (n * 2)) * Math.PI * 2;
    ctx.lineTo(Math.cos(a) * r, Math.sin(a) * r);
  }
  ctx.closePath();
}

const spark = (core: string, mid: string, outer: string, scale: number): Painter => (ctx, size, t) => {
  const k = t; // 0..1
  const R = size / 2 * scale;
  ctx.globalCompositeOperation = 'lighter';
  const g = ctx.createRadialGradient(0, 0, 0, 0, 0, R * (0.4 + k * 0.6));
  g.addColorStop(0, core); g.addColorStop(0.35, mid); g.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.fillStyle = g;
  ctx.globalAlpha = 1 - k * 0.7;
  ctx.beginPath(); ctx.arc(0, 0, R * (0.4 + k * 0.6), 0, Math.PI * 2); ctx.fill();
  ctx.fillStyle = mid;
  star(ctx, 8, R * (0.5 + k * 0.5), R * 0.12, k * 0.6);
  ctx.fill();
  ctx.fillStyle = core;
  star(ctx, 4, R * (0.7 + k * 0.4) * (1 - k * 0.3), R * 0.06, 0.4 + k);
  ctx.fill();
  // flying streaks
  ctx.strokeStyle = outer; ctx.lineWidth = 1.2; ctx.lineCap = 'round';
  for (let i = 0; i < 10; i++) {
    const a = i * 2.39996;
    const r0 = R * (0.3 + k * 0.9), r1 = r0 + R * 0.25 * (1 - k);
    ctx.beginPath(); ctx.moveTo(Math.cos(a) * r0, Math.sin(a) * r0); ctx.lineTo(Math.cos(a) * r1, Math.sin(a) * r1); ctx.stroke();
  }
  if (k > 0.2) {
    ctx.globalAlpha = (1 - k) * 0.8;
    ctx.strokeStyle = mid; ctx.lineWidth = 2 * (1 - k) + 0.5;
    ctx.beginPath(); ctx.arc(0, 0, R * k * 1.1, 0, Math.PI * 2); ctx.stroke();
  }
};

const dust: Painter = (ctx, size, t) => {
  for (let i = 0; i < 7; i++) {
    const x = (i - 3) * size * 0.09 * (1 + t * 1.5);
    const r = size * (0.08 + 0.08 * Math.sin(i * 1.7) ** 2) * (0.6 + t);
    ctx.globalAlpha = 0.55 * (1 - t);
    ctx.fillStyle = '#d8c8a8';
    ctx.beginPath(); ctx.arc(x, -r * 0.6 - t * size * 0.08, r, 0, Math.PI * 2); ctx.fill();
  }
};

const superFlash: Painter = (ctx, size, t) => {
  const R = size / 2;
  ctx.globalCompositeOperation = 'lighter';
  const g = ctx.createRadialGradient(0, 0, 0, 0, 0, R * (0.3 + t * 0.7));
  g.addColorStop(0, 'rgba(255,255,255,0.95)'); g.addColorStop(0.3, 'rgba(255,220,120,0.7)'); g.addColorStop(1, 'rgba(255,120,0,0)');
  ctx.globalAlpha = 1 - t * 0.6;
  ctx.fillStyle = g;
  ctx.beginPath(); ctx.arc(0, 0, R, 0, Math.PI * 2); ctx.fill();
  ctx.fillStyle = 'rgba(255,255,220,0.9)';
  star(ctx, 12, R * (0.4 + t * 0.6), R * 0.05, t * 2);
  ctx.fill();
};

export function buildFx(): CharacterDef {
  const sprites = new Map<string, Sprite>();
  const anims = new Map<number, Animation>();
  const add = (no: number, size: number, n: number, ticks: number, p: Painter) => {
    const frames: AnimFrame[] = [];
    for (let i = 0; i < n; i++) {
      sprites.set(`${no},${i}`, frameSprite(size, i / (n - 1 || 1), p));
      frames.push({ group: no, index: i, x: 0, y: 0, time: ticks, flipH: false, flipV: false, blend: 'add', xscale: 1, yscale: 1, angle: 0, clsn1: [], clsn2: [] });
    }
    const a: Animation = { id: no, frames, loopStart: 0, totalTime: 0 };
    computeTotal(a);
    anims.set(no, a);
  };
  add(0, 40, 5, 2, spark('#ffffff', '#ffe08a', '#ffb347', 0.8));
  add(1, 56, 6, 2, spark('#ffffff', '#ffd06a', '#ff8c2a', 0.9));
  add(2, 84, 7, 2, spark('#ffffff', '#ffb84a', '#ff5a1f', 1));
  add(3, 84, 7, 2, spark('#ffffff', '#b8e0ff', '#5aa8ff', 1));
  add(40, 50, 5, 2, spark('#ffffff', '#8fd3ff', '#2d7dff', 0.85));
  add(30, 130, 8, 3, superFlash);
  add(120, 60, 6, 3, dust);
  // aliases for common fightfx numbers
  for (const [from, to] of [[4, 1], [5, 2], [6, 0], [7, 1], [8, 2], [9, 2], [41, 40], [42, 40], [43, 40]]) anims.set(from, anims.get(to)!);
  return {
    id: 'fx', name: 'fx', displayName: 'fx', author: 'Ultimate MUGEN', source: 'native', localW: 320, localH: 240,
    constants: defaultConstants(), sprites, anims, commands: [], states: new Map(), sounds: new Map(),
  };
}
