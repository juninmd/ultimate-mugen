use super::*;

pub(super) fn gate(pm: &mut Pixmap, t: Transform, cx: f32) {
    let stone = lin(60.0, 214.0, &[(0.0, "#2a4a50"), (1.0, "#0c1c22")]);
    // pillars with round bases, stacked beams (kasagi + nuki), shimenawa rope
    for sx in [-1.0f32, 1.0] {
        let px = cx + sx * 98.0;
        rect(pm, t, px - 11.0, 62.0, 22.0, 150.0, &stone);
        rect(
            pm,
            t,
            px - 14.0,
            190.0,
            28.0,
            24.0,
            &lin(190.0, 214.0, &[(0.0, "#305458"), (1.0, "#0e2228")]),
        );
        rect(
            pm,
            t,
            px - 11.0 + if sx < 0.0 { 15.0 } else { 0.0 },
            62.0,
            4.0,
            130.0,
            &solid(rgba("#9ad8d0", 0.16)),
        );
    }
    let mut b = PathBuilder::new();
    b.move_to(cx - 154.0, 54.0);
    b.quad_to(cx, 24.0, cx + 154.0, 54.0);
    b.line_to(cx + 142.0, 68.0);
    b.quad_to(cx, 44.0, cx - 142.0, 68.0);
    b.close();
    fill(
        pm,
        t,
        b.finish(),
        &lin(30.0, 68.0, &[(0.0, "#3a6068"), (1.0, "#12282e")]),
    );
    rect(
        pm,
        t,
        cx - 124.0,
        80.0,
        248.0,
        11.0,
        &lin(80.0, 91.0, &[(0.0, "#2e5258"), (1.0, "#10242a")]),
    );
    rect(
        pm,
        t,
        cx - 124.0,
        80.0,
        248.0,
        1.0,
        &solid(rgba("#a0fff0", 0.35)),
    );
    let mut b = PathBuilder::new();
    b.move_to(cx - 98.0, 97.0);
    b.quad_to(cx, 116.0, cx + 98.0, 97.0);
    stroke(pm, t, b.finish(), &solid(rgba("#c8b88a", 1.0)), 4.0);
    for k in 0..7 {
        let f = k as f32 / 6.0;
        let x = cx - 98.0 + f * 196.0;
        let y = 97.0 + 19.0 * 2.0 * f * (1.0 - f) * 0.9;
        fill(
            pm,
            t,
            poly(&[
                (x - 2.5, y + 2.0),
                (x + 2.5, y + 2.0),
                (x + 1.5, y + 11.0),
                (x - 1.5, y + 8.0),
            ]),
            &solid(rgba("#f4fff8", 0.85)),
        );
    }
    // inner portal
    rect(
        pm,
        t,
        cx - 87.0,
        91.0,
        174.0,
        123.0,
        &radial(
            cx,
            150.0,
            96.0,
            &[
                (0.0, "#d8fffa", 0.6),
                (0.5, "#5ac8c8", 0.3),
                (1.0, "#2a8898", 0.0),
            ],
        ),
    );
    grain(
        pm,
        t.sx,
        (cx - 160.0, 24.0, 320.0, 190.0),
        4.0,
        4.0,
        0.35,
        3.0,
    );
    blobs(
        pm,
        t.sx,
        (cx - 160.0, 24.0, 320.0, 190.0),
        0.5,
        1.8,
        3.0,
        0.6,
        0.2,
        "#10303a",
        0.45,
        0.0,
        true,
    );
    // moss
    blobs(
        pm,
        t.sx,
        (cx - 160.0, 150.0, 320.0, 64.0),
        0.6,
        1.6,
        9.0,
        0.5,
        0.25,
        "#2a7a5a",
        0.4,
        0.0,
        true,
    );
}
pub(super) fn toro(pm: &mut Pixmap, t: Transform, x: f32, by: f32) {
    let c = lin(by - 50.0, by, &[(0.0, "#4a7078"), (1.0, "#1a343a")]);
    rect(pm, t, x - 5.0, by - 12.0, 10.0, 12.0, &c);
    rect(pm, t, x - 2.5, by - 26.0, 5.0, 16.0, &c);
    rect(pm, t, x - 8.0, by - 30.0, 16.0, 4.0, &c);
    rect(
        pm,
        t,
        x - 6.0,
        by - 42.0,
        12.0,
        12.0,
        &solid(rgba("#142a30", 1.0)),
    );
    rect(
        pm,
        t,
        x - 4.0,
        by - 40.0,
        8.0,
        8.0,
        &radial(
            x,
            by - 36.0,
            6.0,
            &[
                (0.0, "#ffffff", 1.0),
                (0.5, "#b8fff0", 1.0),
                (1.0, "#4ad8c8", 1.0),
            ],
        ),
    );
    fill(
        pm,
        t,
        poly(&[(x - 11.0, by - 42.0), (x, by - 52.0), (x + 11.0, by - 42.0)]),
        &c,
    );
    fill(pm, t, circle(x, by - 54.0, 1.8), &c);
}
