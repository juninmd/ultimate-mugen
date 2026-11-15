use super::*;

pub(super) fn pagoda(
    pm: &mut Pixmap,
    t: Transform,
    x: f32,
    by: f32,
    bw: f32,
    tiers: i32,
    col: &str,
    win: f32,
) {
    let dark = solid(rgba(col, 1.0));
    rect(pm, t, x - bw * 0.35, by - 30.0, bw * 0.7, 30.0, &dark);
    for k in 0..tiers {
        let ty = by - 30.0 - k as f32 * 22.0;
        let tw = bw * (1.0 - k as f32 * 0.2);
        let mut b = PathBuilder::new();
        b.move_to(x - tw / 2.0 - 11.0, ty + 7.0);
        b.quad_to(x - tw / 4.0, ty - 2.0, x, ty - 13.0);
        b.quad_to(x + tw / 4.0, ty - 2.0, x + tw / 2.0 + 11.0, ty + 7.0);
        b.quad_to(x + tw / 2.0, ty + 3.0, x + tw / 2.0 - 4.0, ty + 3.0);
        b.line_to(x - tw / 2.0 + 4.0, ty + 3.0);
        b.quad_to(x - tw / 2.0, ty + 3.0, x - tw / 2.0 - 11.0, ty + 7.0);
        b.close();
        fill(pm, t, b.finish(), &dark);
        rect(pm, t, x - tw * 0.3, ty + 2.0, tw * 0.6, 18.0, &dark);
        if win > 0.0 {
            for q in 0..3 {
                rect(
                    pm,
                    t,
                    x - tw * 0.22 + q as f32 * tw * 0.18,
                    ty + 7.0,
                    3.0,
                    6.0,
                    &solid(rgba("#ffbe5a", win)),
                );
            }
        }
    }
    let top = by - 30.0 - tiers as f32 * 22.0;
    stroke(pm, t, line(x, top - 6.0, x, top - 26.0), &dark, 1.4);
    for q in 0..3 {
        fill(
            pm,
            t,
            ellipse(x, top - 10.0 - q as f32 * 5.0, 3.0, 1.0),
            &dark,
        );
    }
}
pub(super) fn sakura(pm: &mut Pixmap, t: Transform, r: &mut Rng, x: f32, by: f32, h: f32) {
    let bark = solid(rgba("#241218", 1.0));
    fill(
        pm,
        t,
        poly(&[
            (x - 7.0, by),
            (x - 4.0, by - h * 0.4),
            (x - 2.0, by - h * 0.62),
            (x + 3.0, by - h * 0.62),
            (x + 5.0, by - h * 0.4),
            (x + 9.0, by),
        ]),
        &bark,
    );
    let mut tips = vec![];
    for k in 0..7 {
        let a = -1.57 + (k as f32 - 3.0) * 0.42 + (r.f() - 0.5) * 0.3;
        let l = h * (0.35 + r.f() * 0.25);
        let (sx, sy) = (x + 0.5, by - h * (0.45 + r.f() * 0.2));
        let (ex, ey) = (sx + a.cos() * l, sy + a.sin() * l);
        let mut b = PathBuilder::new();
        b.move_to(sx, sy);
        b.quad_to(
            sx + a.cos() * l * 0.4 + (r.f() - 0.5) * 10.0,
            sy + a.sin() * l * 0.6,
            ex,
            ey,
        );
        stroke(pm, t, b.finish(), &bark, 3.4 - k as f32 * 0.2);
        tips.push((ex, ey));
    }
    stroke(
        pm,
        t,
        line(x - 3.0, by, x - 2.0, by - h * 0.55),
        &solid(rgba("#8a4a50", 0.4)),
        0.8,
    );
    let pal = ["#ffd4e0", "#ffbcd0", "#f8a4c0", "#e87aa0", "#b84e80"];
    for (ex, ey) in tips {
        fill(
            pm,
            t,
            ellipse(ex, ey + 3.0, 27.0, 20.0),
            &solid(rgba("#a8406a", 0.9)),
        );
        for _ in 0..70 {
            let (a, d) = (r.f() * std::f32::consts::TAU, r.f().sqrt() * 25.0);
            let (px, py) = (ex + a.cos() * d * 1.1, ey + a.sin() * d * 0.8);
            let lit =
                (1.0 - ((py - (ey - 18.0)) / 36.0).clamp(0.0, 1.0)) * 4.0 + (r.f() - 0.5) * 1.6;
            let k = (4.0 - lit).clamp(0.0, 4.0) as usize;
            fill(
                pm,
                t,
                circle(px, py, 2.2 + r.f() * 3.6),
                &solid(rgba(pal[k], 1.0)),
            );
        }
        for _ in 0..10 {
            fill(
                pm,
                t,
                circle(
                    ex + (r.f() - 0.5) * 36.0,
                    ey - 12.0 + r.f() * 6.0,
                    1.4 + r.f(),
                ),
                &solid(rgba("#fff0f4", 0.9)),
            );
        }
    }
}
