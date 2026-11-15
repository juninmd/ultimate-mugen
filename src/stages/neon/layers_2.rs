use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 4 wet asphalt street with puddles, lane paint, manholes and a barrier
        layer(1100.0, 120.0, 175.0, 1.0, 12, 1.0, &|pm, t, w, h, r| {
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
                    &[(0.0, "#2a1c48"), (0.35, "#160c2c"), (1.0, "#080414")],
                ),
            );
            grain(pm, rr, (0.0, 28.0, w, h - 28.0), 6.0, 6.0, 0.7, 1.0);
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.25,
                0.8,
                4.0,
                0.58,
                0.25,
                "#000000",
                0.4,
                0.0,
                true,
            );
            for _ in 0..380 {
                let a = 0.1 + r.f() * 0.25;
                rect(
                    pm,
                    t,
                    r.f() * w,
                    30.0 + r.f() * 88.0,
                    0.7,
                    0.7,
                    &solid(rgba(if r.f() < 0.5 { "#8a7ac8" } else { "#000000" }, a)),
                );
            }
            for _ in 0..16 {
                let x = r.f() * w;
                let y = 40.0 + r.f() * 70.0;
                let mut b = PathBuilder::new();
                b.move_to(x, y);
                for _ in 0..3 {
                    b.line_to(x + (r.f() - 0.2) * 30.0, y + (r.f() - 0.5) * 8.0);
                }
                stroke(pm, t, b.finish(), &solid(rgba("#000000", 0.5)), 0.5);
            }
            // curb + lane markings
            rect(
                pm,
                t,
                0.0,
                26.0,
                w,
                4.0,
                &lin(26.0, 30.0, &[(0.0, "#4a3a78"), (1.0, "#1c1236")]),
            );
            let mut x = 8.0;
            while x < w {
                rect(pm, t, x, 70.0, 34.0, 2.4, &solid(rgba("#e8e0ff", 0.55)));
                x += 78.0;
            }
            rect(pm, t, 0.0, 104.0, w, 2.0, &solid(rgba("#ffe23a", 0.35)));
            for i in 0..3 {
                let x = 200.0 + i as f32 * 380.0;
                fill(
                    pm,
                    t,
                    ellipse(x, 62.0, 13.0, 3.6),
                    &solid(rgba("#0c0618", 1.0)),
                );
                stroke(
                    pm,
                    t,
                    ellipse(x, 62.0, 13.0, 3.6),
                    &solid(rgba("#6a5aa8", 0.6)),
                    0.6,
                );
                for k in 0..4 {
                    rect(
                        pm,
                        t,
                        x - 9.0 + k as f32 * 4.6,
                        61.0,
                        0.7,
                        2.2,
                        &solid(rgba("#4a3a78", 0.8)),
                    );
                }
            }
            // puddles reflecting the neon sky
            for i in 0..11 {
                let (x, y) = (r.f() * w, 44.0 + r.f() * 66.0);
                let (pw, ph) = (22.0 + r.f() * 40.0, 3.0 + r.f() * 4.5);
                fill(
                    pm,
                    t,
                    ellipse(x, y, pw, ph),
                    &radial(
                        x,
                        y,
                        pw,
                        &[
                            (0.0, if i % 2 == 0 { "#ff6ad8" } else { "#52e8ff" }, 0.8),
                            (0.5, "#6a40c0", 0.65),
                            (1.0, "#1a0c38", 0.5),
                        ],
                    ),
                );
                stroke(
                    pm,
                    t,
                    ellipse(x, y, pw, ph),
                    &solid(rgba("#c0b0ff", 0.4)),
                    0.5,
                );
                rect(
                    pm,
                    t,
                    x - pw * 0.5,
                    y - 0.6,
                    pw,
                    0.8,
                    &solid(rgba("#ffffff", 0.35)),
                );
            }
            // barrier
            rect(pm, t, 0.0, 6.0, w, 2.0, &solid(rgba("#4c3a80", 1.0)));
            rect(pm, t, 0.0, 6.0, w, 0.7, &solid(rgba("#a898ff", 0.6)));
            let mut x = 0.0;
            while x < w {
                rect(
                    pm,
                    t,
                    x,
                    6.0,
                    1.6,
                    22.0,
                    &lin(6.0, 28.0, &[(0.0, "#4a3a80"), (1.0, "#1a1234")]),
                );
                x += 15.0;
            }
            let mut x = 0.0;
            while x < w {
                rect(pm, t, x, 20.0, 12.0, 1.0, &solid(rgba("#33255a", 0.9)));
                x += 15.0;
            }
        }),

        // 5 dynamic: hanging cables + street lamps in front
        dynamic(560.0, 240.0, 0.0, 1.2, 1.0, true),
    ]
}
