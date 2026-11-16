use super::*;

pub(super) fn layers() -> Vec<Layer> {
    vec![
        // 4 polished stone floor with carved runes
        layer(1100.0, 120.0, 175.0, 1.0, 44, 1.0, &|pm, t, w, h, r| {
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
                    &[(0.0, "#a8c0bc"), (0.2, "#6a8684"), (1.0, "#2a3c3c")],
                ),
            );
            // stone slabs
            let n = 7;
            for i in 0..n {
                let (y0, y1) = (
                    30.0 + (i as f32 / n as f32).powf(1.5) * 90.0,
                    30.0 + ((i + 1) as f32 / n as f32).powf(1.5) * 90.0,
                );
                let sw = 70.0 * (0.7 + i as f32 * 0.25);
                let mut x = -(hash(i, 5) * sw);
                while x < w {
                    let tone = hash((x / 7.0) as u32, i + 9);
                    rect(
                        pm,
                        t,
                        x + 0.4,
                        y0 + 0.2,
                        sw - 0.8,
                        y1 - y0 - 0.4,
                        &solid(rgba(
                            if tone < 0.5 { "#aac4c0" } else { "#8aa8a6" },
                            0.18 * (1.0 + i as f32 * 0.1),
                        )),
                    );
                    rect(pm, t, x, y0, 0.9, y1 - y0, &solid(rgba("#10262a", 0.7)));
                    x += sw;
                }
                rect(pm, t, 0.0, y1 - 0.4, w, 0.9, &solid(rgba("#10262a", 0.7)));
                rect(pm, t, 0.0, y0 + 0.5, w, 0.6, &solid(rgba("#e8fffa", 0.28)));
            }
            grain(pm, rr, (0.0, 28.0, w, h - 28.0), 6.0, 6.0, 0.5, 2.0);
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.25,
                1.1,
                4.0,
                0.55,
                0.3,
                "#16302e",
                0.5,
                0.0,
                true,
            );
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.45,
                1.6,
                9.0,
                0.6,
                0.25,
                "#2a8a6a",
                0.35,
                0.0,
                true,
            );
            for _ in 0..24 {
                let x = r.f() * w;
                let y = 34.0 + r.f() * 80.0;
                let mut b = PathBuilder::new();
                b.move_to(x, y);
                b.line_to(x + (r.f() - 0.5) * 20.0, y + (r.f() - 0.5) * 6.0);
                stroke(pm, t, b.finish(), &solid(rgba("#0a1c20", 0.6)), 0.5);
            }
            // rune glyph rings every 270 units
            for k in 0..4 {
                let cx = 120.0 + k as f32 * 290.0;
                let cy = 74.0;
                for (rx, ry) in [(52.0, 11.0), (36.0, 7.5)] {
                    stroke(
                        pm,
                        t,
                        ellipse(cx, cy, rx, ry),
                        &solid(rgba("#5affea", 0.28)),
                        0.8,
                    );
                }
                for q in 0..12 {
                    let a = q as f32 / 12.0 * std::f32::consts::TAU;
                    let (x, y) = (cx + a.cos() * 44.0, cy + a.sin() * 9.0);
                    stroke(
                        pm,
                        t,
                        line(x - 1.5, y - 1.0, x + 1.5, y + 1.0 + (q % 3) as f32),
                        &solid(rgba("#7affee", 0.5)),
                        0.7,
                    );
                }
            }
            shade_v(pm, rr, (0.0, 28.0, w, h - 28.0), 0.86, 1.0);
            // top: low stone wall + steps
            rect(
                pm,
                t,
                0.0,
                12.0,
                w,
                18.0,
                &lin(12.0, 30.0, &[(0.0, "#5a7a78"), (1.0, "#1c3030")]),
            );
            let mut x = 0.0;
            while x < w {
                rect(pm, t, x, 12.0, 0.8, 18.0, &solid(rgba("#0a1c20", 0.7)));
                x += 30.0 + hash((x / 30.0) as u32, 6) * 24.0;
            }
            rect(pm, t, 0.0, 12.0, w, 1.6, &solid(rgba("#d8fff8", 0.4)));
            grain(pm, rr, (0.0, 12.0, w, 18.0), 5.0, 5.0, 0.5, 4.0);
        }),

        // 5 dynamic foreground: hanging charms
        dynamic(560.0, 240.0, 0.0, 1.2, 1.0, true),
    ]
}
