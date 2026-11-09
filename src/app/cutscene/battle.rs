use crate::app::App;
use crate::gfx::*;
use crate::poly;
use std::sync::Arc;
use ume_engine::chardef::CharDef;

fn draw_line_quad(app: &mut App, a: Vec2, b: Vec2, width: f32, col: Color) {
    let d = b - a;
    let len = (d.x * d.x + d.y * d.y).sqrt().max(0.001);
    let perp = vec2(-d.y, d.x) * (width * 0.5 / len);
    app.ui.quad([a + perp, b + perp, b - perp, a - perp], col);
}

pub fn draw_battle_effects(
    app: &mut App,
    t: f32,
    page: usize,
    is_finish: bool,
    hd: &Option<Arc<CharDef>>,
    bd: &Option<Arc<CharDef>>,
    hc: Color,
    bc: Color,
) {
    let s = app.ui.s;
    let (mid_x, ground_y) = (640.0, 610.0);
    if !is_finish {
        draw_clash_fight(app, t, page, hd, bd, hc, bc, s, mid_x, ground_y);
    } else {
        draw_defeat_climax(app, t, page, hd, bd, hc, bc, s, mid_x, ground_y);
    }
}

fn draw_clash_fight(
    app: &mut App,
    t: f32,
    page: usize,
    hd: &Option<Arc<CharDef>>,
    bd: &Option<Arc<CharDef>>,
    _hc: Color,
    _bc: Color,
    s: f32,
    mid_x: f32,
    ground_y: f32,
) {
    let cycle = (t * 4.0) % 2.0;
    let clash_t = (t * 8.0).floor() as usize;
    let hero_anims = [1000, 1001, 1002, 100];
    let boss_anims = [1000, 1001, 1002, 100];
    let (anim_h, anim_b) = if page >= 3 {
        (0, -2196)
    } else {
        (hero_anims[clash_t % hero_anims.len()], -1000 - boss_anims[(clash_t + 1) % boss_anims.len()])
    };

    let offset = if cycle < 1.0 { (cycle * 60.0).sin() * 80.0 } else { 0.0 };
    let hero_x = mid_x - 140.0 + offset;
    let boss_x = mid_x + 140.0 - offset;
    let pt = app.preview_t;

    let line_col = if page >= 3 { Color::new(0.0, 0.94, 1.0, 0.18) } else { Color::new(1.0, 0.2, 0.3, 0.18) };
    draw_speedlines(app, s, line_col);

    if let Some(d) = hd {
        app.draw_char(d, anim_h, pt, hero_x, ground_y, 490.0, false);
    }
    if let Some(d) = bd {
        let by = if page >= 3 { ground_y - 65.0 - (t * 3.0).sin() * 18.0 } else { ground_y };
        app.draw_char(d, anim_b, pt, boss_x, by, 520.0, true);
    }

    let cx = (hero_x + boss_x) * 0.5;
    let cy = ground_y - 180.0;
    let clash_pulse = (t * 16.0).sin().abs();
    let spark_col = if page >= 3 { Color::new(0.1, 0.95, 1.0, 0.8) } else { Color::new(1.0, 0.85, 0.3, 0.8) };
    poly::ring(vec2(app.ui.x(cx), app.ui.y(cy)), (45.0 + clash_pulse * 65.0) * s, 3.5 * s, spark_col);
    poly::circle(vec2(app.ui.x(cx), app.ui.y(cy)), (18.0 + clash_pulse * 25.0) * s, Color::new(1.0, 1.0, 1.0, 0.85));

    for i in 0..6 {
        let ang = (i as f32 * 1.047) + t * 5.0;
        let r0 = (20.0 + clash_pulse * 30.0) * s;
        let r1 = (70.0 + clash_pulse * 90.0) * s;
        let p0 = vec2(app.ui.x(cx) + ang.cos() * r0, app.ui.y(cy) + ang.sin() * r0);
        let p1 = vec2(app.ui.x(cx) + ang.cos() * r1, app.ui.y(cy) + ang.sin() * r1);
        draw_line_quad(app, p0, p1, 2.5 * s, spark_col);
    }

    if page >= 3 {
        draw_cosmic_nova(app, s, boss_x, ground_y - 200.0, t);
    }
}

fn draw_defeat_climax(
    app: &mut App,
    t: f32,
    page: usize,
    hd: &Option<Arc<CharDef>>,
    bd: &Option<Arc<CharDef>>,
    _hc: Color,
    _bc: Color,
    s: f32,
    mid_x: f32,
    ground_y: f32,
) {
    let pt = app.preview_t;
    if page == 0 || page == 1 {
        let dash_k = (t * 2.8).min(1.0);
        let hero_x = 240.0 + dash_k * 360.0;
        let boss_x = mid_x + 180.0 + (dash_k.powi(2) * 120.0);
        draw_speedlines(app, s, Color::new(1.0, 0.92, 0.5, 0.3));

        if let Some(d) = hd {
            app.draw_char(d, 1001, pt, hero_x, ground_y, 500.0, false);
        }
        if let Some(d) = bd {
            let stagger_anim = if dash_k > 0.85 { 5002 } else { 5000 };
            app.draw_char(d, stagger_anim, pt, boss_x, ground_y, 510.0, true);
        }

        if dash_k > 0.8 {
            let hit_x = (hero_x + boss_x) * 0.5;
            let hit_y = ground_y - 180.0;
            poly::circle(vec2(app.ui.x(hit_x), app.ui.y(hit_y)), 120.0 * s, Color::new(1.0, 1.0, 1.0, 0.75));
            poly::ring(vec2(app.ui.x(hit_x), app.ui.y(hit_y)), 180.0 * s, 6.0 * s, Color::new(1.0, 0.85, 0.2, 0.9));
        }
    } else {
        if let Some(d) = hd {
            app.draw_char(d, 181, pt, 320.0, ground_y, 480.0, false);
        }
        if let Some(d) = bd {
            app.draw_char(d, 5150, pt, 980.0, ground_y + 10.0, 490.0, true);
        }
        draw_shattered_blade(app, s, 940.0, ground_y - 40.0, t);
        draw_dawn_fracture(app, s, mid_x, 220.0, t, page);
    }
}

fn draw_speedlines(app: &mut App, s: f32, col: Color) {
    let t = app.ui.t;
    for i in 0..10 {
        let y = 60.0 + (i as f32 * 62.0) + (t * 80.0) % 62.0;
        let x0 = (t * 900.0 + i as f32 * 173.0) % 1280.0;
        let x1 = x0 + 190.0;
        let p0 = vec2(app.ui.x(x0), app.ui.y(y));
        let p1 = vec2(app.ui.x(x1), app.ui.y(y));
        draw_line_quad(app, p0, p1, 2.0 * s, col);
    }
}

fn draw_cosmic_nova(app: &mut App, s: f32, x: f32, y: f32, t: f32) {
    let p = vec2(app.ui.x(x), app.ui.y(y));
    let pulse = 1.0 + 0.15 * (t * 10.0).sin();
    poly::circle(p, 110.0 * s * pulse, Color::new(0.0, 0.92, 1.0, 0.18));
    poly::ring(p, 140.0 * s * pulse, 4.0 * s, Color::new(0.0, 0.95, 1.0, 0.65));
    poly::ring(p, 175.0 * s * (2.0 - pulse), 2.0 * s, Color::new(0.95, 0.0, 0.45, 0.45));
}

fn draw_shattered_blade(app: &mut App, s: f32, x: f32, y: f32, t: f32) {
    for i in 0..8 {
        let f = i as f32;
        let drift_x = x + (f * 17.0).cos() * (30.0 + t * 25.0) + f * 9.0;
        let drift_y = y - (f * 13.0).sin().abs() * (20.0 + t * 35.0) + (t * 15.0);
        let ang = t * 3.0 + f * 1.5;
        let p = vec2(app.ui.x(drift_x), app.ui.y(drift_y));
        let p0 = p + vec2(ang.cos() * 8.0 * s, ang.sin() * 8.0 * s);
        let p1 = p - vec2(ang.cos() * 8.0 * s, ang.sin() * 8.0 * s);
        draw_line_quad(app, p0, p1, 2.5 * s, Color::new(0.2, 0.95, 1.0, 0.7));
    }
}

fn draw_dawn_fracture(app: &mut App, s: f32, x: f32, y: f32, t: f32, page: usize) {
    let p = vec2(app.ui.x(x), app.ui.y(y));
    let k = if page >= 4 { 1.0 } else { 0.4 };
    poly::circle(p, 160.0 * s, Color::new(1.0, 0.92, 0.55, 0.18 * k));
    poly::ring(p, 165.0 * s, 5.0 * s, Color::new(1.0, 0.85, 0.35, 0.7 * k));
    for i in 0..12 {
        let ang = i as f32 * std::f32::consts::TAU / 12.0 + t * 0.2;
        let p0 = p + vec2(ang.cos() * 165.0 * s, ang.sin() * 165.0 * s);
        let p1 = p + vec2(ang.cos() * 320.0 * s, ang.sin() * 320.0 * s);
        draw_line_quad(app, p0, p1, 3.0 * s, Color::new(1.0, 0.9, 0.5, 0.35 * k));
    }
    for i in 0..7 {
        let f = i as f32;
        let ca = (f * 1.2).cos() * 120.0 * s;
        let cb = (f * 1.5).sin() * 120.0 * s;
        let p1 = vec2(p.x + ca, p.y + cb);
        draw_line_quad(app, p, p1, 2.0 * s, Color::new(1.0, 0.98, 0.8, 0.8 * k));
    }
}
