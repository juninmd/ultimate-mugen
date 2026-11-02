import type { Match, StageInfo } from '../engine/match';
import { makeCanvas } from '../engine/canvas';

const R = 3; // layer resolution multiplier
function rng(seed: number) { return () => { seed = (seed * 1664525 + 1013904223) >>> 0; return seed / 4294967296; }; }

interface Layer { canvas: CanvasImageSource | null; w: number; h: number; delta: number; dy: number; y: number; add?: boolean }
type Painter = (ctx: CanvasRenderingContext2D, w: number, h: number, r: () => number) => void;

function layer(w: number, h: number, y: number, delta: number, paint: Painter, seed = 1, dy = delta): Layer {
  const c = makeCanvas(w * R, h * R);
  if (c) { c.ctx.scale(R, R); paint(c.ctx, w, h, rng(seed)); }
  return { canvas: c?.canvas ?? null, w, h, delta, dy, y };
}

interface Particle { x: number; y: number; vx: number; vy: number; s: number; a: number; c: string }

export interface StageDef { id: string; name: string; thumb: string; build: () => StageInfo }

function makeStage(name: string, zoffset: number, layers: () => { back: Layer[]; front?: Layer[]; particles?: (m: Match) => Particle[]; tint?: string; music?: number }): StageInfo {
  const L = layers();
  let parts: Particle[] | null = null;
  const drawL = (ctx: CanvasRenderingContext2D, m: Match, S: number, l: Layer) => {
    if (!l.canvas) return;
    const x = m.viewW / 2 - l.w / 2 - m.camera.x * l.delta;
    const y = l.y - m.camera.y * l.dy + (zoffset - 205);
    ctx.save();
    if (l.add) ctx.globalCompositeOperation = 'lighter';
    ctx.drawImage(l.canvas, x * S, y * S, l.w * S, l.h * S);
    ctx.restore();
  };
  const info: StageInfo = {
    name, boundLeft: -230, boundRight: 230, boundHigh: -80, boundLow: 0, p1x: -70, p2x: 70, zoffset, verticalFollow: 0.25, tension: 60,
    music: String(L.music ?? 0),
    drawBack(ctx, m, S) {
      for (const l of L.back) drawL(ctx, m, S, l);
    },
    drawFront(ctx, m, S) {
      for (const l of L.front ?? []) drawL(ctx, m, S, l);
      if (L.particles) {
        if (!parts) parts = L.particles(m);
        ctx.save();
        for (const p of parts) {
          ctx.globalAlpha = p.a;
          ctx.fillStyle = p.c;
          const px = ((p.x - m.camera.x * 0.9) % (m.viewW + 40) + m.viewW + 40) % (m.viewW + 40) - 20;
          ctx.beginPath(); ctx.arc(px * S, p.y * S, p.s * S, 0, Math.PI * 2); ctx.fill();
        }
        ctx.restore();
      }
      if (L.tint) { ctx.fillStyle = L.tint; ctx.fillRect(0, 0, m.viewW * S, m.viewH * S); }
    },
    update(m) {
      if (!parts) return;
      for (const p of parts) {
        p.x += p.vx; p.y += p.vy;
        if (p.y > m.viewH + 5) { p.y = -5; }
        if (p.y < -10) p.y = m.viewH + 5;
      }
    },
  };
  return info;
}

// ------------------------------------------------------------------ painters
function grad(ctx: CanvasRenderingContext2D, y0: number, y1: number, stops: [number, string][]) {
  const g = ctx.createLinearGradient(0, y0, 0, y1);
  for (const [o, c] of stops) g.addColorStop(o, c);
  return g;
}
function mountains(ctx: CanvasRenderingContext2D, w: number, base: number, amp: number, color: string, r: () => number, step = 30) {
  ctx.fillStyle = color;
  ctx.beginPath(); ctx.moveTo(0, base + 100);
  let y = base - amp * r();
  for (let x = 0; x <= w + step; x += step) { y = Math.max(base - amp, Math.min(base, y + (r() - 0.5) * amp * 0.8)); ctx.lineTo(x, y); }
  ctx.lineTo(w, base + 100); ctx.closePath(); ctx.fill();
}
function floorPlanks(ctx: CanvasRenderingContext2D, w: number, h: number, c1: string, c2: string, lines: string) {
  ctx.fillStyle = grad(ctx, 0, h, [[0, c1], [1, c2]]);
  ctx.fillRect(0, 0, w, h);
  ctx.strokeStyle = lines; ctx.lineWidth = 0.6;
  for (let i = 0; i < 12; i++) { const y = Math.pow(i / 12, 1.6) * h; ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke(); }
  for (let x = -w; x < w * 2; x += 28) { ctx.beginPath(); ctx.moveTo(w / 2 + (x - w / 2) * 0.55, 0); ctx.lineTo(w / 2 + (x - w / 2) * 1.6, h); ctx.stroke(); }
}

function dojo(): StageInfo {
  return makeStage('Telhado do Dojo — Pôr do Sol', 205, () => ({
    music: 0,
    back: [
      layer(520, 240, 0, 0.05, (ctx, w, h) => {
        ctx.fillStyle = grad(ctx, 0, h, [[0, '#2a1e5c'], [0.45, '#d4507a'], [0.7, '#ff9a4a'], [1, '#ffd38a']]);
        ctx.fillRect(0, 0, w, h);
        const g = ctx.createRadialGradient(w / 2, 150, 5, w / 2, 150, 70);
        g.addColorStop(0, 'rgba(255,250,210,1)'); g.addColorStop(0.3, 'rgba(255,220,140,0.8)'); g.addColorStop(1, 'rgba(255,160,80,0)');
        ctx.fillStyle = g; ctx.beginPath(); ctx.arc(w / 2, 150, 70, 0, Math.PI * 2); ctx.fill();
        ctx.fillStyle = 'rgba(255,240,200,0.9)'; ctx.beginPath(); ctx.arc(w / 2, 150, 22, 0, Math.PI * 2); ctx.fill();
        for (let i = 0; i < 6; i++) { ctx.fillStyle = 'rgba(120,40,90,0.25)'; ctx.fillRect(0, 60 + i * 14, w, 3); }
      }, 3),
      layer(700, 240, 0, 0.18, (ctx, w, _h, r) => { mountains(ctx, w, 175, 70, '#6a3570', r, 25); mountains(ctx, w, 190, 40, '#4a2558', r, 18); }, 7),
      layer(900, 240, 0, 0.4, (ctx, w, _h, r) => {
        for (let i = 0; i < 9; i++) {
          const x = 40 + i * 100 + r() * 30, by = 200 - r() * 15, bw = 60 + r() * 30;
          ctx.fillStyle = '#2b1736';
          ctx.fillRect(x - bw * 0.35, by - 30, bw * 0.7, 30);
          for (let t = 0; t < 2 + Math.floor(r() * 2); t++) {
            const ty = by - 30 - t * 22, tw = bw * (1 - t * 0.2);
            ctx.beginPath(); ctx.moveTo(x - tw / 2 - 8, ty + 4); ctx.quadraticCurveTo(x - tw / 4, ty - 4, x, ty - 12); ctx.quadraticCurveTo(x + tw / 4, ty - 4, x + tw / 2 + 8, ty + 4); ctx.closePath(); ctx.fill();
            ctx.fillRect(x - tw * 0.3, ty, tw * 0.6, 18);
            ctx.fillStyle = 'rgba(255,190,90,0.6)';
            for (let k = 0; k < 3; k++) ctx.fillRect(x - tw * 0.22 + k * tw * 0.18, ty + 6, 3, 5);
            ctx.fillStyle = '#2b1736';
          }
        }
      }, 11),
      layer(1100, 120, 175, 1, (ctx, w, h) => {
        // roof edge + floor
        ctx.fillStyle = '#3b2420'; ctx.fillRect(0, 22, w, h);
        ctx.save(); ctx.translate(0, 30); floorPlanks(ctx, w, h - 30, '#a8694a', '#5a3022', 'rgba(40,15,10,0.45)'); ctx.restore();
        ctx.fillStyle = '#5c3a2c'; ctx.fillRect(0, 26, w, 4);
        for (let x = 10; x < w; x += 70) {
          ctx.fillStyle = '#2a1712'; ctx.fillRect(x, 0, 5, 28);
          ctx.fillStyle = '#e64632'; ctx.beginPath(); ctx.ellipse(x + 2.5, 8, 6, 8, 0, 0, Math.PI * 2); ctx.fill();
          ctx.fillStyle = 'rgba(255,230,150,0.55)'; ctx.beginPath(); ctx.ellipse(x + 2.5, 8, 3, 5, 0, 0, Math.PI * 2); ctx.fill();
        }
      }, 13, 1),
    ],
    particles: () => { const r = rng(5); return Array.from({ length: 40 }, () => ({ x: r() * 470, y: r() * 240, vx: 0.3 + r() * 0.5, vy: 0.25 + r() * 0.4, s: 0.8 + r() * 1.1, a: 0.6 + r() * 0.4, c: r() < 0.5 ? '#ffc0d8' : '#ff9ec0' })); },
  }));
}

function neon(): StageInfo {
  return makeStage('Neo-Metrópole 2099', 205, () => ({
    music: 1,
    back: [
      layer(520, 240, 0, 0.05, (ctx, w, h, r) => {
        ctx.fillStyle = grad(ctx, 0, h, [[0, '#05030f'], [0.6, '#1d0b3a'], [1, '#4a1460']]); ctx.fillRect(0, 0, w, h);
        for (let i = 0; i < 80; i++) { ctx.fillStyle = `rgba(255,255,255,${r() * 0.6})`; ctx.fillRect(r() * w, r() * 120, 0.7, 0.7); }
        ctx.fillStyle = 'rgba(255,60,200,0.15)'; ctx.beginPath(); ctx.arc(w * 0.7, 60, 30, 0, Math.PI * 2); ctx.fill();
      }, 2),
      layer(760, 240, 0, 0.2, (ctx, w, _h, r) => {
        for (let x = 0; x < w; x += 18 + r() * 14) {
          const bh = 60 + r() * 110, bw = 16 + r() * 18;
          ctx.fillStyle = '#120a26'; ctx.fillRect(x, 200 - bh, bw, bh + 40);
          for (let y = 200 - bh + 4; y < 200; y += 5) for (let xx = x + 2; xx < x + bw - 2; xx += 4) if (r() < 0.3) { ctx.fillStyle = r() < 0.5 ? 'rgba(120,200,255,0.5)' : 'rgba(255,140,220,0.4)'; ctx.fillRect(xx, y, 1.6, 2); }
        }
      }, 4),
      layer(1000, 240, 0, 0.5, (ctx, w, _h, r) => {
        const neonC = ['#ff2fd0', '#22e8ff', '#ffe23a', '#7d5bff', '#3dff8a'];
        for (let x = 0; x < w; x += 40 + r() * 40) {
          const bh = 90 + r() * 90, bw = 30 + r() * 30;
          ctx.fillStyle = '#0b0618'; ctx.fillRect(x, 205 - bh, bw, bh + 40);
          ctx.strokeStyle = 'rgba(80,60,140,0.6)'; ctx.lineWidth = 0.5; ctx.strokeRect(x, 205 - bh, bw, bh + 40);
          if (r() < 0.8) {
            const c = neonC[Math.floor(r() * neonC.length)];
            const sy = 205 - bh + 10 + r() * 40, sw = 8 + r() * 6, sh = 20 + r() * 30;
            ctx.shadowColor = c; ctx.shadowBlur = 8; ctx.strokeStyle = c; ctx.lineWidth = 1.2;
            ctx.strokeRect(x + bw / 2 - sw / 2, sy, sw, sh);
            for (let k = 0; k < 3; k++) { ctx.beginPath(); ctx.moveTo(x + bw / 2 - 2, sy + 4 + k * sh / 3.3); ctx.lineTo(x + bw / 2 + 2, sy + 6 + k * sh / 3.3); ctx.stroke(); }
            ctx.shadowBlur = 0;
          }
        }
      }, 9),
      layer(1100, 120, 175, 1, (ctx, w, h) => {
        ctx.fillStyle = grad(ctx, 30, h, [[0, '#1a1030'], [1, '#05020c']]); ctx.fillRect(0, 30, w, h);
        ctx.fillStyle = '#2a1a48'; ctx.fillRect(0, 28, w, 3);
        // reflections
        const cs = ['rgba(255,47,208,0.25)', 'rgba(34,232,255,0.22)', 'rgba(255,226,58,0.15)'];
        for (let x = 0; x < w; x += 23) { ctx.fillStyle = cs[(x / 23 | 0) % 3]; ctx.fillRect(x, 34, 3 + (x % 7), 40 + (x % 30)); }
        ctx.strokeStyle = 'rgba(255,255,255,0.1)';
        for (let x = 0; x < w; x += 60) { ctx.beginPath(); ctx.moveTo(x, 60); ctx.lineTo(x + 30, 60); ctx.stroke(); }
        // railings
        for (let x = 0; x < w; x += 16) { ctx.fillStyle = '#33255a'; ctx.fillRect(x, 6, 1.5, 22); }
        ctx.fillStyle = '#4c3a80'; ctx.fillRect(0, 6, w, 2);
      }, 12, 1),
    ],
    particles: () => { const r = rng(9); return Array.from({ length: 70 }, () => ({ x: r() * 470, y: r() * 240, vx: -0.6, vy: 5 + r() * 2, s: 0.35, a: 0.35, c: '#9ad8ff' })); },
  }));
}

function bamboo(): StageInfo {
  return makeStage('Bambuzal ao Luar', 205, () => ({
    music: 2,
    back: [
      layer(520, 240, 0, 0.04, (ctx, w, h) => {
        ctx.fillStyle = grad(ctx, 0, h, [[0, '#04101c'], [0.6, '#123a4a'], [1, '#2c6a6a']]); ctx.fillRect(0, 0, w, h);
        const g = ctx.createRadialGradient(w * 0.35, 55, 2, w * 0.35, 55, 60);
        g.addColorStop(0, 'rgba(240,255,250,1)'); g.addColorStop(0.25, 'rgba(200,240,230,0.6)'); g.addColorStop(1, 'rgba(100,180,180,0)');
        ctx.fillStyle = g; ctx.beginPath(); ctx.arc(w * 0.35, 55, 60, 0, Math.PI * 2); ctx.fill();
        ctx.fillStyle = '#f2fffa'; ctx.beginPath(); ctx.arc(w * 0.35, 55, 16, 0, Math.PI * 2); ctx.fill();
      }, 2),
      ...[0.2, 0.45, 0.75].map((d, li) => layer(700 + li * 300, 260, -10, d, (ctx, w, _h, r) => {
        const col = ['#0e3a3a', '#0a2c2a', '#06201c'][li];
        for (let x = 0; x < w; x += 10 + r() * 22) {
          const bw = 3 + li * 2 + r() * 2;
          ctx.fillStyle = col;
          ctx.fillRect(x, 0, bw, 260);
          ctx.fillStyle = 'rgba(160,230,200,0.12)'; ctx.fillRect(x + bw * 0.6, 0, bw * 0.25, 260);
          ctx.fillStyle = col;
          for (let y = 20 + r() * 30; y < 230; y += 26 + r() * 10) { ctx.fillRect(x - 1, y, bw + 2, 1.6); if (r() < 0.4) { ctx.beginPath(); ctx.ellipse(x + bw + 8, y - 3, 9, 2, -0.4, 0, Math.PI * 2); ctx.fill(); } }
        }
      }, 20 + li)),
      layer(1100, 120, 175, 1, (ctx, w, h) => {
        ctx.fillStyle = grad(ctx, 28, h, [[0, '#2e4a3a'], [1, '#0c1a14']]); ctx.fillRect(0, 28, w, h);
        ctx.fillStyle = 'rgba(200,255,240,0.08)'; for (let i = 0; i < 6; i++) ctx.fillRect(0, 30 + i * 3, w, 1);
        ctx.fillStyle = '#1b3328';
        for (let x = 0; x < w; x += 7) { ctx.beginPath(); ctx.moveTo(x, 30); ctx.lineTo(x + 2, 22 - (x % 5)); ctx.lineTo(x + 4, 30); ctx.fill(); }
      }, 5, 1),
    ],
    tint: 'rgba(40,120,140,0.06)',
    particles: () => { const r = rng(3); return Array.from({ length: 30 }, () => ({ x: r() * 470, y: 80 + r() * 140, vx: (r() - 0.5) * 0.3, vy: (r() - 0.5) * 0.2, s: 0.9, a: 0.8, c: '#d9ff7a' })); },
  }));
}

function plains(): StageInfo {
  return makeStage('Planalto Rochoso', 205, () => ({
    music: 3,
    back: [
      layer(520, 240, 0, 0.04, (ctx, w, h, r) => {
        ctx.fillStyle = grad(ctx, 0, h, [[0, '#2c6fd6'], [0.6, '#8cc6f2'], [1, '#e8f2f0']]); ctx.fillRect(0, 0, w, h);
        for (let i = 0; i < 9; i++) { const x = r() * w, y = 30 + r() * 70; ctx.fillStyle = 'rgba(255,255,255,0.8)'; for (let k = 0; k < 5; k++) { ctx.beginPath(); ctx.arc(x + k * 9, y + Math.sin(k) * 3, 8 + r() * 6, 0, Math.PI * 2); ctx.fill(); } }
      }, 8),
      layer(760, 240, 0, 0.15, (ctx, w, _h, r) => { mountains(ctx, w, 170, 60, '#8aa0b8', r, 30); }, 31),
      layer(1000, 240, 0, 0.4, (ctx, w, _h, r) => {
        for (let x = 0; x < w; x += 80 + r() * 90) {
          const mh = 50 + r() * 70, mw = 40 + r() * 50;
          ctx.fillStyle = grad(ctx, 200 - mh, 205, [[0, '#c98a5a'], [1, '#7a4a30']]);
          ctx.beginPath(); ctx.moveTo(x, 205); ctx.lineTo(x + 6, 200 - mh); ctx.lineTo(x + mw - 4, 200 - mh - 4); ctx.lineTo(x + mw, 205); ctx.fill();
          ctx.fillStyle = 'rgba(255,230,190,0.25)'; ctx.fillRect(x + 6, 200 - mh, 6, mh);
          ctx.strokeStyle = 'rgba(80,40,20,0.4)'; for (let y = 205 - mh + 10; y < 205; y += 9) { ctx.beginPath(); ctx.moveTo(x + 3, y); ctx.lineTo(x + mw - 2, y + 1); ctx.stroke(); }
        }
      }, 33),
      layer(1100, 120, 175, 1, (ctx, w, h, r) => {
        ctx.fillStyle = grad(ctx, 28, h, [[0, '#d8b080'], [1, '#8a6040']]); ctx.fillRect(0, 28, w, h);
        for (let i = 0; i < 120; i++) { ctx.fillStyle = `rgba(90,60,40,${0.2 + r() * 0.3})`; ctx.beginPath(); ctx.ellipse(r() * w, 34 + r() * 80, 1 + r() * 4, 0.6 + r(), 0, 0, Math.PI * 2); ctx.fill(); }
        for (let i = 0; i < 4; i++) { const x = r() * w; ctx.strokeStyle = 'rgba(70,40,20,0.5)'; ctx.lineWidth = 0.8; ctx.beginPath(); ctx.ellipse(x, 60 + r() * 30, 30, 6, 0, 0, Math.PI * 2); ctx.stroke(); }
      }, 35, 1),
    ],
    particles: () => { const r = rng(4); return Array.from({ length: 18 }, () => ({ x: r() * 470, y: 150 + r() * 80, vx: 0.8 + r(), vy: -0.05, s: 0.6, a: 0.4, c: '#e8c8a0' })); },
  }));
}

function spirit(): StageInfo {
  return makeStage('Portão dos Espíritos', 205, () => ({
    music: 4,
    back: [
      layer(520, 240, 0, 0.04, (ctx, w, h, r) => {
        ctx.fillStyle = grad(ctx, 0, h, [[0, '#0a1420'], [0.5, '#1d4a52'], [1, '#9ad0c8']]); ctx.fillRect(0, 0, w, h);
        for (let i = 0; i < 50; i++) { ctx.fillStyle = `rgba(180,255,240,${r() * 0.5})`; ctx.beginPath(); ctx.arc(r() * w, r() * 140, r() * 1.2, 0, Math.PI * 2); ctx.fill(); }
      }, 40),
      layer(800, 240, 0, 0.25, (ctx, w) => {
        // giant gate (torii-like but original stone arch)
        const cx = w / 2;
        ctx.fillStyle = '#0d2228';
        ctx.fillRect(cx - 110, 60, 22, 150); ctx.fillRect(cx + 88, 60, 22, 150);
        ctx.beginPath(); ctx.moveTo(cx - 150, 52); ctx.quadraticCurveTo(cx, 30, cx + 150, 52); ctx.lineTo(cx + 140, 64); ctx.quadraticCurveTo(cx, 46, cx - 140, 64); ctx.fill();
        ctx.fillRect(cx - 120, 84, 240, 10);
        const g = ctx.createRadialGradient(cx, 140, 5, cx, 140, 90);
        g.addColorStop(0, 'rgba(200,255,250,0.55)'); g.addColorStop(1, 'rgba(120,220,220,0)');
        ctx.fillStyle = g; ctx.fillRect(cx - 90, 94, 180, 120);
      }, 41),
      layer(1000, 240, 0, 0.5, (ctx, w, _h, r) => {
        for (let x = 20; x < w; x += 60 + r() * 30) {
          ctx.fillStyle = '#16323a'; ctx.fillRect(x, 150, 10, 60); ctx.fillRect(x - 5, 145, 20, 6);
          const g = ctx.createRadialGradient(x + 5, 140, 1, x + 5, 140, 14); g.addColorStop(0, 'rgba(160,255,240,0.9)'); g.addColorStop(1, 'rgba(160,255,240,0)');
          ctx.fillStyle = g; ctx.beginPath(); ctx.arc(x + 5, 140, 14, 0, Math.PI * 2); ctx.fill();
        }
      }, 43),
      layer(1100, 120, 175, 1, (ctx, w, h) => {
        ctx.fillStyle = grad(ctx, 28, h, [[0, '#c8d4d0'], [1, '#5a6a68']]); ctx.fillRect(0, 28, w, h);
        ctx.strokeStyle = 'rgba(40,60,60,0.35)'; ctx.lineWidth = 0.6;
        for (let i = 0; i < 8; i++) { const y = 30 + Math.pow(i / 8, 1.5) * 90; ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke(); }
        for (let x = 0; x < w; x += 40) { ctx.beginPath(); ctx.moveTo(x, 30); ctx.lineTo(w / 2 + (x - w / 2) * 1.8, h); ctx.stroke(); }
      }, 44, 1),
    ],
    particles: () => { const r = rng(6); return Array.from({ length: 35 }, () => ({ x: r() * 470, y: r() * 240, vx: (r() - 0.5) * 0.2, vy: -0.3 - r() * 0.4, s: 0.7 + r(), a: 0.5, c: '#b8fff0' })); },
  }));
}

function volcano(): StageInfo {
  return makeStage('Cratera Vulcânica', 205, () => ({
    music: 5,
    back: [
      layer(520, 240, 0, 0.04, (ctx, w, h) => {
        ctx.fillStyle = grad(ctx, 0, h, [[0, '#120404'], [0.5, '#4a0e08'], [1, '#c8381a']]); ctx.fillRect(0, 0, w, h);
      }, 50),
      layer(760, 240, 0, 0.18, (ctx, w, _h, r) => {
        mountains(ctx, w, 180, 90, '#240806', r, 35);
        const g = ctx.createLinearGradient(0, 100, 0, 200); g.addColorStop(0, 'rgba(255,90,20,0)'); g.addColorStop(1, 'rgba(255,120,30,0.4)');
        ctx.fillStyle = g; ctx.fillRect(0, 100, w, 100);
      }, 51),
      layer(1000, 240, 0, 0.45, (ctx, w, _h, r) => {
        for (let x = 0; x < w; x += 50 + r() * 70) {
          const hh = 30 + r() * 60;
          ctx.fillStyle = '#1a0605'; ctx.beginPath(); ctx.moveTo(x, 210); ctx.lineTo(x + 15, 205 - hh); ctx.lineTo(x + 30, 210); ctx.fill();
          ctx.strokeStyle = '#ff6a1a'; ctx.lineWidth = 0.8; ctx.shadowColor = '#ff4a00'; ctx.shadowBlur = 5;
          ctx.beginPath(); ctx.moveTo(x + 15, 205 - hh + 5); ctx.lineTo(x + 12, 205 - hh / 2); ctx.lineTo(x + 17, 208); ctx.stroke(); ctx.shadowBlur = 0;
        }
      }, 52),
      layer(1100, 120, 175, 1, (ctx, w, h, r) => {
        ctx.fillStyle = grad(ctx, 28, h, [[0, '#3a1610'], [1, '#120504']]); ctx.fillRect(0, 28, w, h);
        ctx.strokeStyle = '#ff7a1a'; ctx.shadowColor = '#ff5000'; ctx.shadowBlur = 6; ctx.lineWidth = 1;
        for (let i = 0; i < 25; i++) { let x = r() * w, y = 34 + r() * 70; ctx.beginPath(); ctx.moveTo(x, y); for (let k = 0; k < 4; k++) { x += 6 + r() * 10; y += (r() - 0.5) * 8; ctx.lineTo(x, y); } ctx.stroke(); }
      }, 53, 1),
    ],
    tint: 'rgba(255,60,0,0.05)',
    particles: () => { const r = rng(8); return Array.from({ length: 45 }, () => ({ x: r() * 470, y: r() * 240, vx: (r() - 0.5) * 0.4, vy: -0.5 - r() * 0.8, s: 0.5 + r() * 0.8, a: 0.9, c: r() < 0.5 ? '#ffb030' : '#ff5a10' })); },
  }));
}

export const STAGES: StageDef[] = [
  { id: 'dojo', name: 'Telhado do Dojo — Pôr do Sol', thumb: 'linear-gradient(#2a1e5c,#d4507a 50%,#ff9a4a)', build: dojo },
  { id: 'neon', name: 'Neo-Metrópole 2099', thumb: 'linear-gradient(#05030f,#1d0b3a 60%,#ff2fd0)', build: neon },
  { id: 'bamboo', name: 'Bambuzal ao Luar', thumb: 'linear-gradient(#04101c,#123a4a 60%,#2c6a6a)', build: bamboo },
  { id: 'plains', name: 'Planalto Rochoso', thumb: 'linear-gradient(#2c6fd6,#8cc6f2 60%,#d8b080)', build: plains },
  { id: 'spirit', name: 'Portão dos Espíritos', thumb: 'linear-gradient(#0a1420,#1d4a52 50%,#9ad0c8)', build: spirit },
  { id: 'volcano', name: 'Cratera Vulcânica', thumb: 'linear-gradient(#120404,#4a0e08 50%,#c8381a)', build: volcano },
];
