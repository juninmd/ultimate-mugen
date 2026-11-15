use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 0 moonlit sky: moon, stars, thin clouds
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.04,
            2,
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
                        &[(0.0, "#030d1a"), (0.55, "#0f3646"), (1.0, "#2c6a6a")],
                    ),
                );
                for _ in 0..110 {
                    let a = 0.2 + r.f() * 0.7;
                    fill(
                        pm,
                        t,
                        circle(r.f() * w, r.f() * 130.0, 0.35 + r.f() * 0.5),
                        &solid(rgba("#dffcff", a)),
                    );
                }
                glowc(pm, t, w * 0.35, 55.0, 90.0, "#bff4ee", 0.55);
                fill(
                    pm,
                    t,
                    circle(w * 0.35, 55.0, 17.0),
                    &radial(
                        w * 0.35 - 4.0,
                        51.0,
                        20.0,
                        &[
                            (0.0, "#ffffff", 1.0),
                            (0.7, "#e6fff8", 1.0),
                            (1.0, "#b8e8e0", 1.0),
                        ],
                    ),
                );
                for (dx, dy, rr) in [
                    (-5.0, -4.0, 3.2),
                    (4.0, 3.0, 4.2),
                    (-2.0, 7.0, 2.0),
                    (6.0, -6.0, 2.2),
                ] {
                    fill(
                        pm,
                        t,
                        circle(w * 0.35 + dx, 55.0 + dy, rr),
                        &solid(rgba("#9fd0c8", 0.4)),
                    );
                }
                blobs(
                    pm,
                    t.sx,
                    (0.0, 20.0, w, 140.0),
                    0.6,
                    2.8,
                    4.0,
                    0.56,
                    0.22,
                    "#0a2a3a",
                    0.5,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    t.sx,
                    (0.0, 20.0, w, 140.0),
                    0.6,
                    2.8,
                    4.0,
                    0.64,
                    0.2,
                    "#9fe6e0",
                    0.22,
                    0.0,
                    false,
                );
            },
        ),

        // 1 far bamboo silhouettes in mist
        lay(
            2.0,
            false,
            900.0,
            260.0,
            -10.0,
            0.18,
            21,
            0.18,
            &|pm, t, w, _, r| {
                let mut x = 0.0;
                while x < w {
                    let bw = 2.5 + r.f() * 2.0;
                    let c = solid(rgba("#194a4a", 1.0));
                    rect(pm, t, x, 0.0, bw, 260.0, &c);
                    rect(
                        pm,
                        t,
                        x + bw * 0.6,
                        0.0,
                        bw * 0.3,
                        260.0,
                        &solid(rgba("#a0e6dc", 0.22)),
                    );
                    let mut y = 24.0 + r.f() * 24.0;
                    while y < 240.0 {
                        rect(
                            pm,
                            t,
                            x - 0.6,
                            y,
                            bw + 1.2,
                            1.2,
                            &solid(rgba("#0e3434", 1.0)),
                        );
                        y += 22.0 + r.f() * 10.0;
                    }
                    x += 8.0 + r.f() * 14.0;
                }
                haze(pm, t.sx, (0.0, 0.0, w, 260.0), "#3a8a88", 0.35, 0.6);
            },
        ),

        // 2 dynamic swaying mid grove, 3 dynamic near grove
        dynamic(900.0, 260.0, -10.0, 0.42, 0.42, false),

        dynamic(1100.0, 260.0, -10.0, 0.72, 0.72, false),
    ]
}
