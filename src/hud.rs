//! In-fight HUD drawn in game-view units (426.67 x 240) using the renderer's transform.
use crate::gfx::*;
use crate::poly;
use crate::render::Gfx;
use crate::session::TeamInfo;
use crate::ui::{hex, Align, Ui};
use ume_engine::fighter::Fighter;
use ume_engine::flags::{asf, gsf};
use ume_engine::matchs::{Match, BASE_H, BASE_W};

mod palette; mod story_hud; use palette::{poly_pt, side_tint};

pub struct Hud {
    pub replay: bool,
    pub inputs: Vec<(u32, u16)>,
    trail: [f32; 2],
    combo: [(i32, f32, i32); 2],
    pop: [f32; 2],
    last_hits: [i32; 2],
    pub bars: Option<[Option<(f32, f32)>; 2]>,
}

impl Hud {
    pub fn new() -> Self {
        Hud {
            replay: false,
            inputs: vec![],
            trail: [1.0, 1.0],
            combo: [(0, 0.0, 0); 2],
            pop: [0.0; 2],
            last_hits: [0; 2],
            bars: None,
        }
    }

    /// Test/demo hook: pretend a combo just finished (shows in the training "last combo" line).
    pub fn inject_last_combo(&mut self, hits: i32, dmg: f32) {
        self.combo[0] = (hits, dmg, 0);
    }

    pub fn draw(
        &mut self,
        ui: &Ui,
        g: &mut Gfx,
        m: &Match,
        gt: f32,
        team: Option<&TeamInfo>,
        human: &[u32],
        cpu: &[u32],
    ) {
        // the world may be zoomed out (big teams) — the HUD always lays out on the base 426×240 view
        g.fit(BASE_W, BASE_H);
        let s = g.view_scale;
        let (gox, goy) = (g.ox, g.oy);
        let p = |x: f32, y: f32| vec2(gox + x * s, goy + y * s);
        // team leaders (first fighter of each side); partners get compact bars
        // Main bar per side: the fighter you control (side 0) / the front-line fighter still standing (side 1); everyone else gets a portrait tile.
        let main_of = |sd: usize| -> Option<&Fighter> {
            let mut first = None;
            let mut alive_fallback = None;
            for f in m.roots().filter(|f| f.side == sd) {
                if m.team_tag && !f.ikemen.standby {
                    return Some(f);
                }
                if human.contains(&f.id) && f.alive {
                    return Some(f);
                }
                if f.alive && alive_fallback.is_none() {
                    alive_fallback = Some(f);
                }
                if first.is_none() {
                    first = Some(f);
                }
            }
            alive_fallback.or(first)
        };
        let Some(r0) = main_of(0) else { return; };
        let Some(r1) = main_of(1) else { return; };
        let roots = [r0, r1];
        let partners: Vec<&Fighter> = if m.fighters.len() > 2 {
            m.roots()
                .filter(|f| f.id != r0.id && f.id != r1.id && !(m.lanes.active && f.side == 1))
                .collect()
        } else {
            Vec::new()
        };
        let w = BASE_W;
        if m.lanes.active && m.lanes.show_foe_hud { story_hud::draw_boss_bar(ui, g, r1); }
        let text = |t: &str, x: f32, y: f32, size: f32, c: Color, al: Align, disp: bool| {
            let font = if disp { &ui.display } else { &ui.body };
            let fs = (size * s).max(9.0) as u16;
            let d = measure_text(t, Some(font), fs, 1.0);
            let px = match al {
                Align::Left => p(x, y).x,
                Align::Center => p(x, y).x - d.width / 2.0,
                Align::Right => p(x, y).x - d.width,
            };
            draw_text_ex(
                t,
                px,
                p(x, y).y,
                TextParams {
                    font: Some(font),
                    font_size: fs,
                    color: c,
                    ..Default::default()
                },
            );
        };
        let outlined = |t: &str, x: f32, y: f32, size: f32, c: Color, al: Align| {
            let o = size * 0.07;
            for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (o, o * 1.5)] {
                text(
                    t,
                    x + dx,
                    y + dy,
                    size,
                    Color::new(0.07, 0.02, 0.12, c.a),
                    al,
                    true,
                );
            }
            text(t, x, y, size, c, al, true);
        };
        for it in &m.texts {
            let al = match it.align {
                0 => Align::Center,
                a if a < 0 => Align::Right,
                _ => Align::Left,
            };
            let c = Color::from_rgba(it.color[0], it.color[1], it.color[2], 255);
            for (k, line) in it.text.lines().enumerate() {
                text(
                    line,
                    it.pos.x,
                    it.pos.y + k as f32 * 9.0 * it.scale[1],
                    8.0 * it.scale[1],
                    c,
                    al,
                    false,
                );
            }
        }
        let hide_bars = m.global_asserts & gsf::NOBARDISPLAY != 0;
        let show_timer = !hide_bars && m.global_asserts & gsf::NOTIMEDISPLAY == 0;
        let hide_combo = roots.iter().any(|f| f.has_assert(asf::NOCOMBODISPLAY));
        if roots.len() >= 2 && !hide_bars {
            let panel_w = w / 2.0 - 26.0;
            for side in 0..(if m.lanes.active { 1 } else { 2 }) {
                let f = roots[side];
                let show_life = !f.has_assert(asf::NOLIFEBARDISPLAY);
                let show_power = !f.has_assert(asf::NOPOWERBARDISPLAY);
                let show_face = !f.has_assert(asf::NOFACEDISPLAY);
                let show_name = !f.has_assert(asf::NONAMEDISPLAY);
                let show_wins = !f.has_assert(asf::NOWINICONDISPLAY);
                let px0 = if side == 0 { 4.0 } else { w / 2.0 + 22.0 };
                let (py0, ph) = (4.0, 23.0);
                let tint = side_tint(side);
                let (dk, cl) = (
                    Color::new(0.02, 0.04, 0.12, 0.7),
                    Color::new(0.02, 0.04, 0.12, 0.0),
                );
                poly::grad_h(
                    &[
                        p(px0 - 4.0, 0.0),
                        p(px0 + panel_w, 0.0),
                        p(px0 + panel_w, py0 + ph),
                        p(px0 - 4.0, py0 + ph),
                    ],
                    if side == 0 { dk } else { cl },
                    if side == 0 { cl } else { dk },
                );
                let hl = |y: f32| {
                    let (a, b) = (p(px0 + 6.0, y), p(px0 + panel_w - 6.0, y));
                    draw_line(
                        a.x,
                        a.y,
                        b.x,
                        b.y,
                        0.5 * s,
                        Color::new(tint[0], tint[1], tint[2], 0.55),
                    );
                };
                hl(py0 + ph - 0.5);
                let (mut bx, mut bw, by, bh) = if side == 0 {
                    (px0 + 27.0, panel_w - 35.0, py0 + 4.0, 5.5)
                } else {
                    (px0 + 8.0, panel_w - 35.0, py0 + 4.0, 5.5)
                };
                if let Some(Some((x, ww))) = self.bars.as_ref().map(|b| b[side].as_ref().copied()) {
                    bx = x.clamp(0.0, w - 20.0);
                    bw = ww.clamp(20.0, (w - bx - 4.0).max(20.0));
                }
                let ratio = (f.life / f.life_max).max(0.0);
                if !(f.move_type == 'H' && f.hit_shake > 0) {
                    self.trail[side] = if self.trail[side] > ratio {
                        (self.trail[side] - 0.006).max(ratio)
                    } else {
                        ratio
                    };
                }
                let low = ratio < 0.25;
                let cut = bh * 0.42;
                let sp = |pts: Vec<Vec2>| -> Vec<Vec2> {
                    pts.into_iter().map(|q| p(q.x, q.y)).collect()
                };
                let capsule =
                    |x: f32, y: f32, ww: f32, hh: f32| sp(poly::chamfer(x, y, ww, hh, hh * 0.42));
                if show_life {
                    if low && ratio > 0.0 {
                        let pul = 0.10 + 0.12 * (gt * 11.0).sin().abs();
                        poly::solid(
                            &capsule(bx - 2.0, by - 2.0, bw + 4.0, bh + 4.0),
                            Color::new(1.0, 0.1, 0.1, pul),
                        );
                    }
                    poly::solid(
                        &capsule(bx - 0.8, by - 0.8, bw + 1.6, bh + 1.6),
                        Color::new(1.0, 1.0, 1.0, 0.22),
                    );
                    poly::grad_v(
                        &capsule(bx, by, bw, bh),
                        Color::new(0.02, 0.03, 0.08, 0.62),
                        Color::new(0.05, 0.07, 0.14, 0.42),
                    );
                    let fill_rect = |r: f32| -> (f32, f32) {
                        let ww = (bw * r).max(0.0);
                        if side == 0 {
                            (bx + bw - ww, ww)
                        } else {
                            (bx, ww)
                        }
                    };
                    if self.trail[side] > ratio + 0.002 {
                        let (fx, fw) = fill_rect(self.trail[side]);
                        poly::grad_v(
                            &capsule(fx, by, fw, bh),
                            Color::new(1.0, 0.55, 0.55, 0.85),
                            Color::new(0.75, 0.1, 0.2, 0.85),
                        );
                    }
                    if ratio > 0.002 {
                        let (top, bot) = if low {
                            (hex("#ffb066"), hex("#e0501a"))
                        } else {
                            (hex("#fff2a8"), hex("#f0a812"))
                        };
                        let (fx, fw) = fill_rect(ratio);
                        poly::grad_v(&capsule(fx, by, fw, bh), top, bot);
                        poly::grad_v(
                            &sp(poly::chamfer(
                                fx + cut * 0.4,
                                by + 0.7,
                                (fw - cut * 0.8).max(0.0),
                                bh * 0.42,
                                bh * 0.2,
                            )),
                            Color::new(1.0, 1.0, 1.0, if low { 0.32 } else { 0.55 }),
                            Color::new(1.0, 1.0, 1.0, 0.0),
                        );
                        let ex = if side == 0 { fx } else { fx + fw };
                        let (e0, e1) = (p(ex, by + 1.0), p(ex, by + bh - 1.0));
                        draw_line(
                            e0.x,
                            e0.y,
                            e1.x,
                            e1.y,
                            0.8 * s,
                            Color::new(1.0, 1.0, 0.92, 0.45),
                        );
                    }
                    for i in 1..10 {
                        let t = i as f32 / 10.0;
                        let (a, b) = (p(bx + bw * t, by + 1.0), p(bx + bw * t, by + bh - 1.0));
                        draw_line(
                            a.x,
                            a.y,
                            b.x,
                            b.y,
                            0.45 * s,
                            Color::new(0.0, 0.0, 0.0, 0.25),
                        );
                    }
                    let acx = if side == 0 {
                        px0 + 12.0
                    } else {
                        px0 + panel_w - 12.0
                    };
                    let ac = p(acx, py0 + 11.5);
                    if show_face {
                        portrait(
                            g,
                            f,
                            ac,
                            10.0 * s,
                            side,
                            s,
                            if low {
                                hex("#ff4d4d")
                            } else if human.contains(&f.id) {
                                hex("#ffcc33")
                            } else {
                                Color::new(1.0, 1.0, 1.0, 0.85)
                            },
                        );
                    }
                    if let Some((tag, tc)) = side_tag(f.id, side, human, cpu) {
                        let (tw, th) = (13.0, 5.0);
                        let tc_pos = p(acx - tw / 2.0, py0 + ph - 3.0);
                        poly::grad_v(
                            &[
                                tc_pos,
                                tc_pos + vec2(tw * s, 0.0),
                                tc_pos + vec2(tw * s, th * s),
                                tc_pos + vec2(0.0, th * s),
                            ],
                            Color::new(0.04, 0.05, 0.1, 0.9),
                            Color::new(0.1, 0.12, 0.2, 0.9),
                        );
                        text(
                            tag,
                            acx,
                            py0 + ph + 1.4,
                            4.2,
                            if tc { hex("#ffcc33") } else { hex("#c9cde6") },
                            Align::Center,
                            true,
                        );
                    }
                }
                if show_name {
                    let nx = if side == 0 { bx } else { bx + bw };
                    let al = if side == 0 { Align::Left } else { Align::Right };
                    let cc = Color::from_rgba(f.def.color[0], f.def.color[1], f.def.color[2], 255);
                    let name = f.def.display_name.to_uppercase();
                    let nsz = 6.4;
                    text(
                        &name,
                        nx + 0.5,
                        by + bh + 6.6,
                        nsz,
                        Color::new(0.0, 0.0, 0.05, 0.7),
                        al,
                        true,
                    );
                    text(&name, nx, by + bh + 6.1, nsz, WHITE, al, true);
                    let name_w =
                        measure_text(&name, Some(&ui.display), ((nsz * s).max(9.0)) as u16, 1.0)
                            .width
                            / s;
                    let uly = p(0.0, by + bh + 8.0).y;
                    let (ux0, ux1) = if side == 0 {
                        (p(nx, 0.0).x, p(nx + name_w, 0.0).x)
                    } else {
                        (p(nx - name_w, 0.0).x, p(nx, 0.0).x)
                    };
                    poly::grad_h(
                        &[
                            vec2(ux0, uly),
                            vec2(ux1, uly),
                            vec2(ux1, uly + 0.8 * s),
                            vec2(ux0, uly + 0.8 * s),
                        ],
                        if side == 0 {
                            poly::with_a(cc, 0.95)
                        } else {
                            poly::with_a(cc, 0.0)
                        },
                        if side == 0 {
                            poly::with_a(cc, 0.0)
                        } else {
                            poly::with_a(cc, 0.95)
                        },
                    );
                    if f.form > 0 && f.alive {
                        let fc = crate::vfx::Vfx::form_color(f);
                        let fcol = Color::new(fc[0], fc[1], fc[2], 1.0);
                        let pulse = 0.75 + 0.25 * (gt * 6.0).sin();
                        let fnm = f.active_form_name();
                        let label = if fnm.is_empty() {
                            "DESPERTO".to_string()
                        } else {
                            fnm.to_uppercase()
                        };
                        let tw = measure_text(
                            &label,
                            Some(&ui.display),
                            ((4.4 * s).max(9.0)) as u16,
                            1.0,
                        )
                        .width
                            / s
                            + 9.0;
                        let tx = if side == 0 {
                            nx + name_w + 6.0
                        } else {
                            nx - name_w - 6.0 - tw
                        };
                        let ty = by + bh + 2.2;
                        poly::grad_h(
                            &sp(poly::chamfer(tx, ty, tw, 5.2, 2.2)),
                            poly::with_a(poly::shade(fcol, 0.5), 0.9 * pulse),
                            poly::with_a(fcol, 0.9 * pulse),
                        );
                        poly::solid(
                            &poly::regular(
                                p(tx + 3.4, ty + 2.6),
                                1.3 * s,
                                4,
                                std::f32::consts::FRAC_PI_2,
                            ),
                            Color::new(1.0, 1.0, 1.0, pulse),
                        );
                        text(&label, tx + 6.2, ty + 4.2, 4.0, WHITE, Align::Left, true);
                    }
                }
                if show_wins {
                    for i in 0..m.rounds_to_win {
                        let won = m.wins[side] > i;
                        let cx = if side == 0 {
                            bx + bw - 4.0 - i as f32 * 9.0
                        } else {
                            bx + 4.0 + i as f32 * 9.0
                        };
                        let c = p(cx, by + bh + 4.6);
                        let dia = poly::regular(c, 2.6 * s, 4, std::f32::consts::FRAC_PI_2);
                        if won {
                            poly::circle(c, 4.0 * s, Color::new(1.0, 0.8, 0.2, 0.25));
                            poly::grad_v(&dia, hex("#fff3a0"), hex("#e8a010"));
                        } else {
                            poly::solid(&dia, Color::new(0.02, 0.02, 0.06, 0.55));
                        }
                        poly::outline(
                            &dia,
                            0.6 * s,
                            if won {
                                WHITE
                            } else {
                                Color::new(1.0, 1.0, 1.0, 0.5)
                            },
                        );
                    }
                }
                if show_power {
                    let (cw, gap, ph2, py) = (26.0, 3.0, 5.0, BASE_H - 15.0);
                    let pw = cw * 3.0 + gap * 2.0;
                    let pxm = if side == 0 { 14.0 } else { w - 14.0 - pw };
                    let lvl = (f.power / 1000.0).floor() as i32;
                    let colors = [
                        hex("#2fa8ff"),
                        hex("#33e07a"),
                        hex("#ffcc33"),
                        hex("#ff4fd8"),
                    ];
                    let (gx, gw) = if side == 0 {
                        (4.0, pw + 36.0)
                    } else {
                        (w - 4.0 - (pw + 36.0), pw + 36.0)
                    };
                    glass(&p, s, gx, py - 5.0, gw, ph2 + 10.0, 5.0, tint);
                    for i in 0..3 {
                        let cxp = if side == 0 {
                            pxm + i as f32 * (cw + gap)
                        } else {
                            pxm + pw - (i as f32 + 1.0) * cw - i as f32 * gap
                        };
                        let col = colors[i.min(3)];
                        poly::solid(
                            &sp(poly::chamfer(cxp - 0.7, py - 0.7, cw + 1.4, ph2 + 1.4, 2.2)),
                            Color::new(1.0, 1.0, 1.0, 0.2),
                        );
                        poly::solid(
                            &sp(poly::chamfer(cxp, py, cw, ph2, 2.0)),
                            Color::new(0.0, 0.0, 0.05, 0.6),
                        );
                        let fr = if lvl >= 3 {
                            1.0
                        } else {
                            (f.power / 1000.0 - i as f32).clamp(0.0, 1.0)
                        };
                        if fr > 0.01 {
                            let fw = cw * fr;
                            let fx = if side == 0 { cxp } else { cxp + cw - fw };
                            let cell = sp(poly::chamfer(fx, py, fw, ph2, 2.0));
                            poly::grad_v(&cell, poly::shade(col, 1.35), poly::shade(col, 0.7));
                            poly::grad_v(
                                &sp(poly::chamfer(
                                    fx + 0.5,
                                    py + 0.4,
                                    (fw - 1.0).max(0.0),
                                    ph2 * 0.45,
                                    1.0,
                                )),
                                Color::new(1.0, 1.0, 1.0, 0.55),
                                Color::new(1.0, 1.0, 1.0, 0.0),
                            );
                            if fr >= 0.999 && (m.game_time >> 3) % 2 == 1 {
                                poly::solid(&cell, Color::new(1.0, 1.0, 1.0, 0.25));
                            }
                        }
                    }
                    let lab = if lvl >= 3 {
                        "MAX".to_string()
                    } else {
                        lvl.to_string()
                    };
                    let bump = if lvl >= 1 {
                        1.0 + 0.1 * (gt * 8.0).sin().abs()
                    } else {
                        1.0
                    };
                    let bcx = if side == 0 {
                        pxm + pw + 11.0
                    } else {
                        pxm - 11.0
                    };
                    let bc = p(bcx, py + ph2 / 2.0);
                    let bcol = colors[(lvl.max(1) - 1).min(3) as usize];
                    let hexa = poly::regular(bc, 6.8 * s * bump, 6, 0.0);
                    poly::grad_v(&hexa, poly::shade(bcol, 1.2), poly::shade(bcol, 0.45));
                    poly::outline(&hexa, 0.8 * s, WHITE);
                    text(
                        &lab,
                        bcx,
                        py + ph2 / 2.0 + if lvl >= 3 { 1.6 } else { 2.4 },
                        if lvl >= 3 { 4.6 } else { 7.0 },
                        WHITE,
                        Align::Center,
                        true,
                    );
                    if lvl >= 3 {
                        let a = 0.08 + 0.07 * (gt * 9.0).sin().abs();
                        poly::solid(
                            &sp(poly::chamfer(pxm - 2.0, py - 2.0, pw + 4.0, ph2 + 4.0, 3.0)),
                            Color::new(1.0, 0.3, 0.85, a),
                        );
                    }
                }
            }
            let strip_y = 30.5;
            let mut col_idx = [0usize; 2];
            let strip_n = |side: usize| -> usize {
                if let Some(t) = team.filter(|t| !t.simul) {
                    t.names[side].len()
                } else {
                    partners.iter().filter(|f| f.side.min(1) == side).count()
                }
            };
            for side in 0..2 {
                let n = strip_n(side);
                if n == 0 {
                    continue;
                }
                let step = TILE_W + TILE_GAP;
                let sw = n as f32 * step + 5.0;
                let sx = if side == 0 { 4.0 } else { w - 4.0 - sw };
                glass(&p, s, sx, strip_y, sw, 16.0, 5.0, side_tint(side));
            }
            for f in &partners {
                let side = f.side.min(1);
                let j = col_idx[side];
                col_idx[side] += 1;
                let step = TILE_W + TILE_GAP;
                let tx = if side == 0 {
                    4.0 + 2.5 + j as f32 * step
                } else {
                    w - 4.0 - 2.5 - TILE_W - j as f32 * step
                };
                let c = p(tx + TILE_W / 2.0, strip_y + 2.0 + TILE_W / 2.0);
                let ratio = (f.life / f.life_max).clamp(0.0, 1.0);
                let mine = human.contains(&f.id);
                portrait(
                    g,
                    f,
                    c,
                    TILE_W / 2.0 * s,
                    side,
                    s,
                    if mine {
                        hex("#ffcc33")
                    } else if f.alive {
                        Color::new(1.0, 1.0, 1.0, 0.7)
                    } else {
                        Color::new(1.0, 1.0, 1.0, 0.22)
                    },
                );
                if !f.alive {
                    poly::circle(c, TILE_W / 2.0 * s, Color::new(0.0, 0.0, 0.05, 0.62));
                    let r = TILE_W * 0.26 * s;
                    draw_line(
                        c.x - r,
                        c.y - r,
                        c.x + r,
                        c.y + r,
                        1.8 * s,
                        Color::new(1.0, 0.3, 0.3, 0.95),
                    );
                    draw_line(
                        c.x - r,
                        c.y + r,
                        c.x + r,
                        c.y - r,
                        1.8 * s,
                        Color::new(1.0, 0.3, 0.3, 0.95),
                    );
                }
                let by = strip_y + 2.0 + TILE_W + 1.0;
                let bar = p(tx, by);
                poly::solid(
                    &[
                        bar,
                        bar + vec2(TILE_W * s, 0.0),
                        bar + vec2(TILE_W * s, 1.5 * s),
                        bar + vec2(0.0, 1.5 * s),
                    ],
                    Color::new(0.02, 0.03, 0.08, 0.7),
                );
                let fc = if !f.alive {
                    hex("#555566")
                } else if ratio < 0.25 {
                    hex("#ff7a2a")
                } else {
                    hex("#33d6a0")
                };
                poly::solid(
                    &[
                        bar,
                        bar + vec2(TILE_W * ratio * s, 0.0),
                        bar + vec2(TILE_W * ratio * s, 1.5 * s),
                        bar + vec2(0.0, 1.5 * s),
                    ],
                    fc,
                );
                if m.team_tag && f.alive {
                    let cd = m.tag_cooldown[side] as f32 / m.tag_cooldown_max.max(1) as f32;
                    let ready = cd <= 0.0;
                    let tbar = p(tx, by + 2.4);
                    poly::solid(
                        &[
                            tbar,
                            tbar + vec2(TILE_W * s, 0.0),
                            tbar + vec2(TILE_W * s, 1.2 * s),
                            tbar + vec2(0.0, 1.2 * s),
                        ],
                        Color::new(0.02, 0.03, 0.08, 0.7),
                    );
                    let fw = TILE_W * (1.0 - cd.clamp(0.0, 1.0)) * s;
                    poly::solid(
                        &[
                            tbar,
                            tbar + vec2(fw, 0.0),
                            tbar + vec2(fw, 1.2 * s),
                            tbar + vec2(0.0, 1.2 * s),
                        ],
                        if ready {
                            hex("#ffcc33")
                        } else {
                            hex("#4aa3ff")
                        },
                    );
                }
            }
            if let Some(t) = team.filter(|t| !t.simul) {
                for side in 0..2 {
                    let n = t.names[side].len();
                    for (k, (name, col)) in t.names[side].iter().enumerate() {
                        let ko = t.ko[side].get(k).copied().unwrap_or(false);
                        let cur = t.idx[side] == k;
                        let step = TILE_W + TILE_GAP;
                        let x = if side == 0 {
                            4.0 + 2.5 + k as f32 * step
                        } else {
                            w - 4.0 - 2.5 - TILE_W - k as f32 * step
                        };
                        let _ = n;
                        let c = p(x + TILE_W / 2.0, strip_y + 8.0);
                        let col = if ko {
                            Color::from_rgba(70, 70, 80, 255)
                        } else {
                            Color::from_rgba(col[0], col[1], col[2], 255)
                        };
                        let r = (if cur { 5.2 } else { 4.4 }) * s;
                        poly::circle(c, r, poly::with_a(col, 0.85));
                        poly::ring(
                            c,
                            r,
                            (if cur { 1.1 } else { 0.6 }) * s,
                            if cur {
                                hex("#ffcc33")
                            } else {
                                Color::new(1.0, 1.0, 1.0, 0.6)
                            },
                        );
                        if ko {
                            let rr = 2.0 * s;
                            draw_line(
                                c.x - rr,
                                c.y - rr,
                                c.x + rr,
                                c.y + rr,
                                1.6 * s,
                                Color::new(1.0, 0.3, 0.3, 0.95),
                            );
                            draw_line(
                                c.x - rr,
                                c.y + rr,
                                c.x + rr,
                                c.y - rr,
                                1.6 * s,
                                Color::new(1.0, 0.3, 0.3, 0.95),
                            );
                        } else {
                            text(
                                name.chars()
                                    .next()
                                    .map(|_| &name[..name.chars().next().unwrap().len_utf8()])
                                    .unwrap_or("?"),
                                x + TILE_W / 2.0,
                                strip_y + 8.0 + 1.7,
                                if cur { 5.2 } else { 4.4 },
                                WHITE,
                                Align::Center,
                                true,
                            );
                        }
                    }
                }
            }
            if m.team_simul && !m.lanes.active {
                let alive = |sd: usize| m.roots().filter(|f| f.side == sd && f.alive).count();
                let tot = |sd: usize| m.roots().filter(|f| f.side == sd).count();
                outlined(
                    &format!("{}/{}", alive(0), tot(0)),
                    w / 2.0 - 22.0,
                    34.0,
                    6.4,
                    hex("#22e8ff"),
                    Align::Center,
                );
                outlined(
                    &format!("{}/{}", alive(1), tot(1)),
                    w / 2.0 + 22.0,
                    34.0,
                    6.4,
                    hex("#ff4d6d"),
                    Align::Center,
                );
                if human.len() == 1 && m.roots().filter(|f| f.side == 0 && f.alive).count() > 1 {
                    text(
                        "TAB · TROCAR DE LUTADOR",
                        10.0,
                        BASE_H - 24.0,
                        5.2,
                        Color::new(1.0, 1.0, 1.0, 0.6),
                        Align::Left,
                        true,
                    );
                }
            }
            for side in 0..2 {
                let Some(lead) = team_label(m, team, side) else {
                    continue;
                };
                let n = strip_n(side);
                if n == 0 {
                    continue;
                }
                let off = 4.0 + n as f32 * (TILE_W + TILE_GAP) + 8.0;
                let (x, al) = if side == 0 {
                    (off, Align::Left)
                } else {
                    (w - off, Align::Right)
                };
                outlined(
                    &format!("TIME {lead}"),
                    x,
                    41.0,
                    5.2,
                    if side == 0 {
                        hex("#22e8ff")
                    } else {
                        hex("#ff4d6d")
                    },
                    al,
                );
            }
            let (cx, cy) = (w / 2.0, 13.0);
            if show_timer {
                for k in 0..9 {
                    let tx = cx - 8.0 + k as f32 * 2.0;
                    let (a, b) = (p(tx, 21.5), p(tx, 23.0));
                    draw_line(a.x, a.y, b.x, b.y, 0.4 * s, Color::new(1.0, 1.0, 1.0, 0.4));
                }
                let t = if m.training || m.round_ticks == 0 {
                    "--".to_string()
                } else {
                    format!("{:02}", (m.timer + 59) / 60)
                };
                let urgent = !m.training && m.round_ticks > 0 && m.timer > 0 && m.timer <= 10 * 60;
                let pulse = if urgent {
                    1.0 + 0.18 * (1.0 - ((m.timer % 60) as f32 / 60.0))
                } else {
                    1.0
                };
                if urgent {
                    let a = 0.12 + 0.1 * (gt * 9.0).sin().abs();
                    poly::solid(
                        &sp_box(&p, cx - 15.0, 5.0, 30.0, 17.0, 6.0),
                        Color::new(1.0, 0.1, 0.1, a),
                    );
                }
                outlined(
                    &t,
                    cx,
                    cy + 5.2 * pulse,
                    14.0 * pulse,
                    if urgent { hex("#ff5a5a") } else { WHITE },
                    Align::Center,
                );
            }
            let team_rows = !partners.is_empty() || team.is_some();
            let cy0 = if team_rows { -5.0 } else { -15.0 };
            // combos
            for (side, att) in roots
                .iter()
                .enumerate()
                .take(if hide_combo { 0 } else { 2 })
            {
                let def = roots[1 - side];
                let c = &mut self.combo[side];
                if def.move_type == 'H'
                    && def.ghv.hitcount >= 2
                    && def
                        .last_hit_by
                        .is_some_and(|id| m.fighters.iter().any(|f| f.id == id && f.root == att.id))
                {
                    *c = (def.ghv.hitcount, att.combo_damage, 70);
                } else if c.2 > 0 {
                    c.2 -= 1;
                }
                if c.2 <= 0 || c.0 < 2 {
                    self.last_hits[side] = 0;
                    continue;
                }
                if c.0 > self.last_hits[side] {
                    self.pop[side] = 1.0;
                    self.last_hits[side] = c.0;
                }
                self.pop[side] = (self.pop[side] - 0.06).max(0.0);
                let a = (c.2 as f32 / 20.0).min(1.0);
                let x = if side == 0 { 18.0 } else { w - 18.0 };
                let al = if side == 0 { Align::Left } else { Align::Right };
                let heat = (c.0 as f32 / 12.0).min(1.0);
                let col = Color::new(1.0, 0.75 - 0.5 * heat, 0.25 - 0.1 * heat, a);
                let sc = 1.0 + 0.55 * self.pop[side] * self.pop[side];
                let ax = if side == 0 { x - 6.0 } else { x + 6.0 };
                for k in 0..3 {
                    let ww = (60.0 - k as f32 * 14.0) * (0.6 + 0.4 * a);
                    let o = if side == 0 { 0.0 } else { -ww };
                    let a0 = p(ax + o, cy0 + 58.0 + k as f32 * 3.0);
                    draw_rectangle(
                        a0.x,
                        a0.y,
                        ww * s,
                        1.2 * s,
                        Color::new(1.0, 0.7, 0.2, a * (0.5 - 0.15 * k as f32)),
                    );
                }
                outlined(
                    &c.0.to_string(),
                    x,
                    cy0 + 80.0 + 4.0 * (sc - 1.0),
                    24.0 * sc,
                    col,
                    al,
                );
                outlined("HITS", x, cy0 + 90.0, 9.0, Color::new(1.0, 1.0, 1.0, a), al);
                outlined(
                    &format!("{} DMG", c.1 as i32),
                    x,
                    cy0 + 99.0,
                    6.5,
                    Color::new(1.0, 0.9, 0.6, a),
                    al,
                );
                // modern-fighter proration: the next hit of this combo is worth this much
                if att.def.source == ume_engine::chardef::Source::Native {
                    let pct = (ume_engine::matchs::combo_scale(c.0) * 100.0).round() as i32;
                    let tag = [
                        poly_pt(p, x - if side == 0 { 2.0 } else { 52.0 }, cy0 + 102.0),
                        poly_pt(p, x + if side == 0 { 52.0 } else { 2.0 }, cy0 + 102.0),
                        poly_pt(p, x + if side == 0 { 49.0 } else { -1.0 }, cy0 + 108.5),
                        poly_pt(p, x - if side == 0 { 5.0 } else { 49.0 }, cy0 + 108.5),
                    ];
                    poly::grad_h(
                        &tag,
                        Color::new(0.5, 0.1, 0.1, 0.8 * a),
                        Color::new(0.1, 0.02, 0.05, 0.5 * a),
                    );
                    text(
                        &format!("DANO {pct}%"),
                        x + if side == 0 { 1.0 } else { -1.0 },
                        cy0 + 107.0,
                        5.2,
                        Color::new(1.0, 0.8, 0.75, a),
                        al,
                        true,
                    );
                }
            }
        }
        if let Some(a) = &m.announce {
            let age = (a.total - a.time) as f32;
            let sc = if age < 8.0 { 2.0 - age / 8.0 } else { 1.0 };
            let al = (a.time as f32 / 12.0).min(1.0);
            let size = if a.text.len() > 10 { 20.0 } else { 30.0 } * sc;
            let ko = a.text == "K.O.";
            let glow = if ko {
                Color::new(1.0, 0.12, 0.25, 0.25 * al)
            } else {
                Color::new(1.0, 0.6, 0.0, 0.22 * al)
            };
            let cc = p(w / 2.0, BASE_H * 0.42);
            for i in 0..7 {
                let t = 1.0 - i as f32 / 7.0;
                draw_circle(
                    cc.x,
                    cc.y,
                    size * 1.9 * t * s,
                    Color::new(glow.r, glow.g, glow.b, glow.a * 0.14),
                );
            }
            // sweeping accent slashes behind the text
            let sweep = (age / 14.0).min(1.0);
            let hw = w * 0.5 * sweep;
            let yy = BASE_H * 0.42;
            let sl = if ko {
                Color::new(0.9, 0.1, 0.2, 0.45 * al)
            } else {
                Color::new(1.0, 0.55, 0.1, 0.4 * al)
            };
            for (dy, th) in [(-size * 0.55, 1.6), (size * 0.62, 1.0)] {
                let a0 = p(w / 2.0 - hw, yy + dy);
                draw_rectangle(a0.x, a0.y, hw * 2.0 * s, th * s, sl);
            }
            let band = p(w / 2.0 - hw, yy - size * 0.5);
            draw_rectangle(
                band.x,
                band.y,
                hw * 2.0 * s,
                size * 1.05 * s,
                Color::new(0.04, 0.0, 0.1, 0.2 * al),
            );
            let o = size * 0.08;
            for (dx, dy) in [
                (-o, 0.0),
                (o, 0.0),
                (0.0, -o),
                (0.0, o),
                (-o, -o),
                (o, o),
                (-o, o),
                (o, -o),
                (o, o * 2.0),
            ] {
                text(
                    &a.text,
                    w / 2.0 + dx,
                    BASE_H * 0.42 + size * 0.35 + dy,
                    size,
                    Color::new(0.07, 0.02, 0.12, al),
                    Align::Center,
                    true,
                );
            }
            let col = if ko {
                Color::new(1.0, 0.25, 0.25, al)
            } else {
                Color::new(1.0, 0.82, 0.25, al)
            };
            text(
                &a.text,
                w / 2.0,
                BASE_H * 0.42 + size * 0.35,
                size,
                col,
                Align::Center,
                true,
            );
        }
        // face-off at the start of the first round: speech bubbles (side 0 speaks first, side 1 answers)
        if m.round_state == 1 && m.round_no == 1 && m.intro_ticks > 0 && !m.training {
            let rt = m.round_time as f32;
            let lead: Vec<&Fighter> = (0..2)
                .filter_map(|sd| m.roots().find(|f| f.side == sd))
                .collect();
            if lead.len() == 2 {
                for (side, f) in lead.iter().enumerate() {
                    let (t0, t1) = if side == 0 {
                        (8.0, 108.0)
                    } else {
                        (100.0, 210.0)
                    };
                    if rt < t0 || rt > t1 || matches!(f.def.source, ume_engine::chardef::Source::Mugen) {
                        continue;
                    }
                    let txt = face_off_line(f, side == 1);
                    let shown: String = txt.chars().take(((rt - t0) * 1.5) as usize + 1).collect();
                    let a = ((rt - t0) / 6.0)
                        .min(1.0)
                        .min((t1 - rt) / 8.0)
                        .clamp(0.0, 1.0);
                    // wrap into at most two lines
                    let words: Vec<&str> = txt.split(' ').collect();
                    let mut cut = words.len();
                    if txt.chars().count() > 26 {
                        let mut acc = 0;
                        for (i, wd) in words.iter().enumerate() {
                            acc += wd.chars().count() + 1;
                            if acc * 2 >= txt.chars().count() {
                                cut = i + 1;
                                break;
                            }
                        }
                    }
                    let (l1, l2) = (
                        words[..cut.min(words.len())].join(" "),
                        words[cut.min(words.len())..].join(" "),
                    );
                    let full1 = l1.chars().count();
                    let sh1: String = shown.chars().take(full1).collect();
                    let sh2: String = shown.chars().skip(full1 + 1).collect();
                    let fs = 7.4;
                    let wdt = measure_text(&l1, Some(&ui.body), (fs * s) as u16, 1.0)
                        .width
                        .max(measure_text(&l2, Some(&ui.body), (fs * s) as u16, 1.0).width)
                        / s
                        + 14.0;
                    let hgt = if l2.is_empty() { 17.0 } else { 27.0 };
                    let bx = if side == 0 {
                        (BASE_W * 0.27 - wdt / 2.0).max(6.0)
                    } else {
                        (BASE_W * 0.73 - wdt / 2.0).min(BASE_W - wdt - 6.0)
                    };
                    let by = 58.0 + side as f32 * 14.0;
                    let col = Color::from_rgba(f.def.color[0], f.def.color[1], f.def.color[2], 255);
                    let sp = |v: Vec<Vec2>| -> Vec<Vec2> {
                        v.into_iter().map(|q| p(q.x, q.y)).collect()
                    };
                    poly::solid(
                        &sp(poly::chamfer(bx + 1.5, by + 1.8, wdt, hgt, 3.5)),
                        Color::new(0.0, 0.0, 0.0, 0.35 * a),
                    );
                    poly::grad_v(
                        &sp(poly::chamfer(bx, by, wdt, hgt, 3.5)),
                        Color::new(1.0, 1.0, 1.0, 0.96 * a),
                        Color::new(0.86, 0.88, 0.96, 0.96 * a),
                    );
                    poly::outline(
                        &sp(poly::chamfer(bx, by, wdt, hgt, 3.5)),
                        1.1 * s,
                        poly::with_a(col, a),
                    );
                    let tx = if side == 0 {
                        bx + wdt * 0.28
                    } else {
                        bx + wdt * 0.72
                    };
                    poly::solid(
                        &[
                            p(tx - 4.0, by + hgt - 0.3),
                            p(tx + 4.0, by + hgt - 0.3),
                            p(tx + if side == 0 { -2.0 } else { 2.0 }, by + hgt + 9.0),
                        ],
                        Color::new(0.93, 0.94, 0.98, 0.96 * a),
                    );
                    // name tab
                    let nm = f
                        .def
                        .display_name
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_uppercase();
                    let nw =
                        measure_text(&nm, Some(&ui.display), (5.6 * s) as u16, 1.0).width / s + 8.0;
                    poly::solid(
                        &sp(poly::slanted(bx + 2.0, by - 7.5, nw, 7.5, 2.0)),
                        poly::with_a(col, 0.95 * a),
                    );
                    text(
                        &nm,
                        bx + 5.0,
                        by - 1.7,
                        5.6,
                        Color::new(1.0, 1.0, 1.0, a),
                        Align::Left,
                        true,
                    );
                    let ink = Color::new(0.08, 0.06, 0.16, a);
                    text(&sh1, bx + 7.0, by + 10.4, fs, ink, Align::Left, false);
                    if !l2.is_empty() {
                        text(&sh2, bx + 7.0, by + 20.4, fs, ink, Align::Left, false);
                    }
                }
            }
        }
        // awakening hint: one super bar is enough
        if m.round_state == 2 && human.len() == 1 {
            if let Some(f) = m.roots().find(|f| human.contains(&f.id)) {
                if f.alive
                    && f.next_form_cost().is_some_and(|c| f.power >= c)
                    && f.def.anims.contains_key(&196)
                {
                    let a = 0.55 + 0.45 * (gt * 5.0).sin().abs();
                    text(
                        "T · DESPERTAR",
                        BASE_W / 2.0,
                        BASE_H - 8.0,
                        6.0,
                        Color::new(1.0, 0.85, 0.3, a),
                        Align::Center,
                        true,
                    );
                }
            }
        }
        if self.replay {
            let a = 0.6 + 0.4 * (gt * 3.0).sin().abs();
            draw_circle(
                p(w / 2.0 - 22.0, 29.0).x,
                p(0.0, 29.0).y,
                2.0 * s,
                Color::new(1.0, 0.2, 0.25, a),
            );
            text(
                "REPLAY",
                w / 2.0 + 2.0,
                31.0,
                6.0,
                Color::new(1.0, 1.0, 1.0, 0.9),
                Align::Center,
                true,
            );
        }
        if m.training {
            text(
                "TREINO",
                w / 2.0,
                30.0,
                6.0,
                Color::new(1.0, 1.0, 1.0, 0.8),
                Align::Center,
                true,
            );
            // attack data: last combo (hits + damage) stays on screen
            let (hits, dmg) = (self.combo[0].0, self.combo[0].1);
            if hits >= 2 {
                text(
                    &format!("ÚLTIMO COMBO  {hits} HITS · {} DMG", dmg as i32),
                    w / 2.0,
                    38.0,
                    5.6,
                    hex("#ffcc33"),
                    Align::Center,
                    true,
                );
            }
            // input history (newest at the bottom): direction arrow, buttons, frames held
            let n = self.inputs.len();
            draw_rectangle(
                p(6.0, 50.0).x,
                p(0.0, 50.0).y,
                62.0 * s,
                (n as f32 * 8.2 + 8.0) * s,
                Color::new(0.02, 0.02, 0.06, 0.55),
            );
            text(
                "INPUTS",
                10.0,
                56.0,
                5.0,
                Color::new(1.0, 1.0, 1.0, 0.6),
                Align::Left,
                true,
            );
            use ume_engine::input::k;
            for (i, (bits, fr)) in self.inputs.iter().enumerate() {
                let y = 62.0 + i as f32 * 8.2;
                let a = 0.45 + 0.55 * (i as f32 + 1.0) / n.max(1) as f32;
                let (dx, dy) = (
                    (bits & k::R != 0) as i32 - (bits & k::L != 0) as i32,
                    (bits & k::D != 0) as i32 - (bits & k::U != 0) as i32,
                );
                let c = p(14.0, y - 2.0);
                if dx != 0 || dy != 0 {
                    let ang = (dy as f32).atan2(dx as f32);
                    let r = 3.0 * s;
                    let (tip, l, rr) = (
                        c + vec2(ang.cos(), ang.sin()) * r,
                        c + vec2((ang + 2.5).cos(), (ang + 2.5).sin()) * r,
                        c + vec2((ang - 2.5).cos(), (ang - 2.5).sin()) * r,
                    );
                    draw_triangle(tip, l, rr, Color::new(0.4, 0.9, 1.0, a));
                } else {
                    draw_circle(c.x, c.y, 0.9 * s, Color::new(1.0, 1.0, 1.0, a * 0.5));
                }
                let names = [
                    (k::X, "LP"),
                    (k::Y, "HP"),
                    (k::A, "LK"),
                    (k::B, "HK"),
                    (k::D2, "S1"),
                    (k::W, "S2"),
                    (k::E, "S3"),
                ];
                let btn: Vec<&str> = names
                    .iter()
                    .filter(|(b, _)| bits & b != 0)
                    .map(|(_, n)| *n)
                    .collect();
                if !btn.is_empty() {
                    text(
                        &btn.join("+"),
                        22.0,
                        y,
                        5.0,
                        Color::new(1.0, 0.85, 0.3, a),
                        Align::Left,
                        true,
                    );
                }
                text(
                    &fr.to_string(),
                    64.0,
                    y,
                    4.6,
                    Color::new(1.0, 1.0, 1.0, a * 0.55),
                    Align::Right,
                    true,
                );
            }
        }
        let _ = gt;
    }
}

/// Round-start line of a fighter: its own `intro` (first speaker) / `taunt` (answer), or a generic one.
fn face_off_line(f: &Fighter, answer: bool) -> String {
    let own = if answer { &f.def.taunt } else { &f.def.intro };
    if !own.is_empty() {
        return own.clone();
    }
    const A: [&str; 5] = [
        "Vamos ver do que você é capaz!",
        "Prepare-se!",
        "Não vou pegar leve!",
        "Que vença o melhor!",
        "Mostre sua força!",
    ];
    const B: [&str; 5] = [
        "Só isso?",
        "Você não me assusta.",
        "Venha com tudo!",
        "Hmpf... entediante.",
        "Vai se arrepender.",
    ];
    let h = f
        .def
        .display_name
        .bytes()
        .fold(7usize, |a, b| a.wrapping_mul(31).wrapping_add(b as usize));
    (if answer { B } else { A })[h % 5].to_string()
}

const TILE_W: f32 = 10.5;
const TILE_GAP: f32 = 2.5;

fn side_tag(id: u32, side: usize, human: &[u32], cpu: &[u32]) -> Option<(&'static str, bool)> {
    if let Some(k) = human.iter().position(|&h| h == id) {
        return Some((if k == 0 { "P1" } else { "P2" }, true));
    }
    if cpu.contains(&id) {
        return Some(("CPU", false));
    }
    let _ = side;
    None
}

fn team_label(m: &Match, team: Option<&TeamInfo>, side: usize) -> Option<String> {
    if let Some(n) = team.and_then(|t| t.names[side].first()) {
        if team.is_some_and(|t| t.names[side].len() > 1) {
            return Some(n.0.clone());
        }
    }
    m.team_leader(side)
}

fn glass(
    p: &impl Fn(f32, f32) -> Vec2,
    s: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    r: f32,
    tint: [f32; 3],
) {
    let a = p(x, y);
    if glass_panel(a.x, a.y, w * s, h * s, r * s, tint) {
        return;
    }
    let pts: Vec<Vec2> = poly::chamfer(x, y, w, h, r * 0.6)
        .into_iter()
        .map(|q| p(q.x, q.y))
        .collect();
    poly::grad_v(
        &pts,
        Color::new(
            0.16 + tint[0] * 0.12,
            0.18 + tint[1] * 0.1,
            0.28 + tint[2] * 0.1,
            0.62,
        ),
        Color::new(0.05, 0.06, 0.12, 0.62),
    );
    poly::outline(&pts, 0.8 * s, Color::new(1.0, 1.0, 1.0, 0.3));
}

fn sp_box(p: &impl Fn(f32, f32) -> Vec2, x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<Vec2> {
    poly::chamfer(x, y, w, h, r * 0.6)
        .into_iter()
        .map(|q| p(q.x, q.y))
        .collect()
}

/// Round avatar on a soft character-coloured glow: head crop of the idle sprite (M.U.G.E.N.: the 9000,x portrait) with a transparent background.
fn portrait(g: &mut Gfx, f: &Fighter, pc: Vec2, half: f32, side: usize, s: f32, frame: Color) {
    let col = Color::from_rgba(f.def.color[0], f.def.color[1], f.def.color[2], 255);
    if let Some((sp, img, full)) = f.def.portrait_pick() {
        let tex = g.texture(&img, sp.scale >= 0.99);
        let (iw, ih) = (img.w as f32, img.h as f32);
        let src = Rect::new(0.0, 0.0, iw, ih);
        let hh = half * 2.0;
        let fit = (hh / src.w).min(hh / src.h);
        let (ww, wh) = (src.w * fit, src.h * fit);
        let (x0, y0) = (pc.x - ww / 2.0, pc.y - wh / 2.0);
        let flip = side == 1 && !full;
        draw_texture_ex(
            &tex,
            x0,
            y0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(ww, wh)),
                source: Some(src),
                flip_x: flip,
                ..Default::default()
            },
        );
    }
    let _ = (s, frame, col);
}

