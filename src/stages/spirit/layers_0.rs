use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 0 night sky (aurora is animated)
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.04,
            40,
            0.04,
            &|pm, t, w, h, r| {
                rect(
                    pm,
                    t,
                    0.0,
                    0.0,
                    w,
                    h,
                    &lin(
                        0.0,
                        h,
                        &[(0.0, "#050c18"), (0.45, "#12363e"), (1.0, "#7ac0b8")],
                    ),
                );
                for _ in 0..120 {
                    let a = r.f() * 0.8;
                    fill(
                        pm,
                        t,
                        circle(r.f() * w, r.f() * 150.0, r.f() * 1.1),
                        &solid(rgba("#c8fff4", a)),
                    );
                }
                glowc(pm, t, w * 0.62, 70.0, 80.0, "#a8fff0", 0.45);
                fill(
                    pm,
                    t,
                    circle(w * 0.62, 70.0, 26.0),
                    &radial(
                        w * 0.62 - 6.0,
                        64.0,
                        30.0,
                        &[
                            (0.0, "#ffffff", 1.0),
                            (0.7, "#d8fff8", 1.0),
                            (1.0, "#9ad8d0", 1.0),
                        ],
                    ),
                );
                blobs(
                    pm,
                    t.sx,
                    (0.0, 30.0, w, 160.0),
                    0.6,
                    2.6,
                    3.0,
                    0.58,
                    0.25,
                    "#0a2630",
                    0.5,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    t.sx,
                    (0.0, 30.0, w, 160.0),
                    0.6,
                    2.6,
                    3.0,
                    0.67,
                    0.2,
                    "#a8fff0",
                    0.2,
                    0.0,
                    false,
                );
            },
        ),

        // 1 far mountains + mist
        lay(
            2.0,
            false,
            760.0,
            240.0,
            0.0,
            0.14,
            45,
            0.14,
            &|pm, t, w, _, _| {
                ridge(pm, t, w, 150.0, 70.0, "#1c4a52", "#3c8a88", 5.0, 0.5);
                ridge(pm, t, w, 180.0, 46.0, "#143a42", "#2a6a6c", 6.0, 0.8);
                haze(pm, t.sx, (0.0, 120.0, w, 120.0), "#9ad8d0", 0.0, 0.6);
                blobs(
                    pm,
                    t.sx,
                    (0.0, 130.0, w, 90.0),
                    0.4,
                    3.0,
                    2.0,
                    0.5,
                    0.3,
                    "#c8fff4",
                    0.3,
                    0.0,
                    true,
                );
            },
        ),

        // 2 the great gate
        lay(
            3.5,
            false,
            800.0,
            240.0,
            0.0,
            0.25,
            41,
            0.25,
            &|pm, t, w, _, _| {
                // spirit forest silhouette behind the gate
                for i in 0..22 {
                    let x = i as f32 * 38.0 + hash(i, 1) * 16.0;
                    pine(pm, t, x, 214.0, 70.0 + hash(i, 2) * 50.0, "#0c2228");
                }
                gate(pm, t, w / 2.0);
                haze(pm, t.sx, (0.0, 170.0, w, 60.0), "#7ad8d0", 0.0, 0.5);
            },
        ),

        // 3 stone lantern row, twisted spirit trees
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.5,
            43,
            0.5,
            &|pm, t, w, _, _| {
                for (i, x) in toro_xs().into_iter().enumerate() {
                    toro(pm, t, x, 212.0);
                    if i % 2 == 1 {
                        let tx = x + 30.0;
                        let bark = solid(rgba("#0c1e22", 1.0));
                        let mut b = PathBuilder::new();
                        b.move_to(tx, 214.0);
                        b.quad_to(tx - 6.0, 170.0, tx + 4.0, 140.0);
                        b.quad_to(tx + 10.0, 120.0, tx - 2.0, 100.0);
                        stroke(pm, t, b.finish(), &bark, 5.0);
                        for k in 0..4 {
                            let a = -1.2 + k as f32 * 0.8;
                            stroke(
                                pm,
                                t,
                                line(
                                    tx + 2.0,
                                    130.0,
                                    tx + 2.0 + a.cos() * 28.0,
                                    130.0 + a.sin() * 24.0,
                                ),
                                &bark,
                                2.0,
                            );
                        }
                    }
                }
                grain(pm, t.sx, (0.0, 90.0, w, 130.0), 4.0, 4.0, 0.3, 3.0);
                haze(pm, t.sx, (0.0, 160.0, w, 60.0), "#6ad0c8", 0.0, 0.3);
            },
        ),
    ]
}
