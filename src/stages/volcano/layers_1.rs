use super::*;

pub(super) fn layers(cracks: &[Vec<(f32, f32)>]) -> Vec<Layer> {
    vec![
        // 4 obsidian slab floor with glowing seams
        layer(
            1100.0,
            120.0,
            175.0,
            1.0,
            53,
            1.0,
            &move |pm, t, w, h, r| {
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
                        &[(0.0, "#3a1c16"), (0.35, "#1c0a08"), (1.0, "#0a0302")],
                    ),
                );
                // hexagonal basalt slabs
                let n = 6;
                for i in 0..n {
                    let (y0, y1) = (
                        30.0 + (i as f32 / n as f32).powf(1.5) * 90.0,
                        30.0 + ((i + 1) as f32 / n as f32).powf(1.5) * 90.0,
                    );
                    let sw = 60.0 * (0.7 + i as f32 * 0.3);
                    let mut x = -hash(i, 4) * sw;
                    while x < w {
                        let tone = hash((x / 5.0) as u32, i);
                        rect(
                            pm,
                            t,
                            x + 0.5,
                            y0 + 0.3,
                            sw - 1.0,
                            y1 - y0 - 0.6,
                            &lin(
                                y0,
                                y1,
                                &[
                                    (0.0, if tone < 0.5 { "#4a2a24" } else { "#3a201c" }),
                                    (1.0, "#160807"),
                                ],
                            ),
                        );
                        rect(
                            pm,
                            t,
                            x + 0.5,
                            y0 + 0.3,
                            sw - 1.0,
                            0.8,
                            &solid(rgba("#ffb080", 0.18)),
                        );
                        x += sw;
                    }
                }
                grain(pm, rr, (0.0, 28.0, w, h - 28.0), 6.0, 6.0, 0.7, 2.0);
                blobs(
                    pm,
                    rr,
                    (0.0, 28.0, w, h - 28.0),
                    0.3,
                    1.0,
                    4.0,
                    0.6,
                    0.3,
                    "#000000",
                    0.5,
                    0.0,
                    true,
                );
                for pl in cracks {
                    let mut b = PathBuilder::new();
                    b.move_to(pl[0].0, pl[0].1);
                    for p in &pl[1..] {
                        b.line_to(p.0, p.1);
                    }
                    let path = b.finish();
                    stroke(pm, t, path.clone(), &solid(rgba("#ff4a00", 0.22)), 4.0);
                    stroke(pm, t, path.clone(), &solid(rgba("#ff8a30", 0.7)), 1.5);
                    stroke(pm, t, path, &solid(rgba("#ffe090", 0.95)), 0.5);
                }
                shade_v(pm, rr, (0.0, 28.0, w, h - 28.0), 0.9, 1.0);
                // far edge: cooled lava ridge
                rect(
                    pm,
                    t,
                    0.0,
                    6.0,
                    w,
                    24.0,
                    &lin(6.0, 30.0, &[(0.0, "#1c0a08"), (1.0, "#3a1a14")]),
                );
                let mut x = 0.0;
                while x < w {
                    let hh = 6.0 + r.f() * 14.0;
                    fill(
                        pm,
                        t,
                        poly(&[
                            (x, 30.0),
                            (x + 3.0, 30.0 - hh),
                            (x + 9.0 + r.f() * 6.0, 30.0),
                        ]),
                        &solid(rgba("#140807", 1.0)),
                    );
                    x += 7.0;
                }
                rect(pm, t, 0.0, 28.5, w, 1.4, &solid(rgba("#ff7a30", 0.8)));
            },
        ),

        // 5 front: basalt silhouettes rimmed with lava light
        lay(
            3.5,
            true,
            560.0,
            240.0,
            0.0,
            1.25,
            55,
            1.0,
            &|pm, t, w, h, r| {
                basalt(pm, t, r, -10.0, h + 4.0, 56.0, 84.0, true);
                basalt(pm, t, r, w - 50.0, h + 4.0, 56.0, 96.0, true);
            },
        ),
    ]
}
