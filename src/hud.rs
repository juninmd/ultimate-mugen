//! In-fight HUD drawn in game-view units (426.67 x 240) using the renderer's transform.
use crate::render::Gfx;
use crate::ui::{alpha, hex, Align, Ui};
use macroquad::prelude::*;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

pub struct Hud { trail: [f32; 2], combo: [(i32, f32, i32); 2], pop: [f32; 2], last_hits: [i32; 2] }

impl Hud {
    pub fn new() -> Self { Hud { trail: [1.0, 1.0], combo: [(0, 0.0, 0); 2], pop: [0.0; 2], last_hits: [0; 2] } }

    pub fn draw(&mut self, ui: &Ui, g: &Gfx, m: &Match, gt: f32) {
        let s = g.view_scale;
        let p = |x: f32, y: f32| vec2(g.ox + x * s, g.oy + y * s);
        let roots: Vec<&Fighter> = m.roots().collect();
        let w = m.view_w;
        let text = |t: &str, x: f32, y: f32, size: f32, c: Color, al: Align, disp: bool| {
            let font = if disp { &ui.display } else { &ui.body };
            let fs = (size * s).max(4.0) as u16;
            let d = measure_text(t, Some(font), fs, 1.0);
            let px = match al { Align::Left => p(x, y).x, Align::Center => p(x, y).x - d.width / 2.0, Align::Right => p(x, y).x - d.width };
            draw_text_ex(t, px, p(x, y).y, TextParams { font: Some(font), font_size: fs, color: c, ..Default::default() });
        };
        let outlined = |t: &str, x: f32, y: f32, size: f32, c: Color, al: Align| {
            let o = size * 0.07;
            for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (o, o * 1.5)] { text(t, x + dx, y + dy, size, Color::new(0.07, 0.02, 0.12, c.a), al, true); }
            text(t, x, y, size, c, al, true);
        };
        if roots.len() >= 2 {
            for side in 0..2 {
                let f = roots[side];
                let bw = w / 2.0 - 34.0; let bh = 9.0; let y = 12.0;
                let x = if side == 0 { 10.0 } else { w / 2.0 + 24.0 };
                let ratio = (f.life / f.life_max).max(0.0);
                if !(f.move_type == 'H' && f.hit_shake > 0) { self.trail[side] = if self.trail[side] > ratio { (self.trail[side] - 0.006).max(ratio) } else { ratio }; }
                let quad = |xx: f32, ww: f32, yy: f32, hh: f32, c: Color| {
                    let pts = if side == 0 { [p(xx + 4.0, yy), p(xx + ww, yy), p(xx + ww - 4.0, yy + hh), p(xx, yy + hh)] } else { [p(xx, yy), p(xx + ww - 4.0, yy), p(xx + ww, yy + hh), p(xx + 4.0, yy + hh)] };
                    draw_triangle(pts[0], pts[1], pts[2], c); draw_triangle(pts[0], pts[2], pts[3], c);
                };
                quad(x - 2.0, bw + 4.0, y - 2.0, bh + 4.0, Color::new(0.04, 0.04, 0.08, 0.85));
                quad(x, bw, y, bh, hex("#3a0d12"));
                let seg = |r: f32, c: Color| { let ww = bw * r; if ww > 0.5 { if side == 0 { quad(x + bw - ww, ww, y, bh, c) } else { quad(x, ww, y, bh, c) } } };
                seg(self.trail[side], hex("#e8283a"));
                let low = ratio < 0.25;
                seg(ratio, if low { hex("#ff7a2a") } else { hex("#ffc928") });
                // highlight
                let ww = bw * ratio;
                if ww > 1.0 { let hx = if side == 0 { x + bw - ww } else { x }; draw_rectangle(p(hx + 2.0, y + 1.0).x, p(0.0, y + 1.0).y, (ww - 4.0).max(0.0) * s, 2.5 * s, Color::new(1.0, 1.0, 0.85, if low { 0.3 } else { 0.55 })); }
                // low-life alarm: pulsing red glow around the bar
                if low && ratio > 0.0 { let pul = 0.12 + 0.12 * (gt * 11.0).sin().abs(); quad(x - 3.0, bw + 6.0, y - 3.0, bh + 6.0, Color::new(1.0, 0.1, 0.1, pul)); }
                // portrait
                let pc = p(if side == 0 { 11.0 } else { w - 11.0 }, y + 18.0);
                let col = Color::from_rgba(f.def.color[0], f.def.color[1], f.def.color[2], 255);
                draw_circle(pc.x, pc.y, 10.0 * s, col);
                draw_circle_lines(pc.x, pc.y, 10.0 * s, 1.2 * s, WHITE);
                text(&f.def.display_name.chars().next().unwrap_or('?').to_uppercase().to_string(), if side == 0 { 11.0 } else { w - 11.0 }, y + 22.0, 11.0, WHITE, Align::Center, true);
                let nx = if side == 0 { x + 14.0 } else { x + bw - 14.0 };
                let al = if side == 0 { Align::Left } else { Align::Right };
                text(&f.def.display_name.to_uppercase(), nx + 0.6, y + bh + 9.6, 7.0, BLACK, al, true);
                text(&f.def.display_name.to_uppercase(), nx, y + bh + 9.0, 7.0, WHITE, al, true);
                for i in 0..m.rounds_to_win {
                    let won = m.wins[side] > i;
                    let cx = if side == 0 { x + bw - 6.0 - i as f32 * 8.0 } else { x + 6.0 + i as f32 * 8.0 };
                    let c = p(cx, y + bh + 6.0);
                    draw_circle(c.x, c.y, 2.6 * s, if won { hex("#ffcc33") } else { Color::new(0.0, 0.0, 0.0, 0.6) });
                    draw_circle_lines(c.x, c.y, 2.6 * s, 0.5 * s, WHITE);
                }
                // power bar
                let (pw, ph, py) = (90.0, 5.0, m.view_h - 14.0);
                let px = if side == 0 { 12.0 } else { w - 12.0 - pw };
                let lv = (f.power / 1000.0).floor() as i32;
                let frac = (f.power % 1000.0) / 1000.0;
                let colors = [hex("#2fa8ff"), hex("#33e07a"), hex("#ffcc33"), hex("#ff4fd8")];
                draw_rectangle(p(px - 1.0, py - 1.0).x, p(0.0, py - 1.0).y, (pw + 2.0) * s, (ph + 2.0) * s, Color::new(0.0, 0.0, 0.0, 0.7));
                if lv > 0 { draw_rectangle(p(px, py).x, p(0.0, py).y, pw * s, ph * s, alpha(colors[((lv - 1) % 4) as usize], 0.45)); }
                let fw = if lv >= 3 { pw } else { pw * frac };
                let fx = if side == 0 { px } else { px + pw - fw };
                draw_rectangle(p(fx, py).x, p(0.0, py).y, fw * s, ph * s, colors[lv.min(3) as usize]);
                draw_rectangle(p(fx, py).x, p(0.0, py).y, fw * s, 1.5 * s, Color::new(1.0, 1.0, 1.0, 0.6));
                if lv >= 1 && (m.game_time >> 3) % 2 == 1 { draw_rectangle(p(px, py).x, p(0.0, py).y, pw * s, ph * s, Color::new(1.0, 1.0, 1.0, 0.2)); }
                let lab = if lv >= 3 { "MAX".to_string() } else { lv.to_string() };
                let bump = if lv >= 1 { 1.0 + 0.12 * (gt * 8.0).sin().abs() } else { 1.0 };
                outlined(&lab, if side == 0 { px + pw + 11.0 } else { px - 11.0 }, py + 6.0, 10.0 * bump, colors[(lv.max(1) - 1).min(3) as usize], Align::Center);
                if lv >= 3 { let c = p(px + pw / 2.0, py + ph / 2.0); let a = 0.08 + 0.06 * (gt * 9.0).sin().abs(); draw_rectangle(c.x - pw * s * 0.62, c.y - ph * s * 1.6, pw * s * 1.24, ph * s * 3.2, Color::new(1.0, 0.3, 0.85, a)); }
            }
            // timer
            let (cx, cy) = (w / 2.0, 18.0);
            let mut pts = vec![];
            for i in 0..6 { let a = std::f32::consts::PI / 6.0 + i as f32 * std::f32::consts::PI / 3.0; pts.push(p(cx + a.cos() * 13.0, cy + a.sin() * 13.0)); }
            let c = p(cx, cy);
            for i in 0..6 { draw_triangle(c, pts[i], pts[(i + 1) % 6], hex("#161a2e")); }
            for i in 0..6 { draw_line(pts[i].x, pts[i].y, pts[(i + 1) % 6].x, pts[(i + 1) % 6].y, 1.0 * s, hex("#ffcc33")); }
            let t = if m.training { "--".to_string() } else { format!("{:02}", (m.timer + 59) / 60) };
            let urgent = !m.training && m.timer > 0 && m.timer <= 10 * 60;
            let pulse = if urgent { 1.0 + 0.18 * (1.0 - ((m.timer % 60) as f32 / 60.0)) } else { 1.0 };
            text(&t, cx, cy + 5.0 * pulse, 14.0 * pulse, if urgent { hex("#ff4d4d") } else { WHITE }, Align::Center, true);
            // combos
            for (side, att) in roots.iter().enumerate().take(2) {
                let def = roots[1 - side];
                let c = &mut self.combo[side];
                if def.move_type == 'H' && def.ghv.hitcount >= 2 && def.last_hit_by.is_some_and(|id| m.fighters.iter().any(|f| f.id == id && f.root == att.id)) { *c = (def.ghv.hitcount, att.combo_damage, 70); }
                else if c.2 > 0 { c.2 -= 1; }
                if c.2 <= 0 || c.0 < 2 { self.last_hits[side] = 0; continue; }
                if c.0 > self.last_hits[side] { self.pop[side] = 1.0; self.last_hits[side] = c.0; }
                self.pop[side] = (self.pop[side] - 0.06).max(0.0);
                let a = (c.2 as f32 / 20.0).min(1.0);
                let x = if side == 0 { 18.0 } else { w - 18.0 };
                let al = if side == 0 { Align::Left } else { Align::Right };
                let heat = (c.0 as f32 / 12.0).min(1.0);
                let col = Color::new(1.0, 0.75 - 0.5 * heat, 0.25 - 0.1 * heat, a);
                let sc = 1.0 + 0.55 * self.pop[side] * self.pop[side];
                let ax = if side == 0 { x - 6.0 } else { x + 6.0 };
                for k in 0..3 { let ww = (60.0 - k as f32 * 14.0) * (0.6 + 0.4 * a); let o = if side == 0 { 0.0 } else { -ww }; let a0 = p(ax + o, 58.0 + k as f32 * 3.0); draw_rectangle(a0.x, a0.y, ww * s, 1.2 * s, Color::new(1.0, 0.7, 0.2, a * (0.5 - 0.15 * k as f32))); }
                outlined(&c.0.to_string(), x, 80.0 + 4.0 * (sc - 1.0), 24.0 * sc, col, al);
                outlined("HITS", x, 90.0, 9.0, Color::new(1.0, 1.0, 1.0, a), al);
                outlined(&format!("{} DMG", c.1 as i32), x, 99.0, 6.5, Color::new(1.0, 0.9, 0.6, a), al);
            }
        }
        if let Some(a) = &m.announce {
            let age = (a.total - a.time) as f32;
            let sc = if age < 8.0 { 2.0 - age / 8.0 } else { 1.0 };
            let al = (a.time as f32 / 12.0).min(1.0);
            let size = if a.text.len() > 10 { 24.0 } else { 38.0 } * sc;
            let ko = a.text == "K.O.";
            let glow = if ko { Color::new(1.0, 0.12, 0.25, 0.25 * al) } else { Color::new(1.0, 0.6, 0.0, 0.22 * al) };
            let cc = p(w / 2.0, m.view_h * 0.42);
            for i in 0..7 { let t = 1.0 - i as f32 / 7.0; draw_circle(cc.x, cc.y, size * 1.9 * t * s, Color::new(glow.r, glow.g, glow.b, glow.a * 0.28)); }
            // sweeping accent slashes behind the text
            let sweep = (age / 14.0).min(1.0);
            let hw = w * 0.5 * sweep;
            let yy = m.view_h * 0.42;
            let sl = if ko { Color::new(0.9, 0.1, 0.2, 0.45 * al) } else { Color::new(1.0, 0.55, 0.1, 0.4 * al) };
            for (dy, th) in [(-size * 0.55, 1.6), (size * 0.62, 1.0)] { let a0 = p(w / 2.0 - hw, yy + dy); draw_rectangle(a0.x, a0.y, hw * 2.0 * s, th * s, sl); }
            let band = p(w / 2.0 - hw, yy - size * 0.5); draw_rectangle(band.x, band.y, hw * 2.0 * s, size * 1.05 * s, Color::new(0.04, 0.0, 0.1, 0.35 * al));
            let o = size * 0.08;
            for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (-o, -o), (o, o), (-o, o), (o, -o), (o, o * 2.0)] { text(&a.text, w / 2.0 + dx, m.view_h * 0.42 + size * 0.35 + dy, size, Color::new(0.07, 0.02, 0.12, al), Align::Center, true); }
            let col = if ko { Color::new(1.0, 0.25, 0.25, al) } else { Color::new(1.0, 0.82, 0.25, al) };
            text(&a.text, w / 2.0, m.view_h * 0.42 + size * 0.35, size, col, Align::Center, true);
        }
        if m.training { text("TREINO", w / 2.0, 46.0, 7.0, Color::new(1.0, 1.0, 1.0, 0.8), Align::Center, true); }
        let _ = gt;
    }
}
