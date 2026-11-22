use super::*;

pub(super) fn lin(y0: f32, y1: f32, stops: &[(f32, &str)]) -> Paint<'static> {
    let st = stops
        .iter()
        .map(|(o, c)| GradientStop::new(*o, rgba(c, 1.0)))
        .collect();
    let mut p = Paint::default();
    p.shader = LinearGradient::new(
        Point::from_xy(0.0, y0),
        Point::from_xy(0.0, y1),
        st,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    p.anti_alias = true;
    p
}
pub(super) fn lin_a(y0: f32, y1: f32, stops: &[(f32, &str, f32)]) -> Paint<'static> {
    let st = stops
        .iter()
        .map(|(o, c, a)| GradientStop::new(*o, rgba(c, *a)))
        .collect();
    let mut p = Paint::default();
    p.shader = LinearGradient::new(
        Point::from_xy(0.0, y0),
        Point::from_xy(0.0, y1),
        st,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    p.anti_alias = true;
    p
}
pub(super) fn lin_h(x0: f32, x1: f32, stops: &[(f32, &str, f32)]) -> Paint<'static> {
    let st = stops
        .iter()
        .map(|(o, c, a)| GradientStop::new(*o, rgba(c, *a)))
        .collect();
    let mut p = Paint::default();
    p.shader = LinearGradient::new(
        Point::from_xy(x0, 0.0),
        Point::from_xy(x1, 0.0),
        st,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    p.anti_alias = true;
    p
}
pub(super) fn rect(pm: &mut Pixmap, t: Transform, x: f32, y: f32, w: f32, h: f32, p: &Paint) {
    if let Some(r) = tiny_skia::Rect::from_xywh(x, y, w.max(0.01), h.max(0.01)) {
        pm.fill_rect(r, p, t, None);
    }
}
pub(super) fn fill(pm: &mut Pixmap, t: Transform, path: Option<tiny_skia::Path>, p: &Paint) {
    if let Some(pa) = path {
        pm.fill_path(&pa, p, FillRule::Winding, t, None);
    }
}
pub(super) fn stroke(pm: &mut Pixmap, t: Transform, path: Option<tiny_skia::Path>, p: &Paint, w: f32) {
    if let Some(pa) = path {
        pm.stroke_path(
            &pa,
            p,
            &Stroke {
                width: w,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            },
            t,
            None,
        );
    }
}
pub(super) fn ellipse(x: f32, y: f32, rx: f32, ry: f32) -> Option<tiny_skia::Path> {
    tiny_skia::Rect::from_xywh(x - rx, y - ry, rx * 2.0, ry * 2.0).and_then(PathBuilder::from_oval)
}
pub(super) fn line(x0: f32, y0: f32, x1: f32, y1: f32) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    b.move_to(x0, y0);
    b.line_to(x1, y1);
    b.finish()
}
pub(super) fn poly(pts: &[(f32, f32)]) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    for (i, p) in pts.iter().enumerate() {
        if i == 0 {
            b.move_to(p.0, p.1)
        } else {
            b.line_to(p.0, p.1)
        }
    }
    b.close();
    b.finish()
}
pub(super) fn radial(cx: f32, cy: f32, r: f32, stops: &[(f32, &str, f32)]) -> Paint<'static> {
    let st = stops
        .iter()
        .map(|(o, c, a)| GradientStop::new(*o, rgba(c, *a)))
        .collect();
    let mut p = Paint::default();
    p.shader = RadialGradient::new(
        Point::from_xy(cx, cy),
        Point::from_xy(cx, cy),
        r,
        st,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    p.anti_alias = true;
    p
}
pub(super) fn glowc(pm: &mut Pixmap, t: Transform, x: f32, y: f32, r: f32, col: &str, a: f32) {
    fill(
        pm,
        t,
        circle(x, y, r),
        &radial(
            x,
            y,
            r,
            &[(0.0, col, a), (0.4, col, a * 0.45), (1.0, col, 0.0)],
        ),
    );
}
/// Noise-driven ridge line filled with a vertical gradient (hills, mesas, distant mountains).
#[allow(clippy::too_many_arguments)]
pub(super) fn ridge(
    pm: &mut Pixmap,
    t: Transform,
    w: f32,
    base: f32,
    amp: f32,
    top: &str,
    bot: &str,
    seed: f32,
    freq: f32,
) {
    let mut b = PathBuilder::new();
    b.move_to(0.0, base + 260.0);
    let mut x = 0.0;
    while x <= w + 3.0 {
        let n = n1(x * freq, seed);
        b.line_to(x, base - amp * (n * n * (3.0 - 2.0 * n)).powf(1.3));
        x += 2.5;
    }
    b.line_to(w + 3.0, base + 260.0);
    b.close();
    fill(
        pm,
        t,
        b.finish(),
        &lin(base - amp, base + 50.0, &[(0.0, top), (1.0, bot)]),
    );
}
pub(super) fn parts(seed: u32, n: usize, f: impl Fn(&mut Rng) -> Particle) -> Vec<Particle> {
    let mut r = Rng(seed);
    (0..n).map(|_| f(&mut r)).collect()
}
pub(super) fn info(name: &str) -> StageInfo {
    StageInfo {
        start_x: 0.0,
        start_y: 0.0,
        zoom_out: 0.62,
        zoom_in: 1.05,
        name: name.into(),
        bound_left: -680.0,
        bound_right: 680.0,
        bound_high: -140.0,
        bound_low: 0.0,
        p1x: -70.0,
        p2x: 70.0,
        zoffset: 205.0,
        vertical_follow: 0.35,
        tension: 72.0,
        ..StageInfo::default()
    }
}
