use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 3 street-level shop fronts (awnings, lit windows, vending machines); the balcony crowd is drawn by stagefx
        lay(
            3.5,
            false,
            1100.0,
            240.0,
            0.0,
            0.8,
            15,
            0.8,
            &|pm, t, w, _, r| {
                let mut x = 0.0;
                let mut i = 0u32;
                while x < w {
                    let bw = 64.0 + r.f() * 50.0;
                    let top = 130.0 + r.f() * 36.0;
                    rect(
                        pm,
                        t,
                        x,
                        top,
                        bw,
                        210.0 - top,
                        &lin(top, 210.0, &[(0.0, "#1a1038"), (1.0, "#0a0618")]),
                    );
                    grain(pm, t.sx, (x, top, bw, 210.0 - top), 3.0, 3.0, 0.35, 3.0);
                    let c = NEON[(hash(i, 3) * 5.0) as usize % 5];
                    rect(
                        pm,
                        t,
                        x + 6.0,
                        176.0,
                        bw - 12.0,
                        30.0,
                        &lin(176.0, 206.0, &[(0.0, "#fff2c8"), (1.0, "#ffb870")]),
                    );
                    rect(
                        pm,
                        t,
                        x + 6.0,
                        176.0,
                        bw - 12.0,
                        30.0,
                        &lin_a(
                            176.0,
                            206.0,
                            &[
                                (0.0, "#000000", 0.5),
                                (0.35, "#000000", 0.0),
                                (1.0, "#000000", 0.35),
                            ],
                        ),
                    );
                    for q in 0..(bw / 14.0) as i32 {
                        rect(
                            pm,
                            t,
                            x + 10.0 + q as f32 * 14.0,
                            184.0,
                            1.0,
                            22.0,
                            &solid(rgba("#1a0c20", 0.7)),
                        );
                    }
                    rect(
                        pm,
                        t,
                        x + 2.0,
                        164.0,
                        bw - 4.0,
                        12.0,
                        &lin(164.0, 176.0, &[(0.0, c), (1.0, "#150a28")]),
                    );
                    for q in 0..((bw - 8.0) / 6.0) as i32 {
                        rect(
                            pm,
                            t,
                            x + 4.0 + q as f32 * 6.0,
                            164.0,
                            3.0,
                            12.0,
                            &solid(rgba("#000000", 0.18)),
                        );
                    }
                    glowc(pm, t, x + bw / 2.0, 190.0, bw * 0.8, "#ffa060", 0.22);
                    stroke(
                        pm,
                        t,
                        line(x + 4.0, top + 6.0, x + bw - 4.0, top + 6.0),
                        &solid(rgba(c, 0.9)),
                        1.0,
                    );
                    x += bw + r.f() * 12.0;
                    i += 1;
                }
                haze(pm, t.sx, (0.0, 120.0, w, 100.0), "#3a1060", 0.0, 0.3);
            },
        ),
    ]
}
