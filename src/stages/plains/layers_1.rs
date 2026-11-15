use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 3 near rocks, cacti, dry shrubs (windmill is animated by stagefx)
        lay(
            3.5,
            false,
            1100.0,
            240.0,
            0.0,
            0.7,
            35,
            0.7,
            &|pm, t, w, _, r| {
                for i in 0..5 {
                    let x = 40.0 + i as f32 * 230.0 + r.f() * 60.0;
                    if i != 2 {
                        let (rw, rh) = (46.0 + r.f() * 40.0, 30.0 + r.f() * 36.0);
                        rockform(pm, t, r, x, 211.0, rw, rh, "#c8845a", "#6a3a26");
                    }
                }
                for i in 0..6 {
                    cactus(
                        pm,
                        t,
                        120.0 + i as f32 * 170.0 + r.f() * 60.0,
                        210.0,
                        22.0 + r.f() * 20.0,
                    );
                }
                for _ in 0..40 {
                    let x = r.f() * w;
                    let hh = 4.0 + r.f() * 8.0;
                    for k in 0..5 {
                        stroke(
                            pm,
                            t,
                            line(x, 210.0, x + (k as f32 - 2.0) * 2.2, 210.0 - hh),
                            &solid(rgba(if r.f() < 0.5 { "#8a7a3a" } else { "#a89448" }, 0.9)),
                            0.6,
                        );
                    }
                }
                grain(pm, t.sx, (0.0, 150.0, w, 70.0), 3.0, 3.0, 0.4, 5.0);
                haze(pm, t.sx, (0.0, 160.0, w, 60.0), "#f4e4c8", 0.0, 0.25);
            },
        ),

        // 4 cracked dry earth
        layer(1100.0, 120.0, 175.0, 1.0, 35, 1.0, &|pm, t, w, h, r| {
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
                    &[(0.0, "#e0bc8c"), (0.5, "#c89c6c"), (1.0, "#9a7048")],
                ),
            );
            grain(pm, rr, (0.0, 28.0, w, h - 28.0), 7.0, 7.0, 0.55, 1.0);
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.3,
                1.0,
                2.0,
                0.55,
                0.3,
                "#8a5a38",
                0.45,
                0.0,
                true,
            );
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.4,
                1.4,
                6.0,
                0.66,
                0.2,
                "#fff0cc",
                0.35,
                0.0,
                true,
            );
            // sand ripples
            for _ in 0..60 {
                let x = r.f() * w;
                let y = 36.0 + r.f() * 80.0;
                let l = 20.0 + r.f() * 60.0;
                stroke(
                    pm,
                    t,
                    line(x, y, x + l, y + (r.f() - 0.5) * 3.0),
                    &solid(rgba("#7a4c2c", 0.14)),
                    0.7,
                );
            }
            // dried-mud cracks (polygon network)
            for _ in 0..34 {
                let (mut x, mut y) = (r.f() * w, 36.0 + r.f() * 80.0);
                let mut b = PathBuilder::new();
                b.move_to(x, y);
                for _ in 0..5 {
                    x += (r.f() - 0.5) * 22.0;
                    y += (r.f() - 0.5) * 8.0;
                    b.line_to(x, y);
                }
                let p = b.finish();
                stroke(pm, t, p.clone(), &solid(rgba("#3a2010", 0.55)), 0.8);
            }
            for _ in 0..180 {
                fill(
                    pm,
                    t,
                    ellipse(
                        r.f() * w,
                        34.0 + r.f() * 84.0,
                        0.8 + r.f() * 2.8,
                        0.5 + r.f() * 1.0,
                    ),
                    &solid(rgba(
                        if r.f() < 0.5 { "#5a3c28" } else { "#f4dcb0" },
                        0.4 + r.f() * 0.3,
                    )),
                );
            }
            shade_v(pm, rr, (0.0, 28.0, w, h - 28.0), 0.78, 1.05);
            // far edge: scattered stones and tufts
            rect(
                pm,
                t,
                0.0,
                0.0,
                w,
                30.0,
                &lin_a(0.0, 30.0, &[(0.0, "#e0bc8c", 0.0), (1.0, "#e0bc8c", 1.0)]),
            );
            for _ in 0..40 {
                let x = r.f() * w;
                let hh = 3.0 + r.f() * 8.0;
                for k in 0..4 {
                    stroke(
                        pm,
                        t,
                        line(x, 29.0, x + (k as f32 - 1.5) * 1.6, 29.0 - hh),
                        &solid(rgba("#a88c4a", 0.9)),
                        0.5,
                    );
                }
            }
            for _ in 0..22 {
                fill(
                    pm,
                    t,
                    ellipse(r.f() * w, 28.0, 2.0 + r.f() * 5.0, 1.4 + r.f() * 2.0),
                    &lin(24.0, 30.0, &[(0.0, "#b88a68"), (1.0, "#6a4630")]),
                );
            }
            rect(pm, t, 0.0, 29.0, w, 1.0, &solid(rgba("#fff0cc", 0.3)));
        }),

        // 5 dynamic foreground grass
        dynamic(560.0, 240.0, 0.0, 1.25, 1.0, true),
    ]
}
