use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 0 ash sky glowing from below
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.04,
            50,
            0.04,
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
                        &[
                            (0.0, "#080202"),
                            (0.45, "#3a0c06"),
                            (0.8, "#a82a10"),
                            (1.0, "#e8581e"),
                        ],
                    ),
                );
                let r = t.sx;
                blobs(
                    pm,
                    r,
                    (0.0, 0.0, w, h),
                    0.6,
                    2.2,
                    1.0,
                    0.5,
                    0.3,
                    "#140404",
                    0.75,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    r,
                    (0.0, 60.0, w, 180.0),
                    0.7,
                    2.6,
                    6.0,
                    0.6,
                    0.2,
                    "#ff6a1e",
                    0.35,
                    0.0,
                    false,
                );
            },
        ),

        // 1 far volcano cone with crater glow and lava streaks
        lay(
            2.0,
            false,
            760.0,
            240.0,
            0.0,
            0.14,
            51,
            0.14,
            &|pm, t, w, _, r| {
                ridge(pm, t, w, 190.0, 50.0, "#2c0c08", "#4a1810", 2.0, 0.6);
                let cx = w * 0.55;
                fill(
                    pm,
                    t,
                    poly(&[
                        (cx - 190.0, 215.0),
                        (cx - 40.0, 96.0),
                        (cx - 18.0, 90.0),
                        (cx + 18.0, 90.0),
                        (cx + 44.0, 98.0),
                        (cx + 200.0, 215.0),
                    ]),
                    &lin(90.0, 215.0, &[(0.0, "#2a0a06"), (1.0, "#4a1a10")]),
                );
                glowc(pm, t, cx, 90.0, 80.0, "#ff7a2a", 0.9);
                fill(
                    pm,
                    t,
                    ellipse(cx, 91.0, 20.0, 4.0),
                    &solid(rgba("#ffd070", 1.0)),
                );
                for k in 0..9 {
                    let (mut x, mut y) = (cx + (r.f() - 0.5) * 24.0, 92.0);
                    let mut b = PathBuilder::new();
                    b.move_to(x, y);
                    for _ in 0..5 {
                        x += (r.f() - 0.5) * 22.0 + (k as f32 - 4.0) * 3.0;
                        y += 14.0 + r.f() * 10.0;
                        b.line_to(x, y);
                    }
                    let p = b.finish();
                    stroke(pm, t, p.clone(), &solid(rgba("#ff5000", 0.35)), 3.0);
                    stroke(pm, t, p, &solid(rgba("#ffa040", 0.95)), 0.9);
                }
                grain(pm, t.sx, (0.0, 80.0, w, 140.0), 3.0, 3.0, 0.5, 2.0);
                haze(pm, t.sx, (0.0, 150.0, w, 90.0), "#c8381a", 0.0, 0.5);
            },
        ),

        // 2 lava river + jagged ridge
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.45,
            52,
            0.45,
            &|pm, t, w, _, r| {
                ridge(pm, t, w, 196.0, 56.0, "#1c0806", "#2c0e0a", 7.0, 0.9);
                let mut x = 0.0;
                while x < w {
                    let hh = 30.0 + r.f() * 60.0;
                    fill(
                        pm,
                        t,
                        poly(&[
                            (x, 214.0),
                            (x + 12.0, 204.0 - hh),
                            (x + 16.0, 207.0 - hh),
                            (x + 32.0, 214.0),
                        ]),
                        &lin(205.0 - hh, 214.0, &[(0.0, "#2a0e0a"), (1.0, "#120504")]),
                    );
                    stroke(
                        pm,
                        t,
                        line(x + 12.0, 204.0 - hh, x + 16.0, 207.0 - hh),
                        &solid(rgba("#ff8a40", 0.6)),
                        0.9,
                    );
                    x += 40.0 + r.f() * 60.0;
                }
                // lava river band
                let mut b = PathBuilder::new();
                b.move_to(0.0, 214.0);
                let mut xx = 0.0;
                while xx <= w {
                    b.line_to(xx, 200.0 + n1(xx * 0.3, 9.0) * 8.0);
                    xx += 4.0;
                }
                b.line_to(w, 220.0);
                b.line_to(0.0, 220.0);
                b.close();
                fill(
                    pm,
                    t,
                    b.finish(),
                    &lin(
                        196.0,
                        220.0,
                        &[(0.0, "#ffe070"), (0.4, "#ff8a1e"), (1.0, "#b82a0a")],
                    ),
                );
                blobs(
                    pm,
                    t.sx,
                    (0.0, 195.0, w, 25.0),
                    0.5,
                    3.0,
                    4.0,
                    0.55,
                    0.25,
                    "#3a0a04",
                    0.5,
                    0.0,
                    true,
                );
                glowc(pm, t, w * 0.25, 204.0, 70.0, "#ff6a1e", 0.5);
                glowc(pm, t, w * 0.75, 204.0, 70.0, "#ff6a1e", 0.5);
                grain(pm, t.sx, (0.0, 100.0, w, 120.0), 3.0, 3.0, 0.4, 7.0);
            },
        ),

        // 3 near basalt pillars with glowing seams
        lay(
            3.5,
            false,
            1100.0,
            240.0,
            0.0,
            0.75,
            54,
            0.75,
            &|pm, t, w, _, r| {
                let mut x = 20.0;
                while x < w {
                    let ww = 34.0 + r.f() * 40.0;
                    let (bh, gl) = (34.0 + r.f() * 60.0, r.f() < 0.5);
                    basalt(pm, t, r, x, 212.0, ww, bh, gl);
                    x += ww + 70.0 + r.f() * 160.0;
                }
                glowc(pm, t, w * 0.5, 214.0, 140.0, "#ff6a1e", 0.25);
                grain(pm, t.sx, (0.0, 100.0, w, 120.0), 4.0, 4.0, 0.45, 8.0);
            },
        ),
    ]
}
