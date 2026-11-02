import type { AnimFrame } from '../mugen/air';
import type { CharacterDef, Sprite } from './chardef';
import type { Fighter } from './fighter';
import type { Match, Explod, Projectile } from './match';

export interface RenderOptions { debug?: boolean }

interface DrawItem { prio: number; order: number; fn: () => void }

export class Renderer {
  ctx: CanvasRenderingContext2D;
  scale = 3;
  offX = 0; offY = 0;
  constructor(public canvas: HTMLCanvasElement) {
    this.ctx = canvas.getContext('2d')!;
  }

  /** resize the backing store to the element size; game area is 16:9 letterboxed */
  fit(m: Match) {
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = Math.round(this.canvas.clientWidth * dpr), h = Math.round(this.canvas.clientHeight * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) { this.canvas.width = w; this.canvas.height = h; }
    const s = Math.min(w / m.viewW, h / m.viewH);
    this.scale = s;
    this.offX = (w - m.viewW * s) / 2;
    this.offY = (h - m.viewH * s) / 2;
  }

  sx(m: Match, x: number) { return x - m.camera.x + m.viewW / 2; }
  sy(m: Match, y: number) { return y - m.camera.y + m.stage.zoffset; }

  drawSprite(m: Match, def: CharacterDef, fr: AnimFrame | null, x: number, y: number, facing: number, lscl: number, extraScale: [number, number] = [1, 1], opts: { alpha?: number; add?: boolean; filter?: string; angle?: number; vfacing?: number } = {}) {
    if (!fr) return;
    const spr: Sprite | undefined = def.sprites.get(`${fr.group},${fr.index}`);
    if (!spr || !spr.image) return;
    const ctx = this.ctx, S = this.scale;
    const xs = (def.constants.get('size.xscale') ?? 1), ys = (def.constants.get('size.yscale') ?? 1);
    const k = (spr.scale ?? 1) * lscl;
    const sxk = k * xs * extraScale[0] * fr.xscale, syk = k * ys * extraScale[1] * fr.yscale;
    let flipX = facing < 0 ? -1 : 1;
    if (fr.flipH) flipX = -flipX;
    let flipY = fr.flipV ? -1 : 1;
    if (opts.vfacing && opts.vfacing < 0) flipY = -flipY;
    const px = this.sx(m, x) + fr.x * lscl * xs * facing, py = this.sy(m, y) + fr.y * lscl * ys;
    ctx.save();
    ctx.translate(px * S, py * S);
    if (opts.angle) ctx.rotate((-opts.angle * Math.PI / 180) * facing);
    ctx.scale(flipX * sxk * S, flipY * syk * S);
    if (fr.blend === 'add' || fr.blend === 'add1' || opts.add) ctx.globalCompositeOperation = 'lighter';
    if (fr.alpha) ctx.globalAlpha = Math.min(1, fr.alpha[0] / 256);
    if (fr.blend === 'sub') { ctx.globalCompositeOperation = 'difference'; }
    if (opts.alpha !== undefined) ctx.globalAlpha *= opts.alpha;
    if (opts.filter) ctx.filter = opts.filter;
    ctx.drawImage(spr.image, -spr.axisX, -spr.axisY, spr.width, spr.height);
    ctx.restore();
  }

  draw(m: Match, o: RenderOptions = {}) {
    const ctx = this.ctx, S = this.scale;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    let shx = 0, shy = 0;
    if (m.shakeTime > 0) { shy = (m.shakeTime % 2 ? 1 : -1) * m.shakeAmpl * 0.5; shx = (Math.random() - 0.5) * m.shakeAmpl * 0.3; }
    ctx.setTransform(1, 0, 0, 1, this.offX + shx * S, this.offY + shy * S);
    ctx.save();
    ctx.beginPath(); ctx.rect(0, 0, m.viewW * S, m.viewH * S); ctx.clip();
    m.stage.drawBack?.(ctx, m, S);

    // super pause darken
    if (m.pauseState?.darken) {
      ctx.fillStyle = 'rgba(0,0,10,0.62)';
      ctx.fillRect(0, 0, m.viewW * S, m.viewH * S);
    }

    // shadows
    for (const f of m.fighters) {
      if (f.destroyed || f.asserts.has('invisible') || f.asserts.has('noshadow') || (f.isHelper && f.helperType !== 'player')) continue;
      const h = Math.max(0, -f.pos.y);
      const r = 22 * Math.max(0.4, 1 - h / 200) * f.lscl * 1;
      ctx.fillStyle = `rgba(0,0,0,${0.35 * Math.max(0.3, 1 - h / 150)})`;
      ctx.beginPath();
      ctx.ellipse(this.sx(m, f.pos.x) * S, this.sy(m, 0) * S, r * S, r * 0.22 * S, 0, 0, Math.PI * 2);
      ctx.fill();
    }

    const items: DrawItem[] = [];
    let order = 0;
    for (const e of m.explods) if (e.under) items.push({ prio: -100 + e.sprPriority, order: order++, fn: () => this.drawExplod(m, e) });
    for (const f of m.fighters) {
      if (f.destroyed) continue;
      items.push({ prio: f.sprPriority + (f.isHelper ? 0 : 0), order: order++, fn: () => this.drawFighter(m, f) });
    }
    for (const p of m.projectiles) items.push({ prio: p.sprPriority, order: order++, fn: () => this.drawProj(m, p) });
    for (const e of m.explods) if (!e.under) items.push({ prio: e.onTop ? 1000 + e.sprPriority : e.sprPriority, order: order++, fn: () => this.drawExplod(m, e) });
    items.sort((a, b) => a.prio - b.prio || a.order - b.order);
    for (const it of items) it.fn();

    m.stage.drawFront?.(ctx, m, S);

    if (o.debug) this.drawDebug(m);
    if (m.flashState) {
      const c = m.flashState.color;
      ctx.fillStyle = `rgba(${c[0]},${c[1]},${c[2]},${Math.min(0.85, m.flashState.time / 10)})`;
      ctx.fillRect(0, 0, m.viewW * S, m.viewH * S);
    }
    ctx.restore();
  }

  drawFighter(m: Match, f: Fighter) {
    if (f.asserts.has('invisible')) return;
    const fr = f.frame();
    // after images
    if (f.afterImage) {
      const tr = f.afterImage.trail;
      for (let i = tr.length - 1; i >= 1; i -= 2) {
        const t = tr[i];
        this.drawSprite(m, f.animDef, t.frame, t.x, t.y, t.facing, f.lscl, f.drawScale, { alpha: 0.35 * (1 - i / tr.length), add: true, filter: 'saturate(0) brightness(0.8) sepia(1) hue-rotate(180deg) saturate(4)' });
      }
    }
    let x = f.pos.x;
    if (f.hitShake > 0 && f.moveType === 'H') x += (f.hitShake % 2 ? 1.5 : -1.5);
    let filter: string | undefined;
    if (f.palfx) {
      const a = f.palfx.add;
      if (a[0] + a[1] + a[2] > 200) filter = 'brightness(2.2) saturate(0.4)';
      else if (a[0] > a[2] + 40) filter = 'sepia(0.6) saturate(3) hue-rotate(-20deg)';
      else if (a[2] > a[0] + 40) filter = 'sepia(0.6) saturate(3) hue-rotate(180deg)';
    }
    if (f.hitShake > 0 && f.moveType === 'H' && f.hitShake % 4 < 2 && !f.ghv.guarded) filter = 'brightness(1.8)';
    let alpha: number | undefined, add = false;
    if (f.trans) {
      if (f.trans.mode.startsWith('add')) { add = true; alpha = f.trans.src / 256; }
      else if (f.trans.mode === 'addalpha') { add = true; alpha = f.trans.src / 256; }
      f.trans = null;
    }
    this.drawSprite(m, f.animDef, fr, x, f.pos.y, f.facing, f.lscl, f.drawScale, { filter, alpha, add, angle: f.angleDraw ? f.angle : 0 });
    f.angleDraw = false;
  }

  drawExplod(m: Match, e: Explod) {
    this.drawSprite(m, e.def, e.player.frame(), e.pos.x, e.pos.y, e.facing, e.lscl, e.scale, { add: e.trans.startsWith('add'), vfacing: e.vfacing });
  }
  drawProj(m: Match, p: Projectile) {
    this.drawSprite(m, p.def, p.player.frame(), p.pos.x, p.pos.y, p.facing, p.lscl, p.scale);
  }

  drawDebug(m: Match) {
    const ctx = this.ctx, S = this.scale;
    const box = (b: number[], color: string) => {
      ctx.strokeStyle = color; ctx.lineWidth = 1;
      ctx.strokeRect(this.sx(m, b[0]) * S, this.sy(m, b[1]) * S, (b[2] - b[0]) * S, (b[3] - b[1]) * S);
    };
    for (const f of m.fighters) {
      for (const b of f.boxes(2)) box(b, '#3af');
      for (const b of f.boxes(1)) box(b, '#f33');
      ctx.fillStyle = '#ff0';
      ctx.font = `${4 * S}px monospace`;
      ctx.fillText(`${f.stateNo} ${f.stateType}${f.moveType} a${f.animNo} t${f.stateTime}`, (this.sx(m, f.pos.x) - 20) * S, (this.sy(m, f.pos.y) + 8) * S);
    }
    for (const p of m.projectiles) for (const b of m.projBoxes(p, 1)) box(b, '#f80');
  }
}
