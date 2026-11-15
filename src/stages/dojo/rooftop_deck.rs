use super::*;

pub(super) fn deck(pm: &mut Pixmap, t: Transform, w: f32, h: f32, rng: &mut Rng) {
    let top = 30.0;
    rect(pm, t, 0.0, 24.0, w, h, &solid(rgba("#2a1410", 1.0)));
    let n = 8;
    let tones = ["#b8754f", "#a96a44", "#c58259", "#9c5f3c", "#b06d48"];
    for i in 0..n {
        let (y0, y1) = (
            top + (i as f32 / n as f32).powf(1.6) * (h - top),
            top + ((i + 1) as f32 / n as f32).powf(1.6) * (h - top),
        );
        let scale = 0.6 + (i as f32 / n as f32) * 1.5;
        let bw = 95.0 * scale;
        let mut x = -rng.f() * bw;
        while x < w {
            let c = tones[(rng.f() * 5.0) as usize % 5];
            rect(
                pm,
                t,
                x + 0.5,
                y0 + 0.2,
                bw - 1.0,
                y1 - y0 - 0.4,
                &lin(y0, y1, &[(0.0, c), (1.0, "#8a5434")]),
            );
            rect(
                pm,
                t,
                x + 0.5,
                y0 + 0.2,
                bw - 1.0,
                0.9,
                &solid(rgba("#ffe0b0", 0.28)),
            );
            rect(pm, t, x, y0, 0.9, y1 - y0, &solid(rgba("#1a0a08", 0.8)));
            for nx in [x + 5.0, x + bw - 6.0] {
                fill(
                    pm,
                    t,
                    circle(nx, (y0 + y1) / 2.0, 0.7 * scale.max(0.7)),
                    &solid(rgba("#2a1008", 0.8)),
                );
            }
            if rng.f() < 0.25 {
                stroke(
                    pm,
                    t,
                    ellipse(
                        x + bw * (0.2 + rng.f() * 0.6),
                        (y0 + y1) / 2.0,
                        3.0 * scale,
                        1.1 * scale,
                    ),
                    &solid(rgba("#4a2410", 0.6)),
                    0.6,
                );
            }
            x += bw;
        }
        rect(pm, t, 0.0, y1 - 0.5, w, 1.0, &solid(rgba("#1a0a08", 0.75)));
    }
    let r = t.sx;
    grain(pm, r, (0.0, top, w, h - top), 0.07, 5.0, 0.6, 3.0);
    blobs(
        pm,
        r,
        (0.0, top, w, h - top),
        0.12,
        3.5,
        5.0,
        0.55,
        0.25,
        "#2a1008",
        0.5,
        0.0,
        true,
    );
    blobs(
        pm,
        r,
        (0.0, top, w, h - top),
        0.2,
        3.0,
        8.0,
        0.7,
        0.2,
        "#ffd9a8",
        0.28,
        0.0,
        true,
    );
    shade_v(pm, r, (0.0, top, w, h - top), 0.72, 1.15);
    // railing: beams, posts with caps, kick-board, hanging paper lanterns
    let wood = lin(0.0, 30.0, &[(0.0, "#5c3226"), (1.0, "#2c1612")]);
    rect(pm, t, 0.0, 22.0, w, 8.0, &solid(rgba("#2a1410", 1.0)));
    rect(pm, t, 0.0, 14.0, w, 3.4, &wood);
    rect(pm, t, 0.0, 14.0, w, 0.8, &solid(rgba("#ffb088", 0.35)));
    rect(pm, t, 0.0, 22.0, w, 2.0, &wood);
    for x in dojo_lanterns() {
        let px = x - 2.5;
        rect(
            pm,
            t,
            px,
            3.0,
            5.0,
            29.0,
            &lin(3.0, 32.0, &[(0.0, "#4a2a20"), (1.0, "#22100c")]),
        );
        rect(pm, t, px - 1.2, 1.6, 7.4, 2.6, &solid(rgba("#6a3a2c", 1.0)));
        rect(
            pm,
            t,
            px + 3.6,
            3.0,
            1.0,
            29.0,
            &solid(rgba("#ffb088", 0.2)),
        );
        glowc(pm, t, x, 12.0, 17.0, "#ff8a3a", 0.35);
        stroke(
            pm,
            t,
            line(x, 4.0, x, 6.0),
            &solid(rgba("#1a0c0a", 1.0)),
            0.6,
        );
        fill(
            pm,
            t,
            ellipse(x, 12.0, 5.2, 7.0),
            &radial(
                x,
                12.0,
                7.0,
                &[
                    (0.0, "#fff0b0", 1.0),
                    (0.45, "#ff7a30", 1.0),
                    (1.0, "#b8281c", 1.0),
                ],
            ),
        );
        for k in 0..4 {
            stroke(
                pm,
                t,
                line(
                    x - 5.0 + k as f32 * 3.3,
                    12.0 - 6.0 * (1.0 - ((k as f32 - 1.5) / 2.0).powi(2)).sqrt(),
                    x - 5.0 + k as f32 * 3.3,
                    12.0 + 6.0 * (1.0 - ((k as f32 - 1.5) / 2.0).powi(2)).sqrt(),
                ),
                &solid(rgba("#7a1810", 0.5)),
                0.35,
            );
        }
        rect(pm, t, x - 3.0, 4.6, 6.0, 1.4, &solid(rgba("#1c0c0a", 1.0)));
        rect(pm, t, x - 3.0, 18.4, 6.0, 1.4, &solid(rgba("#1c0c0a", 1.0)));
    }
}
