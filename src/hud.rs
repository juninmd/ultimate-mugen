//! In-fight HUD drawn in game-view units (426.67 x 240) using the renderer's transform.
use crate::render::Gfx;
use crate::ui::{alpha, hex, Align, Ui};
use macroquad::prelude::*;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;
use crate::poly;
use crate::session::TeamInfo;

pub struct Hud { pub replay: bool, pub inputs: Vec<(u32, u16)>, trail: [f32; 2], combo: [(i32, f32, i32); 2], pop: [f32; 2], last_hits: [i32; 2] }

impl Hud {
    pub fn new() -> Self { Hud { replay: false, inputs: vec![], trail: [1.0, 1.0], combo: [(0, 0.0, 0); 2], pop: [0.0; 2], last_hits: [0; 2] } }

    /// Test/demo hook: pretend a combo just finished (shows in the training "last combo" line).
    pub fn inject_last_combo(&mut self, hits: i32, dmg: f32) { self.combo[0] = (hits, dmg, 0); }

    pub fn draw(&mut self, ui: &Ui, g: &mut Gfx, m: &Match, gt: f32, team: Option<&TeamInfo>, human: &[u32]) {
        let s = g.view_scale;
        let (gox, goy) = (g.ox, g.oy);
        let p = |x: f32, y: f32| vec2(gox + x * s, goy + y * s);
        // team leaders (first fighter of each side); partners get compact bars
        // Main bar per side: the fighter you control (side 0) / the front-line fighter still standing (side 1); everyone else gets a portrait tile.
        let main_of = |sd: usize| -> Option<&Fighter> {
            let all: Vec<&Fighter> = m.roots().filter(|f| f.side == sd).collect();
            all.iter().copied().find(|f| human.contains(&f.id) && f.alive).or_else(|| all.iter().copied().find(|f| f.alive)).or_else(|| all.first().copied())
        };
        let roots: Vec<&Fighter> = (0..2).filter_map(main_of).collect();
        let partners: Vec<&Fighter> = m.roots().filter(|f| !roots.iter().any(|r| r.id == f.id)).collect();
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
                let skew = if side == 0 { 4.5 } else { -4.5 };
                let low = ratio < 0.25;
                let gold = hex("#ffcc33");
                // view-space polygon → screen space
                let sp = |pts: Vec<Vec2>| -> Vec<Vec2> { pts.into_iter().map(|q| p(q.x, q.y)).collect() };
                // 1 drop shadow, 2 metallic frame, 3 gold bevel (bright top edge / dark bottom edge), 4 recessed well
                poly::solid(&sp(poly::slanted(x - 1.0, y - 0.2, bw + 6.0, bh + 6.0, skew)), Color::new(0.0, 0.0, 0.03, 0.45));
                let frame = sp(poly::slanted(x - 2.5, y - 2.5, bw + 5.0, bh + 5.0, skew));
                poly::grad_v(&frame, hex("#3b4166"), hex("#0a0c1a"));
                { let (a0, a1, b0, b1) = (frame[0], frame[1], frame[3], frame[2]); draw_line(a0.x, a0.y, a1.x, a1.y, 1.1 * s, gold); draw_line(b0.x, b0.y, b1.x, b1.y, 1.1 * s, shade_gold()); }
                let well = sp(poly::slanted(x - 0.8, y - 0.8, bw + 1.6, bh + 1.6, skew));
                poly::grad_v(&well, hex("#16060b"), hex("#3a0d12"));
                // damage trail (what you just lost), life fill with a vertical gradient, gloss and a hot leading edge
                let fillp = |r: f32| -> Vec<Vec2> { let ww = (bw * r).max(0.0); if side == 0 { poly::slanted(x + bw - ww, y, ww, bh, skew) } else { poly::slanted(x, y, ww, bh, skew) } };
                if self.trail[side] > ratio + 0.002 { poly::grad_v(&sp(fillp(self.trail[side])), hex("#ff8a8a"), hex("#c01428")); }
                if ratio > 0.002 {
                    let (top, bot) = if low { (hex("#ffb066"), hex("#e0501a")) } else { (hex("#fff0a0"), hex("#e8a010")) };
                    let fp = sp(fillp(ratio));
                    poly::grad_v(&fp, top, bot);
                    // gloss: upper half, white fading to nothing
                    let gl = sp(if side == 0 { poly::slanted(x + bw - bw * ratio, y, bw * ratio, bh * 0.5, skew * 0.5) } else { poly::slanted(x, y, bw * ratio, bh * 0.5, skew * 0.5) });
                    poly::grad_v(&gl, Color::new(1.0, 1.0, 1.0, if low { 0.28 } else { 0.5 }), Color::new(1.0, 1.0, 1.0, 0.0));
                    // hot leading edge = the fill's inner/outer end edge
                    let (e0, e1) = if side == 0 { (fp[0], fp[3]) } else { (fp[1], fp[2]) };
                    draw_line(e0.x, e0.y, e1.x, e1.y, 1.0 * s, Color::new(1.0, 1.0, 0.9, 0.9));
                }
                // 10 segment ticks, following the slant of the well
                for i in 1..10 { let t = i as f32 / 10.0; let (t0, t1) = (well[0].lerp(well[1], t), well[3].lerp(well[2], t)); draw_line(t0.x, t0.y, t1.x, t1.y, 0.5 * s, Color::new(0.0, 0.0, 0.0, 0.35)); }
                // low-life alarm: pulsing red glow around the bar
                if low && ratio > 0.0 { let pul = 0.12 + 0.12 * (gt * 11.0).sin().abs(); poly::solid(&sp(poly::slanted(x - 3.5, y - 3.5, bw + 7.0, bh + 7.0, skew)), Color::new(1.0, 0.1, 0.1, pul)); }
                // portrait (octagonal frame)
                let pc = p(if side == 0 { 12.0 } else { w - 12.0 }, y + 19.0);
                portrait(g, f, pc, 11.0 * s, side, s, if low { hex("#ff4d4d") } else if human.contains(&f.id) { gold } else { hex("#d8dcf0") });
                if human.contains(&f.id) { text(if side == 0 { "P1" } else { "P2" }, if side == 0 { 12.0 } else { w - 12.0 }, y + 36.0, 5.5, gold, Align::Center, true); }
                // name plate: slanted polygon fading out from the portrait side, in the character colour
                let nx = if side == 0 { x + 16.0 } else { x + bw - 16.0 };
                let al = if side == 0 { Align::Left } else { Align::Right };
                let cc = Color::from_rgba(f.def.color[0], f.def.color[1], f.def.color[2], 255);
                let plate = sp(poly::slanted(if side == 0 { x + 12.0 } else { x + bw - 150.0 }, y + bh + 2.0, 138.0, 9.0, skew));
                if side == 0 { poly::grad_h(&plate, poly::with_a(poly::shade(cc, 0.55), 0.92), poly::with_a(cc, 0.0)); } else { poly::grad_h(&plate, poly::with_a(cc, 0.0), poly::with_a(poly::shade(cc, 0.55), 0.92)); }
                { let (l0, l1) = (plate[3], plate[2]); draw_line(l0.x, l0.y, l1.x, l1.y, 0.7 * s, poly::with_a(cc, 0.9)); }
                text(&f.def.display_name.to_uppercase(), nx + 0.6, y + bh + 9.6, 7.0, BLACK, al, true);
                text(&f.def.display_name.to_uppercase(), nx, y + bh + 9.0, 7.0, WHITE, al, true);
                // round pips: diamonds
                for i in 0..m.rounds_to_win {
                    let won = m.wins[side] > i;
                    let cx = if side == 0 { x + bw - 6.0 - i as f32 * 9.0 } else { x + 6.0 + i as f32 * 9.0 };
                    let c = p(cx, y + bh + 7.2);
                    let dia = poly::regular(c, 3.4 * s, 4, std::f32::consts::FRAC_PI_2);
                    if won { poly::circle(c, 5.0 * s, Color::new(1.0, 0.8, 0.2, 0.25)); poly::grad_v(&dia, hex("#fff3a0"), hex("#e8a010")); } else { poly::solid(&dia, Color::new(0.02, 0.02, 0.06, 0.8)); }
                    poly::outline(&dia, 0.6 * s, if won { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.45) });
                }
                // super meter: three slanted cells (one per level) + hexagonal level badge
                let (cw, gap, ph, py) = (28.0, 3.0, 5.2, m.view_h - 14.0);
                let pw = cw * 3.0 + gap * 2.0;
                let px = if side == 0 { 12.0 } else { w - 12.0 - pw };
                let lvl = (f.power / 1000.0).floor() as i32;
                let colors = [hex("#2fa8ff"), hex("#33e07a"), hex("#ffcc33"), hex("#ff4fd8")];
                for i in 0..3 {
                    let cxp = if side == 0 { px + i as f32 * (cw + gap) } else { px + pw - (i as f32 + 1.0) * cw - i as f32 * gap };
                    let col = colors[i.min(3)];
                    poly::grad_v(&sp(poly::slanted(cxp - 0.8, py - 0.8, cw + 1.6, ph + 1.6, 3.0)), hex("#2a2f4a"), hex("#05060e"));
                    poly::solid(&sp(poly::slanted(cxp, py, cw, ph, 3.0)), Color::new(0.0, 0.0, 0.03, 0.85));
                    let fr = if lvl >= 3 { 1.0 } else { (f.power / 1000.0 - i as f32).clamp(0.0, 1.0) };
                    if fr > 0.01 {
                        let fw = cw * fr;
                        let fx = if side == 0 { cxp } else { cxp + cw - fw };
                        let cell = sp(poly::slanted(fx, py, fw, ph, 3.0));
                        poly::grad_v(&cell, poly::shade(col, 1.35), poly::shade(col, 0.7));
                        let gl = sp(poly::slanted(fx, py, fw, ph * 0.45, 3.0 * 0.45));
                        poly::grad_v(&gl, Color::new(1.0, 1.0, 1.0, 0.55), Color::new(1.0, 1.0, 1.0, 0.0));
                        if fr >= 0.999 && (m.game_time >> 3) % 2 == 1 { poly::solid(&cell, Color::new(1.0, 1.0, 1.0, 0.25)); }
                    }
                }
                let lab = if lvl >= 3 { "MAX".to_string() } else { lvl.to_string() };
                let bump = if lvl >= 1 { 1.0 + 0.1 * (gt * 8.0).sin().abs() } else { 1.0 };
                let bc = p(if side == 0 { px + pw + 9.5 } else { px - 9.5 }, py + ph / 2.0);
                let bcol = colors[(lvl.max(1) - 1).min(3) as usize];
                let hexa = poly::regular(bc, 6.6 * s * bump, 6, 0.0);
                poly::grad_v(&hexa, poly::shade(bcol, 1.2), poly::shade(bcol, 0.45));
                poly::outline(&hexa, 0.8 * s, WHITE);
                text(&lab, if side == 0 { px + pw + 9.5 } else { px - 9.5 }, py + ph / 2.0 + if lvl >= 3 { 1.6 } else { 2.2 }, if lvl >= 3 { 4.6 } else { 7.0 }, WHITE, Align::Center, true);
                if lvl >= 3 { let a = 0.1 + 0.08 * (gt * 9.0).sin().abs(); poly::solid(&sp(poly::slanted(px - 3.0, py - 3.0, pw + 6.0, ph + 6.0, 3.0)), Color::new(1.0, 0.3, 0.85, a)); }
            }
            // partner tiles: portrait + life bar for every other fighter on the team; yours is highlighted, fallen ones greyed out
            let mut col_idx = [0usize; 2];
            for f in &partners {
                let side = f.side.min(1);
                let j = col_idx[side]; col_idx[side] += 1;
                let (tw, gap) = (19.0, 3.0);
                let tx = if side == 0 { 10.0 + j as f32 * (tw + gap) } else { w - 10.0 - tw - j as f32 * (tw + gap) };
                let ty = 35.0;
                let c = p(tx + tw / 2.0, ty + tw / 2.0);
                let ratio = (f.life / f.life_max).clamp(0.0, 1.0);
                let mine = human.contains(&f.id);
                portrait(g, f, c, tw / 2.0 * s, side, s, if mine { hex("#ffcc33") } else if f.alive { Color::new(1.0, 1.0, 1.0, 0.75) } else { Color::new(1.0, 1.0, 1.0, 0.25) });
                if !f.alive {
                    draw_rectangle(c.x - tw / 2.0 * s, c.y - tw / 2.0 * s, tw * s, tw * s, Color::new(0.0, 0.0, 0.05, 0.62));
                    let r = tw * 0.28 * s;
                    draw_line(c.x - r, c.y - r, c.x + r, c.y + r, 2.0 * s, Color::new(1.0, 0.3, 0.3, 0.95));
                    draw_line(c.x - r, c.y + r, c.x + r, c.y - r, 2.0 * s, Color::new(1.0, 0.3, 0.3, 0.95));
                }
                // life bar under the tile
                let by = ty + tw + 1.5;
                draw_rectangle(p(tx, by).x, p(0.0, by).y, tw * s, 2.6 * s, Color::new(0.04, 0.04, 0.08, 0.9));
                draw_rectangle(p(tx, by).x, p(0.0, by).y, tw * ratio * s, 2.6 * s, if !f.alive { hex("#555566") } else if ratio < 0.25 { hex("#ff7a2a") } else { hex("#33d6a0") });
                if mine { text(&f.def.display_name.split_whitespace().next().unwrap_or("").to_uppercase(), tx + tw / 2.0, by + 9.5, 4.8, hex("#ffcc33"), Align::Center, true); }
            }
            // alive counters beside the timer + control-switch hint (parallel team mode)
            if m.team_simul {
                let alive = |sd: usize| m.roots().filter(|f| f.side == sd && f.alive).count();
                let tot = |sd: usize| m.roots().filter(|f| f.side == sd).count();
                outlined(&format!("{}/{}", alive(0), tot(0)), w / 2.0 - 24.0, 40.0, 8.0, hex("#22e8ff"), Align::Center);
                outlined(&format!("{}/{}", alive(1), tot(1)), w / 2.0 + 24.0, 40.0, 8.0, hex("#ff4d6d"), Align::Center);
                if human.len() == 1 && m.roots().filter(|f| f.side == 0 && f.alive).count() > 1 { text("TAB · TROCAR DE LUTADOR", 10.0, m.view_h - 20.0, 5.2, Color::new(1.0, 1.0, 1.0, 0.55), Align::Left, true); }
            }
            // sequence roster: portraits of every team member (KO'd ones greyed out, current one highlighted)
            if let Some(t) = team.filter(|t| !t.simul) {
                for side in 0..2 {
                    for (k, (name, col)) in t.names[side].iter().enumerate() {
                        let ko = t.ko[side].get(k).copied().unwrap_or(false);
                        let cur = t.idx[side] == k;
                        let x = if side == 0 { 10.0 + k as f32 * 15.0 } else { w - 10.0 - 13.0 - k as f32 * 15.0 };
                        let c = p(x + 6.5, 45.5);
                        let col = if ko { Color::from_rgba(70, 70, 80, 255) } else { Color::from_rgba(col[0], col[1], col[2], 255) };
                        draw_circle(c.x, c.y, (if cur { 7.2 } else { 5.6 }) * s, col);
                        draw_circle_lines(c.x, c.y, (if cur { 7.2 } else { 5.6 }) * s, (if cur { 1.4 } else { 0.7 }) * s, if cur { hex("#ffcc33") } else { Color::new(1.0, 1.0, 1.0, 0.6) });
                        if ko { let r = 3.2 * s; draw_line(c.x - r, c.y - r, c.x + r, c.y + r, 1.6 * s, Color::new(1.0, 0.3, 0.3, 0.95)); draw_line(c.x - r, c.y + r, c.x + r, c.y - r, 1.6 * s, Color::new(1.0, 0.3, 0.3, 0.95)); } else { text(name.chars().next().map(|_| &name[..name.chars().next().unwrap().len_utf8()]).unwrap_or("?"), x + 6.5, 47.9, if cur { 7.0 } else { 5.5 }, WHITE, Align::Center, true); }
                    }
                }
            }
            // timer
            let (cx, cy) = (w / 2.0, 18.0);
            let c = p(cx, cy);
            poly::circle(c, 17.0 * s, Color::new(0.0, 0.0, 0.03, 0.35));
            let (outer, inner) = (poly::regular(c, 14.8 * s, 6, std::f32::consts::FRAC_PI_6), poly::regular(c, 12.3 * s, 6, std::f32::consts::FRAC_PI_6));
            poly::grad_v(&outer, hex("#ffe08a"), hex("#a8700a"));
            poly::grad_v(&inner, hex("#2b3358"), hex("#080a18"));
            poly::outline(&inner, 0.6 * s, Color::new(0.0, 0.0, 0.0, 0.6));
            for v in poly::regular(c, 13.5 * s, 6, std::f32::consts::FRAC_PI_6) { poly::circle(v, 0.9 * s, hex("#fff3c0")); }
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
                // modern-fighter proration: the next hit of this combo is worth this much
                if att.def.source == ume_engine::chardef::Source::Native {
                    let pct = (ume_engine::matchs::combo_scale(c.0) * 100.0).round() as i32;
                    let tag = [poly_pt(p, x - if side == 0 { 2.0 } else { 52.0 }, 102.0), poly_pt(p, x + if side == 0 { 52.0 } else { 2.0 }, 102.0), poly_pt(p, x + if side == 0 { 49.0 } else { -1.0 }, 108.5), poly_pt(p, x - if side == 0 { 5.0 } else { 49.0 }, 108.5)];
                    poly::grad_h(&tag, Color::new(0.5, 0.1, 0.1, 0.8 * a), Color::new(0.1, 0.02, 0.05, 0.5 * a));
                    text(&format!("DANO {pct}%"), x + if side == 0 { 1.0 } else { -1.0 }, 107.0, 5.2, Color::new(1.0, 0.8, 0.75, a), al, true);
                }
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
        if self.replay {
            let a = 0.6 + 0.4 * (gt * 3.0).sin().abs();
            draw_circle(p(w / 2.0 - 22.0, 49.0).x, p(0.0, 49.0).y, 2.4 * s, Color::new(1.0, 0.2, 0.25, a));
            text("REPLAY", w / 2.0 + 2.0, 51.5, 7.0, Color::new(1.0, 1.0, 1.0, 0.9), Align::Center, true);
        }
        if m.training {
            text("TREINO", w / 2.0, 46.0, 7.0, Color::new(1.0, 1.0, 1.0, 0.8), Align::Center, true);
            // attack data: last combo (hits + damage) stays on screen
            let (hits, dmg) = (self.combo[0].0, self.combo[0].1);
            if hits >= 2 { text(&format!("ÚLTIMO COMBO  {hits} HITS · {} DMG", dmg as i32), w / 2.0, 56.0, 6.0, hex("#ffcc33"), Align::Center, true); }
            // input history (newest at the bottom): direction arrow, buttons, frames held
            let n = self.inputs.len();
            draw_rectangle(p(6.0, 74.0).x, p(0.0, 74.0).y, 62.0 * s, (n as f32 * 8.2 + 8.0) * s, Color::new(0.02, 0.02, 0.06, 0.55));
            text("INPUTS", 10.0, 80.0, 5.0, Color::new(1.0, 1.0, 1.0, 0.6), Align::Left, true);
            use ume_engine::input::k;
            for (i, (bits, fr)) in self.inputs.iter().enumerate() {
                let y = 86.0 + i as f32 * 8.2;
                let a = 0.45 + 0.55 * (i as f32 + 1.0) / n.max(1) as f32;
                let (dx, dy) = ((bits & k::R != 0) as i32 - (bits & k::L != 0) as i32, (bits & k::D != 0) as i32 - (bits & k::U != 0) as i32);
                let c = p(14.0, y - 2.0);
                if dx != 0 || dy != 0 {
                    let ang = (dy as f32).atan2(dx as f32);
                    let r = 3.0 * s;
                    let (tip, l, rr) = (c + vec2(ang.cos(), ang.sin()) * r, c + vec2((ang + 2.5).cos(), (ang + 2.5).sin()) * r, c + vec2((ang - 2.5).cos(), (ang - 2.5).sin()) * r);
                    draw_triangle(tip, l, rr, Color::new(0.4, 0.9, 1.0, a));
                } else { draw_circle(c.x, c.y, 0.9 * s, Color::new(1.0, 1.0, 1.0, a * 0.5)); }
                let names = [(k::X, "LP"), (k::Y, "HP"), (k::A, "LK"), (k::B, "HK"), (k::D2, "S1"), (k::W, "S2"), (k::E, "S3")];
                let btn: Vec<&str> = names.iter().filter(|(b, _)| bits & b != 0).map(|(_, n)| *n).collect();
                if !btn.is_empty() { text(&btn.join("+"), 22.0, y, 5.0, Color::new(1.0, 0.85, 0.3, a), Align::Left, true); }
                text(&fr.to_string(), 64.0, y, 4.6, Color::new(1.0, 1.0, 1.0, a * 0.55), Align::Right, true);
            }
        }
        let _ = gt;
    }
}

fn shade_gold() -> Color { hex("#7a5208") }

/// Portrait in an octagonal (chamfered) frame: head crop of the idle sprite (M.U.G.E.N.: the 9000,x portrait) on the character colour.
fn portrait(g: &mut Gfx, f: &Fighter, pc: Vec2, half: f32, side: usize, s: f32, frame: Color) {
    let col = Color::from_rgba(f.def.color[0], f.def.color[1], f.def.color[2], 255);
    let (x0, y0, d) = (pc.x - half, pc.y - half, half * 2.0);
    let cut = d * 0.24;
    let dark = Color::new(0.03, 0.04, 0.09, 1.0);
    // backing plate + coloured field
    poly::grad_v(&poly::chamfer(x0 - 2.2 * s, y0 - 2.2 * s, d + 4.4 * s, d + 4.4 * s, cut + 1.6 * s), hex("#4a5078"), hex("#0a0c1a"));
    poly::grad_v(&poly::chamfer(x0, y0, d, d, cut), poly::shade(col, 1.15), poly::shade(col, 0.55));
    let por = f.def.sprites.get(&(9000, 1)).or(f.def.sprites.get(&(9000, 0))).filter(|sp| f.def.source == ume_engine::chardef::Source::Mugen && sp.image.is_some()).map(|sp| (sp.clone(), true))
        .or_else(|| f.def.anims.get(&0).and_then(|a| a.frames.first()).and_then(|fr| f.def.sprites.get(&(fr.group, fr.index))).filter(|sp| sp.image.is_some()).map(|sp| (sp.clone(), false)));
    if let Some((sp, full)) = por {
        let img = sp.image.clone().unwrap();
        let tex = g.texture(&img, sp.scale >= 0.99);
        let (iw, ih) = (img.w as f32, img.h as f32);
        let src = if full { let sd = iw.min(ih); Rect::new((iw - sd) / 2.0, 0.0, sd, sd) } else { let sd = (ih * 0.40).min(iw); let cx = sp.axis_x.clamp(sd / 2.0, (iw - sd / 2.0).max(sd / 2.0)); Rect::new(cx - sd / 2.0, ih * 0.02, sd, sd) };
        draw_texture_ex(&tex, x0, y0, WHITE, DrawTextureParams { dest_size: Some(vec2(d, d)), source: Some(src), flip_x: side == 1 && !full, ..Default::default() });
    }
    // mask the four image corners so the picture follows the chamfer
    let pad = 2.2 * s;
    for (corner, dx, dy) in [(vec2(x0, y0), 1.0, 1.0), (vec2(x0 + d, y0), -1.0, 1.0), (vec2(x0 + d, y0 + d), -1.0, -1.0), (vec2(x0, y0 + d), 1.0, -1.0)] {
        let (a, b, c2) = (corner + vec2(-dx * pad, -dy * pad), corner + vec2(dx * (cut + 0.4 * s), -dy * pad), corner + vec2(-dx * pad, dy * (cut + 0.4 * s)));
        poly::solid(&[a, b, vec2(corner.x + dx * cut, corner.y), vec2(corner.x, corner.y + dy * cut), c2], dark);
    }
    // inner shine along the top edge, then the frame
    poly::outline(&poly::chamfer(x0, y0, d, d, cut), 1.4 * s, frame);
    draw_line(x0 + cut, y0 + 1.0 * s, x0 + d - cut, y0 + 1.0 * s, 0.7 * s, Color::new(1.0, 1.0, 1.0, 0.35));
}

fn poly_pt(p: impl Fn(f32, f32) -> Vec2, x: f32, y: f32) -> Vec2 { p(x, y) }
