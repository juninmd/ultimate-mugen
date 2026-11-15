use super::*;

pub(super) fn basalt(pm: &mut Pixmap, t: Transform, r: &mut Rng, x: f32, by: f32, w: f32, h: f32, glow: bool) {
    let n = 4;
    let cw = w / n as f32;
    for i in 0..n {
        let hh = h * (0.55 + r.f() * 0.45);
        let cx = x + i as f32 * cw;
        fill(
            pm,
            t,
            poly(&[
                (cx, by),
                (cx + 1.0, by - hh),
                (cx + cw * 0.5, by - hh - 3.0),
                (cx + cw, by - hh + 1.0),
                (cx + cw + 1.0, by),
            ]),
            &lin_h(
                cx,
                cx + cw,
                &[
                    (0.0, "#3a1a14", 1.0),
                    (0.5, "#241008", 1.0),
                    (1.0, "#0e0504", 1.0),
                ],
            ),
        );
        rect(
            pm,
            t,
            cx,
            by - hh,
            1.0,
            hh,
            &solid(rgba("#ff7a30", if glow { 0.7 } else { 0.35 })),
        );
        stroke(
            pm,
            t,
            line(cx + cw * 0.5, by - hh - 3.0, cx + cw, by - hh + 1.0),
            &solid(rgba("#ff9a50", 0.5)),
            0.8,
        );
        for k in 1..4 {
            rect(
                pm,
                t,
                cx + 1.0,
                by - hh * k as f32 / 4.0,
                cw - 1.5,
                0.6,
                &solid(rgba("#0a0302", 0.6)),
            );
        }
    }
}
