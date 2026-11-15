use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 3 temple wall with sakura trees (students are drawn on top by stagefx)
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.72,
            13,
            0.72,
            &|pm, t, w, _, r| {
                sakura(pm, t, r, 250.0, 208.0, 118.0);
                sakura(pm, t, r, 730.0, 208.0, 104.0);
                sakura(pm, t, r, 940.0, 208.0, 90.0);
                rect(
                    pm,
                    t,
                    0.0,
                    168.0,
                    w,
                    44.0,
                    &lin(168.0, 212.0, &[(0.0, "#4e2c3c"), (1.0, "#2a1624")]),
                );
                grain(pm, t.sx, (0.0, 168.0, w, 44.0), 3.0, 3.0, 0.55, 1.0);
                blobs(
                    pm,
                    t.sx,
                    (0.0, 168.0, w, 44.0),
                    0.35,
                    0.8,
                    9.0,
                    0.6,
                    0.2,
                    "#1c0e18",
                    0.5,
                    0.0,
                    true,
                );
                let mut x = 0.0;
                while x < w {
                    fill(
                        pm,
                        t,
                        ellipse(x + 3.5, 165.0, 3.8, 5.0),
                        &solid(rgba("#1e1018", 1.0)),
                    );
                    rect(pm, t, x, 164.0, 7.0, 4.0, &solid(rgba("#1e1018", 1.0)));
                    x += 7.0;
                }
                rect(pm, t, 0.0, 168.0, w, 1.2, &solid(rgba("#ff9a6a", 0.4)));
                let mut x = 40.0;
                while x < w {
                    rect(
                        pm,
                        t,
                        x,
                        168.0,
                        6.0,
                        44.0,
                        &lin(168.0, 212.0, &[(0.0, "#4a2224"), (1.0, "#2a1214")]),
                    );
                    rect(
                        pm,
                        t,
                        x + 4.5,
                        168.0,
                        1.0,
                        44.0,
                        &solid(rgba("#ff9a6a", 0.2)),
                    );
                    x += 120.0;
                }
            },
        ),

        // 4 wooden engawa deck with railing and paper lanterns
        layer(1100.0, 120.0, 175.0, 1.0, 13, 1.0, &|pm, t, w, h, r| {
            deck(pm, t, w, h, r)
        }),

        // 5 front: temple eave across the top (dark beam + tile row), the lanterns hang from it
        lay(
            3.0,
            true,
            580.0,
            36.0,
            -2.0,
            0.1,
            17,
            0.0,
            &|pm, t, w, h, _| {
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
                        &[(0.0, "#120810"), (0.6, "#1e0e16"), (1.0, "#2a1420")],
                    ),
                );
                let mut x = 0.0;
                while x < w {
                    fill(
                        pm,
                        t,
                        ellipse(x + 5.0, 12.0, 5.2, 8.0),
                        &lin(4.0, 20.0, &[(0.0, "#2c1620"), (1.0, "#0e060a")]),
                    );
                    x += 10.0;
                }
                rect(
                    pm,
                    t,
                    0.0,
                    19.0,
                    w,
                    6.0,
                    &lin(19.0, 25.0, &[(0.0, "#3a1c1c"), (1.0, "#1a0c0c")]),
                );
                rect(pm, t, 0.0, 19.0, w, 0.8, &solid(rgba("#ff9a6a", 0.45)));
                grain(pm, t.sx, (0.0, 19.0, w, 6.0), 0.1, 4.0, 0.6, 2.0);
                rect(
                    pm,
                    t,
                    0.0,
                    25.0,
                    w,
                    11.0,
                    &lin_a(25.0, 36.0, &[(0.0, "#120810", 0.6), (1.0, "#120810", 0.0)]),
                );
            },
        ),

        // 6 dynamic placeholder for foreground banners
        dynamic(560.0, 240.0, 0.0, 1.22, 1.0, true),
    ]
}
