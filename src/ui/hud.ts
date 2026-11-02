import type { Match } from '../engine/match';
import type { Fighter } from '../engine/fighter';

const FONT = '"Russo One", "Arial Black", Impact, system-ui, sans-serif';

export class Hud {
  trail = [1, 1];
  combo: { side: number; count: number; dmg: number; time: number }[] = [{ side: 0, count: 0, dmg: 0, time: 0 }, { side: 1, count: 0, dmg: 0, time: 0 }];

  draw(ctx: CanvasRenderingContext2D, m: Match, S: number, offX: number, offY: number) {
    ctx.save();
    ctx.setTransform(S, 0, 0, S, offX, offY);
    const W = m.viewW;
    const [p1, p2] = m.roots;
    if (p1 && p2) {
      this.bar(ctx, p1, 0, W, m);
      this.bar(ctx, p2, 1, W, m);
      this.timer(ctx, m, W);
      this.power(ctx, p1, 0, W, m);
      this.power(ctx, p2, 1, W, m);
      this.combos(ctx, m, W);
    }
    if (m.announce) this.announce(ctx, m, W);
    if (m.training) {
      ctx.font = `bold 7px ${FONT}`;
      ctx.fillStyle = 'rgba(255,255,255,0.8)';
      ctx.textAlign = 'center';
      ctx.fillText('TREINO', W / 2, 46);
    }
    ctx.restore();
  }

  bar(ctx: CanvasRenderingContext2D, f: Fighter, side: number, W: number, m: Match) {
    const bw = W / 2 - 34, bh = 9, y = 12;
    const x = side === 0 ? 10 : W / 2 + 24;
    const ratio = Math.max(0, f.life / f.lifeMax);
    // trail
    const tr = this.trail[side];
    if (f.moveType === 'H' && f.hitShake > 0) { /* hold trail during combo */ }
    else this.trail[side] = tr > ratio ? Math.max(ratio, tr - 0.006) : ratio;
    ctx.save();
    // frame
    const skew = side === 0 ? 1 : -1;
    const poly = (xx: number, ww: number, hh: number) => {
      ctx.beginPath();
      if (side === 0) { ctx.moveTo(xx + 4, y); ctx.lineTo(xx + ww, y); ctx.lineTo(xx + ww - 4, y + hh); ctx.lineTo(xx, y + hh); }
      else { ctx.moveTo(xx, y); ctx.lineTo(xx + ww - 4, y); ctx.lineTo(xx + ww, y + hh); ctx.lineTo(xx + 4, y + hh); }
      ctx.closePath();
    };
    ctx.fillStyle = 'rgba(10,10,20,0.85)';
    poly(x - 2, bw + 4, bh + 4); ctx.translate(0, -2); ctx.fill(); ctx.translate(0, 2);
    ctx.save();
    poly(x, bw, bh); ctx.clip();
    ctx.fillStyle = '#3a0d12';
    ctx.fillRect(x, y, bw, bh);
    const tw = bw * this.trail[side];
    ctx.fillStyle = '#e8283a';
    if (side === 0) ctx.fillRect(x + bw - tw, y, tw, bh); else ctx.fillRect(x, y, tw, bh);
    const lw = bw * ratio;
    const g = ctx.createLinearGradient(0, y, 0, y + bh);
    const low = ratio < 0.25;
    g.addColorStop(0, low ? '#ffd0a0' : '#fff6a8'); g.addColorStop(0.45, low ? '#ff7a2a' : '#ffc928'); g.addColorStop(1, low ? '#c43a00' : '#e08a00');
    ctx.fillStyle = g;
    if (side === 0) ctx.fillRect(x + bw - lw, y, lw, bh); else ctx.fillRect(x, y, lw, bh);
    ctx.fillStyle = 'rgba(255,255,255,0.35)';
    ctx.fillRect(x, y + 1, bw, 1.5);
    ctx.restore();
    ctx.strokeStyle = 'rgba(255,255,255,0.75)'; ctx.lineWidth = 0.7;
    poly(x, bw, bh); ctx.stroke();
    // portrait
    const px = side === 0 ? 4 : W - 4, py = y + 4.5;
    ctx.beginPath(); ctx.arc(px + (side === 0 ? 7 : -7), py + 14, 9, 0, Math.PI * 2);
    ctx.fillStyle = f.def.color ?? '#333'; ctx.fill();
    ctx.save(); ctx.clip();
    const p = f.def.portrait ?? f.def.sprites.get('9000,0');
    if (p && p.image) {
      const ps = p.scale ?? 1;
      const big = Math.max(p.width, p.height) * ps;
      if (f.def.source === 'native') {
        const k = 40 / big;
        ctx.save();
        ctx.translate(px + (side === 0 ? 7 : -7), py + 14 + 30);
        ctx.scale((side === 0 ? 1 : -1) * k * ps, k * ps);
        ctx.drawImage(p.image, -p.axisX, -p.axisY - 10 / (k * ps) * 0, p.width, p.height);
        ctx.restore();
      } else {
        const k = 18 / Math.max(p.width, p.height);
        ctx.drawImage(p.image, px + (side === 0 ? 7 : -7) - p.width * k / 2, py + 14 - p.height * k / 2, p.width * k, p.height * k);
      }
    }
    ctx.restore();
    ctx.strokeStyle = '#fff'; ctx.lineWidth = 0.8;
    ctx.beginPath(); ctx.arc(px + (side === 0 ? 7 : -7), py + 14, 9, 0, Math.PI * 2); ctx.stroke();
    // name
    ctx.font = `7px ${FONT}`;
    ctx.textAlign = side === 0 ? 'left' : 'right';
    ctx.fillStyle = '#000';
    const nx = side === 0 ? x + 12 : x + bw - 12;
    ctx.fillText(f.def.displayName.toUpperCase(), nx + 0.6, y + bh + 9.6);
    ctx.fillStyle = '#fff';
    ctx.fillText(f.def.displayName.toUpperCase(), nx, y + bh + 9);
    // round pips
    for (let i = 0; i < m.roundsToWin; i++) {
      const won = m.wins[side] > i;
      const cx = side === 0 ? x + bw - 6 - i * 8 : x + 6 + i * 8;
      ctx.beginPath(); ctx.arc(cx, y + bh + 6, 2.6, 0, Math.PI * 2);
      ctx.fillStyle = won ? '#ffcc33' : 'rgba(0,0,0,0.6)'; ctx.fill();
      ctx.strokeStyle = '#fff'; ctx.lineWidth = 0.5; ctx.stroke();
    }
    ctx.restore();
    void skew;
  }

  timer(ctx: CanvasRenderingContext2D, m: Match, W: number) {
    const cx = W / 2, cy = 18;
    ctx.save();
    ctx.beginPath();
    for (let i = 0; i < 6; i++) { const a = Math.PI / 6 + i * Math.PI / 3; ctx.lineTo(cx + Math.cos(a) * 13, cy + Math.sin(a) * 13); }
    ctx.closePath();
    const g = ctx.createLinearGradient(0, cy - 13, 0, cy + 13);
    g.addColorStop(0, '#2b2f4a'); g.addColorStop(1, '#0c0e1a');
    ctx.fillStyle = g; ctx.fill();
    ctx.strokeStyle = '#ffcc33'; ctx.lineWidth = 0.9; ctx.stroke();
    ctx.font = `14px ${FONT}`;
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillStyle = '#fff';
    const t = m.training ? '∞' : String(Math.ceil(m.timer / 60)).padStart(2, '0');
    ctx.fillText(t, cx, cy + 1);
    ctx.restore();
  }

  power(ctx: CanvasRenderingContext2D, f: Fighter, side: number, W: number, m: Match) {
    const bw = 90, bh = 5, y = m.viewH - 14;
    const x = side === 0 ? 12 : W - 12 - bw;
    const lv = Math.floor(f.power / 1000);
    const frac = (f.power % 1000) / 1000;
    ctx.save();
    ctx.fillStyle = 'rgba(0,0,0,0.7)';
    ctx.fillRect(x - 1, y - 1, bw + 2, bh + 2);
    const colors = ['#2fa8ff', '#33e07a', '#ffcc33', '#ff4fd8'];
    if (lv > 0) { ctx.fillStyle = colors[(lv - 1) % 4]; ctx.globalAlpha = 0.45; ctx.fillRect(x, y, bw, bh); ctx.globalAlpha = 1; }
    const pw = lv >= 3 ? bw : bw * frac;
    const g = ctx.createLinearGradient(0, y, 0, y + bh);
    const c = colors[Math.min(lv, 3)];
    g.addColorStop(0, '#fff'); g.addColorStop(0.4, c); g.addColorStop(1, c);
    ctx.fillStyle = g;
    if (side === 0) ctx.fillRect(x, y, pw, bh); else ctx.fillRect(x + bw - pw, y, pw, bh);
    if (lv >= 1 && (m.gameTime >> 3) % 2) { ctx.fillStyle = 'rgba(255,255,255,0.25)'; ctx.fillRect(x, y, bw, bh); }
    ctx.font = `9px ${FONT}`;
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillStyle = '#000';
    const lx = side === 0 ? x + bw + 9 : x - 9;
    ctx.fillText(String(lv), lx + 0.6, y + 3.1);
    ctx.fillStyle = colors[Math.max(0, Math.min(lv, 3) - (lv ? 1 : 0))];
    ctx.fillText(String(lv), lx, y + 2.5);
    ctx.restore();
  }

  combos(ctx: CanvasRenderingContext2D, m: Match, W: number) {
    for (const att of m.roots) {
      const def = m.roots.find((r) => r.side !== att.side);
      if (!def) continue;
      const c = this.combo[att.side];
      if (def.moveType === 'H' && def.ghv.hitcount >= 2 && def.lastHitBy?.root === att) {
        c.count = def.ghv.hitcount; c.dmg = att.comboDamage; c.time = 70;
      } else if (c.time > 0) c.time--;
      if (c.time <= 0 || c.count < 2) continue;
      const x = att.side === 0 ? 18 : W - 18;
      ctx.save();
      ctx.textAlign = att.side === 0 ? 'left' : 'right';
      ctx.globalAlpha = Math.min(1, c.time / 20);
      ctx.font = `22px ${FONT}`;
      ctx.lineWidth = 3; ctx.strokeStyle = '#1a0630';
      ctx.strokeText(String(c.count), x, 78);
      const g = ctx.createLinearGradient(0, 60, 0, 80);
      g.addColorStop(0, '#fff7b0'); g.addColorStop(1, '#ff8a1f');
      ctx.fillStyle = g;
      ctx.fillText(String(c.count), x, 78);
      ctx.font = `8px ${FONT}`;
      ctx.lineWidth = 2;
      const tx = att.side === 0 ? x : x;
      ctx.strokeText('HITS', tx, 88); ctx.fillStyle = '#fff'; ctx.fillText('HITS', tx, 88);
      ctx.font = `6px ${FONT}`;
      ctx.strokeText(`${c.dmg} DMG`, tx, 96); ctx.fillText(`${c.dmg} DMG`, tx, 96);
      ctx.restore();
    }
  }

  announce(ctx: CanvasRenderingContext2D, m: Match, W: number) {
    const a = m.announce!;
    const txt = a.text;
    const age = (txt === 'FIGHT!' ? 50 : txt === 'K.O.' ? 110 : 70) - a.time;
    const sc = age < 8 ? 2 - age / 8 : 1;
    ctx.save();
    ctx.translate(W / 2, m.viewH * 0.42);
    ctx.scale(sc, sc);
    ctx.globalAlpha = Math.min(1, a.time / 12);
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    const size = txt.length > 10 ? 24 : 38;
    ctx.font = `${size}px ${FONT}`;
    ctx.lineJoin = 'round';
    ctx.lineWidth = 7; ctx.strokeStyle = '#12051f';
    ctx.strokeText(txt, 0, 0);
    const g = ctx.createLinearGradient(0, -size / 2, 0, size / 2);
    if (txt === 'K.O.') { g.addColorStop(0, '#ffe0e0'); g.addColorStop(0.5, '#ff3b3b'); g.addColorStop(1, '#8a0010'); }
    else { g.addColorStop(0, '#fffbe0'); g.addColorStop(0.5, '#ffcc33'); g.addColorStop(1, '#ff6a00'); }
    ctx.fillStyle = g;
    ctx.shadowColor = txt === 'K.O.' ? '#ff2040' : '#ffaa00'; ctx.shadowBlur = 12;
    ctx.fillText(txt, 0, 0);
    ctx.restore();
  }
}
