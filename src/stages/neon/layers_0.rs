use super::*;

pub(super) fn layers(signs: &[(f32, f32, f32, f32, usize)]) -> Vec<Layer> {
    vec![
        // 0 smoggy night sky with a ringed planet
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.05,
            2,
            0.05,
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
                        &[
                            (0.0, "#04020e"),
                            (0.5, "#1a0a38"),
                            (0.8, "#3d1260"),
                            (1.0, "#6a1c70"),
                        ],
                    ),
                );
                let rr = t.sx;
                for _ in 0..90 {
                    let a = r.f() * 0.7;
                    rect(
                        pm,
                        t,
                        r.f() * w,
                        r.f() * 120.0,
                        0.7,
                        0.7,
                        &solid(rgba("#ffffff", a)),
                    );
                }
                glowc(pm, t, w * 0.72, 62.0, 56.0, "#ff3cc8", 0.4);
                fill(
                    pm,
                    t,
                    circle(w * 0.72, 62.0, 22.0),
                    &radial(
                        w * 0.72 - 6.0,
                        56.0,
                        28.0,
                        &[
                            (0.0, "#ffb0e8", 1.0),
                            (0.6, "#a02a98", 1.0),
                            (1.0, "#3a0e50", 1.0),
                        ],
                    ),
                );
                stroke(
                    pm,
                    t,
                    ellipse(w * 0.72, 62.0, 40.0, 7.0),
                    &solid(rgba("#ffd0f4", 0.5)),
                    1.4,
                );
                blobs(
                    pm,
                    rr,
                    (0.0, 20.0, w, 200.0),
                    0.6,
                    2.6,
                    2.0,
                    0.5,
                    0.3,
                    "#ff3cc8",
                    0.22,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    rr,
                    (0.0, 60.0, w, 160.0),
                    0.7,
                    2.8,
                    5.0,
                    0.55,
                    0.25,
                    "#22d8ff",
                    0.16,
                    0.0,
                    false,
                );
            },
        ),

        // 1 far skyline
        lay(
            2.5,
            false,
            760.0,
            240.0,
            0.0,
            0.2,
            4,
            0.2,
            &|pm, t, w, _, _| {
                for (x, bw, bh) in skyline(21, w, 16.0, 34.0, 60.0, 170.0, 8.0) {
                    tower(pm, t, x, bw, bh, 200.0, "#1c1040", 3, 0.38);
                }
                haze(pm, t.sx, (0.0, 60.0, w, 160.0), "#7a1c88", 0.0, 0.55);
            },
        ),

        // 2 mid city with neon sign boards
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.5,
            9,
            0.5,
            &|pm, t, w, _, _| {
                for (x, bw, bh) in skyline(41, w, 28.0, 56.0, 90.0, 190.0, 54.0) {
                    tower(pm, t, x, bw, bh, 205.0, "#150c2c", 5, 0.2);
                    rect(pm, t, x, 205.0 - bh, bw, 1.4, &solid(rgba("#6a50b0", 0.5)));
                    grain(pm, t.sx, (x, 205.0 - bh, bw, bh), 4.0, 4.0, 0.3, 2.0);
                    for k in 0..2 {
                        rect(
                            pm,
                            t,
                            x + 4.0 + k as f32 * (bw - 12.0),
                            205.0 - bh - 5.0,
                            1.2,
                            5.0,
                            &solid(rgba("#33255a", 1.0)),
                        );
                    }
                }
                for (x, y, sw, sh, c) in signs {
                    let col = NEON[*c];
                    glowc(
                        pm,
                        t,
                        x + sw / 2.0,
                        y + sh / 2.0,
                        sh.max(*sw) * 1.1,
                        col,
                        0.42,
                    );
                    let rp =
                        tiny_skia::Rect::from_xywh(*x, *y, *sw, *sh).map(PathBuilder::from_rect);
                    rect(pm, t, *x, *y, *sw, *sh, &solid(rgba("#0a0618", 0.9)));
                    stroke(pm, t, rp.clone(), &solid(rgba(col, 0.3)), 4.0);
                    stroke(pm, t, rp, &solid(rgba(col, 1.0)), 1.2);
                    for q in 0..3 {
                        stroke(
                            pm,
                            t,
                            line(
                                x + 2.0,
                                y + 4.0 + q as f32 * sh / 3.3,
                                x + sw - 2.0,
                                y + 6.0 + q as f32 * sh / 3.3,
                            ),
                            &solid(rgba(col, 1.0)),
                            1.1,
                        );
                    }
                }
                haze(pm, t.sx, (0.0, 130.0, w, 90.0), "#5a1c80", 0.0, 0.38);
            },
        ),
    ]
}
