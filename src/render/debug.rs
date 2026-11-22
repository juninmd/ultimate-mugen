use super::{Gfx, MatchRenderer};
use crate::gfx::*;
use crate::session::debug_ctrl::{self, DebugLayers};
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

impl MatchRenderer {
    pub(super) fn draw_debug_world(&self, g: &Gfx, m: &Match) {
        let mode = debug_ctrl::layers();
        if mode == DebugLayers::HudOnly {
            return;
        }
        let s = g.view_scale;
        let show_guides = mode == DebugLayers::All;
        let pt = |wx: f32, wy: f32| vec2(g.ox + Self::sx(m, wx) * s, g.oy + Self::sy(m, wy) * s);
        let fill_box = |b: &[f32; 4], fill: Color, line: Color, stroke: f32| {
            let x = g.ox + Self::sx(m, b[0]) * s;
            let y = g.oy + Self::sy(m, b[1]) * s;
            let w = (b[2] - b[0]) * s;
            let h = (b[3] - b[1]) * s;
            draw_rectangle(x, y, w, h, fill);
            draw_rectangle_lines(x, y, w, h, stroke, line);
        };

        if show_guides {
            let gy = pt(0.0, 0.0).y;
            draw_line(
                g.ox,
                gy,
                g.ox + m.view_w * s,
                gy,
                1.0,
                Color::new(0.4, 0.9, 0.7, 0.35),
            );
            for bx in [m.stage.bound_left, m.stage.bound_right] {
                let lx = pt(bx, 0.0).x;
                draw_line(
                    lx,
                    gy - 120.0 * s,
                    lx,
                    gy + 10.0 * s,
                    1.0,
                    Color::new(0.95, 0.65, 0.2, 0.45),
                );
            }
            let cam = pt(m.camera.x, m.camera.y - 45.0);
            draw_line(
                cam.x - 6.0 * s,
                cam.y,
                cam.x + 6.0 * s,
                cam.y,
                1.0,
                Color::new(0.9, 0.9, 1.0, 0.5),
            );
            draw_line(
                cam.x,
                cam.y - 6.0 * s,
                cam.x,
                cam.y + 6.0 * s,
                1.0,
                Color::new(0.9, 0.9, 1.0, 0.5),
            );
        }

        for f in m.fighters.iter().filter(|f| !f.destroyed) {
            if show_guides {
                let pb = pushbox(f);
                draw_rectangle_lines(
                    g.ox + Self::sx(m, pb[0]) * s,
                    g.oy + Self::sy(m, pb[1]) * s,
                    (pb[2] - pb[0]) * s,
                    (pb[3] - pb[1]) * s,
                    1.0,
                    Color::new(1.0, 0.78, 0.2, 0.65),
                );
            }
            for b in f.boxes(2) {
                fill_box(
                    &b,
                    Color::new(0.18, 0.55, 1.0, 0.14),
                    Color::new(0.25, 0.68, 1.0, 0.95),
                    1.6,
                );
            }
            for b in f.boxes(1) {
                fill_box(
                    &b,
                    Color::new(1.0, 0.18, 0.22, 0.24),
                    Color::new(1.0, 0.25, 0.25, 1.0),
                    2.0,
                );
            }
            if show_guides {
                draw_links(m, f, &pt, s);
                draw_pivot_and_vel(f, &pt, s);
                let label = if f.is_helper {
                    format!("H#{} S{} t{}", f.helper_id, f.state_no, f.state_time)
                } else {
                    format!(
                        "S{} {}{} a{} t{}",
                        f.state_no, f.state_type, f.move_type, f.anim_no, f.state_time
                    )
                };
                let p = pt(f.pos.x, f.pos.y + f.lane);
                draw_text(&label, p.x - 20.0 * s, p.y + 10.0 * s, 4.0 * s, YELLOW);
            }
        }

        for p in &m.projectiles {
            for b in Match::proj_boxes(p, 1) {
                fill_box(&b, Color::new(1.0, 0.5, 0.1, 0.25), ORANGE, 1.8);
            }
            if show_guides {
                let o = pt(p.pos.x, p.pos.y);
                draw_circle(o.x, o.y, 2.0 * s, ORANGE);
                draw_text(
                    &format!("P#{}", p.id),
                    o.x - 8.0 * s,
                    o.y - 6.0 * s,
                    3.6 * s,
                    ORANGE,
                );
            }
        }
    }
}

fn pushbox(f: &Fighter) -> [f32; 4] {
    let back = f.back_width();
    let front = f.front_width();
    let top = f.height_top();
    let bot = f.height_bottom();
    let y = f.pos.y + f.lane;
    if f.facing >= 0.0 {
        [f.pos.x - back, y - top, f.pos.x + front, y + bot]
    } else {
        [f.pos.x - front, y - top, f.pos.x + back, y + bot]
    }
}

fn draw_pivot_and_vel(f: &Fighter, pt: &impl Fn(f32, f32) -> Vec2, s: f32) {
    let p = pt(f.pos.x, f.pos.y + f.lane);
    let col = if f.is_helper {
        Color::new(0.95, 0.45, 1.0, 0.95)
    } else if f.state_type == 'A' {
        Color::new(0.3, 0.9, 1.0, 0.95)
    } else {
        Color::new(0.35, 1.0, 0.55, 0.95)
    };
    draw_line(p.x - 4.0 * s, p.y, p.x + 4.0 * s, p.y, 1.5, col);
    draw_line(p.x, p.y - 4.0 * s, p.x, p.y + 4.0 * s, 1.5, col);
    if f.vel.x.abs() > 0.05 || f.vel.y.abs() > 0.05 {
        let tip = pt(f.pos.x + f.vel.x * 6.0, f.pos.y + f.lane + f.vel.y * 6.0);
        draw_line(
            p.x,
            p.y,
            tip.x,
            tip.y,
            1.6,
            Color::new(0.65, 1.0, 0.4, 0.85),
        );
        draw_circle(tip.x, tip.y, 1.6 * s, Color::new(0.65, 1.0, 0.4, 0.95));
    }
}

fn draw_links(m: &Match, f: &Fighter, pt: &impl Fn(f32, f32) -> Vec2, s: f32) {
    let a = pt(f.pos.x, f.pos.y + f.lane - 36.0);
    if f.is_helper {
        if let Some(pid) = f.parent {
            if let Some(p) = m.idx(pid).map(|idx| &m.fighters[idx]) {
                let b = pt(p.pos.x, p.pos.y + p.lane - 36.0);
                draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(0.5, 0.85, 1.0, 0.45));
            }
        }
    }
    for &(tid, _) in &f.targets {
        if let Some(t) = m.idx(tid).map(|idx| &m.fighters[idx]) {
            let b = pt(t.pos.x, t.pos.y + t.lane - 36.0);
            draw_line(a.x, a.y, b.x, b.y, 1.4, Color::new(1.0, 0.35, 0.85, 0.75));
            let mid = vec2((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
            draw_text(
                &format!(
                    "T#{} ({:.0},{:.0})",
                    t.id,
                    t.pos.x - f.pos.x,
                    t.pos.y - f.pos.y
                ),
                mid.x,
                mid.y - 3.0 * s,
                3.2 * s,
                Color::new(1.0, 0.55, 0.9, 0.9),
            );
        }
    }
}
