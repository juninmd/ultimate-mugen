use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 4 shallow pond with stepping stones, lily pads, reeds
        layer(1100.0, 120.0, 175.0, 1.0, 5, 1.0, &|pm, t, w, h, r| {
            let rr = t.sx;
            rect(
                pm,
                t,
                0.0,
                28.0,
                w,
                h,
                &lin(
                    28.0,
                    h,
                    &[(0.0, "#2f6a66"), (0.4, "#14403f"), (1.0, "#082220")],
                ),
            );
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.18,
                0.9,
                3.0,
                0.5,
                0.3,
                "#0a2a2c",
                0.5,
                0.0,
                true,
            );
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.3,
                1.4,
                7.0,
                0.62,
                0.2,
                "#8fe8d8",
                0.2,
                0.0,
                true,
            );
            for _ in 0..90 {
                let y = 32.0 + r.f() * 84.0;
                let x = r.f() * w;
                let l = 6.0 + r.f() * 26.0;
                rect(
                    pm,
                    t,
                    x,
                    y,
                    l,
                    0.5,
                    &solid(rgba("#c8fff4", 0.1 + r.f() * 0.18)),
                );
            }
            // pebbles under shallow water
            for _ in 0..70 {
                fill(
                    pm,
                    t,
                    ellipse(
                        r.f() * w,
                        40.0 + r.f() * 74.0,
                        1.5 + r.f() * 2.5,
                        0.8 + r.f(),
                    ),
                    &solid(rgba(if r.f() < 0.5 { "#6a8a80" } else { "#3a5a56" }, 0.35)),
                );
            }
            // stepping stones
            for i in 0..7 {
                let x = 60.0 + i as f32 * 150.0 + r.f() * 40.0;
                let y = 60.0 + r.f() * 40.0;
                let (sw, sh) = (18.0 + r.f() * 10.0, 3.6 + r.f() * 1.5);
                fill(
                    pm,
                    t,
                    ellipse(x, y + 1.6, sw, sh),
                    &solid(rgba("#07201f", 0.6)),
                );
                fill(
                    pm,
                    t,
                    ellipse(x, y, sw, sh),
                    &lin(y - sh, y + sh, &[(0.0, "#8aa8a0"), (1.0, "#3a5a56")]),
                );
                stroke(
                    pm,
                    t,
                    ellipse(x, y, sw, sh),
                    &solid(rgba("#d8fff0", 0.35)),
                    0.5,
                );
            }
            grain(pm, rr, (0.0, 28.0, w, h - 28.0), 6.0, 6.0, 0.35, 2.0);
            // lily pads and flowers
            for _ in 0..22 {
                let (x, y, lr) = (r.f() * w, 38.0 + r.f() * 78.0, 3.0 + r.f() * 3.4);
                let mut b = PathBuilder::new();
                b.move_to(x, y);
                b.push_oval(
                    tiny_skia::Rect::from_xywh(x - lr * 1.8, y - lr * 0.5, lr * 3.6, lr).unwrap(),
                );
                fill(
                    pm,
                    t,
                    ellipse(x, y, lr * 1.8, lr * 0.5),
                    &lin(
                        y - lr * 0.5,
                        y + lr * 0.5,
                        &[(0.0, "#5aa860"), (1.0, "#2a6a46")],
                    ),
                );
                stroke(
                    pm,
                    t,
                    line(x, y, x + lr * 1.6, y - lr * 0.1),
                    &solid(rgba("#0e3a2a", 0.8)),
                    0.35,
                );
                if r.f() < 0.3 {
                    fill(
                        pm,
                        t,
                        ellipse(x - lr, y - lr * 0.4, 1.4, 1.0),
                        &solid(rgba("#ffd0e8", 0.95)),
                    );
                }
            }
            // far bank with ferns and rocks
            rect(
                pm,
                t,
                0.0,
                0.0,
                w,
                30.0,
                &lin(0.0, 30.0, &[(0.0, "#071c1c"), (1.0, "#0e3030")]),
            );
            let mut x = 0.0;
            while x < w {
                let hh = 8.0 + r.f() * 18.0;
                let mut b = PathBuilder::new();
                b.move_to(x, 30.0);
                b.quad_to(x + 2.0, 30.0 - hh * 0.8, x + 5.0 + r.f() * 6.0, 30.0 - hh);
                b.quad_to(x + 5.0, 30.0 - hh * 0.5, x + 7.0, 30.0);
                b.close();
                fill(
                    pm,
                    t,
                    b.finish(),
                    &solid(rgba(if r.f() < 0.5 { "#0e3a30" } else { "#164a3a" }, 1.0)),
                );
                x += 3.0 + r.f() * 3.0;
            }
            for _ in 0..16 {
                let x = r.f() * w;
                fill(
                    pm,
                    t,
                    ellipse(x, 28.0, 7.0 + r.f() * 9.0, 4.0 + r.f() * 3.0),
                    &lin(22.0, 32.0, &[(0.0, "#4a6e66"), (1.0, "#14302c")]),
                );
            }
            rect(pm, t, 0.0, 29.0, w, 1.4, &solid(rgba("#b0fff0", 0.25)));
        }),

        // 5 dynamic foreground stalks
        dynamic(560.0, 260.0, -10.0, 1.28, 1.0, true),
    ]
}
