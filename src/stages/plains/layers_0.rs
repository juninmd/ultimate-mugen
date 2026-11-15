use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 0 desert sky: gradient, sun, cloud noise
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.04,
            8,
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
                            (0.0, "#1f5fc8"),
                            (0.45, "#6ab0ee"),
                            (0.8, "#c8e4f4"),
                            (1.0, "#f4ecd8"),
                        ],
                    ),
                );
                let r = t.sx;
                glowc(pm, t, w * 0.78, 40.0, 100.0, "#fff4c0", 0.8);
                fill(
                    pm,
                    t,
                    circle(w * 0.78, 40.0, 11.0),
                    &solid(rgba("#ffffff", 1.0)),
                );
                blobs(
                    pm,
                    r,
                    (0.0, 10.0, w, 170.0),
                    0.7,
                    2.4,
                    1.0,
                    0.55,
                    0.22,
                    "#ffffff",
                    0.85,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    r,
                    (0.0, 10.0, w, 170.0),
                    0.7,
                    2.4,
                    1.0,
                    0.7,
                    0.2,
                    "#9ab0d0",
                    0.35,
                    0.0,
                    false,
                );
            },
        ),

        // 1 far mesas
        lay(
            2.0,
            false,
            760.0,
            240.0,
            0.0,
            0.14,
            31,
            0.14,
            &|pm, t, w, _, r| {
                let mut x = 0.0;
                while x < w {
                    let mw = 60.0 + r.f() * 120.0;
                    let mh = 24.0 + r.f() * 44.0;
                    let by = 178.0;
                    fill(
                        pm,
                        t,
                        poly(&[
                            (x, by + 30.0),
                            (x + 6.0, by - mh * 0.7),
                            (x + 10.0, by - mh),
                            (x + mw - 10.0, by - mh),
                            (x + mw - 4.0, by - mh * 0.6),
                            (x + mw, by + 30.0),
                        ]),
                        &lin(by - mh, by + 10.0, &[(0.0, "#c9a898"), (1.0, "#a8847c")]),
                    );
                    for k in 0..4 {
                        rect(
                            pm,
                            t,
                            x + 6.0,
                            by - mh + 6.0 + k as f32 * (mh / 4.0),
                            mw - 12.0,
                            1.0,
                            &solid(rgba("#8a6a64", 0.4)),
                        );
                    }
                    x += mw * 0.7 + r.f() * 60.0;
                }
                ridge(pm, t, w, 196.0, 26.0, "#bf9a86", "#d8c0a8", 4.0, 0.7);
                haze(pm, t.sx, (0.0, 120.0, w, 120.0), "#e8e0d0", 0.2, 0.7);
            },
        ),

        // 2 mid canyon walls: lit / shadow faces, strata, erosion grooves
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.4,
            33,
            0.4,
            &|pm, t, w, _, r| {
                let mut x = -20.0;
                while x < w {
                    let mh = 60.0 + r.f() * 86.0;
                    let mw = 56.0 + r.f() * 56.0;
                    let top = 198.0 - mh;
                    let split = x + mw * (0.45 + r.f() * 0.15);
                    let sil = [
                        (x, 212.0),
                        (x + 4.0, 196.0 - mh * 0.85),
                        (x + 13.0, top + 2.0),
                        (split, top),
                        (x + mw - 7.0, 196.0 - mh * 0.7),
                        (x + mw, 212.0),
                    ];
                    fill(
                        pm,
                        t,
                        poly(&[sil[0], sil[1], sil[2], sil[3], (split - 2.0, 212.0)]),
                        &lin(
                            top,
                            212.0,
                            &[(0.0, "#e8a874"), (0.5, "#c47c50"), (1.0, "#8a5036")],
                        ),
                    );
                    fill(
                        pm,
                        t,
                        poly(&[(split - 2.0, 212.0), sil[3], sil[4], sil[5]]),
                        &lin(
                            top,
                            212.0,
                            &[(0.0, "#a8684a"), (0.5, "#8a4c34"), (1.0, "#5a3022")],
                        ),
                    );
                    let mut y = top + 8.0;
                    let mut k = 0;
                    while y < 210.0 {
                        let col = if k % 2 == 0 {
                            rgba("#4a2214", 0.4)
                        } else {
                            rgba("#f4cc9c", 0.25)
                        };
                        stroke(
                            pm,
                            t,
                            line(x + mw * 0.12, y, x + mw * 0.88, y + (r.f() - 0.5) * 2.0),
                            &solid(col),
                            0.9,
                        );
                        y += 4.0 + r.f() * 7.0;
                        k += 1;
                    }
                    for _ in 0..3 {
                        let gx = x + mw * (0.2 + r.f() * 0.6);
                        let gy = top + 8.0 + r.f() * mh * 0.4;
                        stroke(
                            pm,
                            t,
                            line(gx, gy, gx + (r.f() - 0.5) * 4.0, gy + 10.0 + r.f() * 16.0),
                            &solid(rgba("#3a1a0e", 0.22)),
                            0.7,
                        );
                    }
                    stroke(
                        pm,
                        t,
                        poly(&[sil[1], sil[2], sil[3]]),
                        &solid(rgba("#ffe6c0", 0.6)),
                        0.9,
                    );
                    x += mw * 0.8 + r.f() * 60.0;
                }
                grain(pm, t.sx, (0.0, 80.0, w, 140.0), 6.0, 6.0, 0.35, 2.0);
                haze(pm, t.sx, (0.0, 100.0, w, 112.0), "#f0e0c8", 0.0, 0.22);
            },
        ),
    ]
}
