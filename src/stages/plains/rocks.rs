use super::*;

pub(super) fn cactus(pm: &mut Pixmap, t: Transform, x: f32, by: f32, h: f32) {
    let g = lin(by - h, by, &[(0.0, "#5a7a3a"), (1.0, "#2e4a24")]);
    rect(pm, t, x - 3.0, by - h, 6.0, h, &g);
    fill(pm, t, ellipse(x, by - h, 3.0, 3.0), &g);
    for (dx, up) in [(-9.0, h * 0.55), (9.0, h * 0.4)] {
        rect(pm, t, x.min(x + dx), by - up, dx.abs(), 3.4, &g);
        rect(pm, t, x + dx - 1.7, by - up - 12.0, 3.4, 12.0 + 3.0, &g);
        fill(pm, t, ellipse(x + dx, by - up - 12.0, 1.7, 1.7), &g);
    }
    rect(pm, t, x + 1.2, by - h, 1.0, h, &solid(rgba("#c8e8a0", 0.3)));
}
pub(super) fn rockform(
    pm: &mut Pixmap,
    t: Transform,
    r: &mut Rng,
    x: f32,
    by: f32,
    w: f32,
    h: f32,
    c1: &str,
    c2: &str,
) {
    let n = 7;
    let mut pts = vec![(x, by)];
    for i in 0..=n {
        let f = i as f32 / n as f32;
        let hh = h * (0.45 + 0.55 * (1.0 - (f * 2.0 - 1.0).powi(2))) * (0.75 + 0.25 * r.f());
        pts.push((x + f * w, by - hh));
    }
    pts.push((x + w, by));
    fill(pm, t, poly(&pts), &lin(by - h, by, &[(0.0, c1), (1.0, c2)]));
    // lit edge on the sun side + strata
    let mut b = PathBuilder::new();
    for (i, p) in pts.iter().enumerate().skip(1).take(n + 1) {
        if i == 1 {
            b.move_to(p.0, p.1)
        } else {
            b.line_to(p.0, p.1)
        }
    }
    stroke(pm, t, b.finish(), &solid(rgba("#ffe6c0", 0.55)), 0.8);
    for k in 1..4 {
        let y = by - h * k as f32 * 0.22;
        stroke(
            pm,
            t,
            line(x + w * 0.1, y, x + w * 0.9, y + (r.f() - 0.5) * 3.0),
            &solid(rgba("#3a1a0e", 0.25)),
            0.7,
        );
    }
}
