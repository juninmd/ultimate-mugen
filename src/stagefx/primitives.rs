use super::*;

pub fn hx(h: u32, a: f32) -> Color {
    Color::new(
        ((h >> 16) & 255) as f32 / 255.0,
        ((h >> 8) & 255) as f32 / 255.0,
        (h & 255) as f32 / 255.0,
        a,
    )
}
pub(super) fn ca(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}
pub(super) fn lerpc(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}
pub(super) fn qd(p: [Vec2; 4], c: [Color; 4]) {
    let vertices = (0..4)
        .map(|i| Vertex::new(p[i].x, p[i].y, 0.0, 0.0, 0.0, c[i]))
        .collect();
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}
pub(super) fn tr(p: [Vec2; 3], c: [Color; 3]) {
    let vertices = (0..3)
        .map(|i| Vertex::new(p[i].x, p[i].y, 0.0, 0.0, 0.0, c[i]))
        .collect();
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2],
        texture: None,
    });
}
/// Soft radial blob (ellipse with `cin` in the centre fading to `cout` on the rim) — glows, fog, light pools.
pub(super) fn fan(c: Vec2, rx: f32, ry: f32, n: usize, cin: Color, cout: Color) {
    let mut vertices = Vec::with_capacity(n + 1);
    vertices.push(Vertex::new(c.x, c.y, 0.0, 0.0, 0.0, cin));
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        vertices.push(Vertex::new(
            c.x + a.cos() * rx,
            c.y + a.sin() * ry,
            0.0,
            0.0,
            0.0,
            cout,
        ));
    }
    let mut indices = Vec::with_capacity(n * 3);
    for i in 0..n {
        indices.extend([0u16, 1 + i as u16, 1 + ((i + 1) % n) as u16]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
/// Triangle strip between two polylines `a` and `b` with a colour per vertex.
pub(super) fn strip(a: &[Vec2], b: &[Vec2], cola: &[Color], colb: &[Color]) {
    let n = a.len().min(b.len());
    if n < 2 {
        return;
    }
    let mut vertices = Vec::with_capacity(n * 2);
    for i in 0..n {
        vertices.push(Vertex::new(a[i].x, a[i].y, 0.0, 0.0, 0.0, cola[i]));
        vertices.push(Vertex::new(b[i].x, b[i].y, 0.0, 0.0, 0.0, colb[i]));
    }
    let mut indices = Vec::with_capacity((n - 1) * 6);
    for i in 0..n - 1 {
        let k = (i * 2) as u16;
        indices.extend([k, k + 1, k + 2, k + 1, k + 3, k + 2]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
/// Thick segment with a colour at each end.
pub(super) fn seg(a: Vec2, b: Vec2, w: f32, ca_: Color, cb: Color) {
    let d = (b - a).normalize_or_zero();
    let n = vec2(-d.y, d.x) * w * 0.5;
    qd([a + n, b + n, b - n, a - n], [ca_, cb, cb, ca_]);
}
pub(super) fn ring_e(c: Vec2, rx: f32, ry: f32, th: f32, col: Color) {
    let n = ((rx.sqrt() * 3.0) as usize).clamp(14, 48);
    let mut vertices = Vec::with_capacity(n * 2);
    for k in 0..n {
        let a = k as f32 / n as f32 * std::f32::consts::TAU;
        let (cs, sn) = (a.cos(), a.sin());
        vertices.push(Vertex::new(
            c.x + cs * (rx + th),
            c.y + sn * (ry + th * 0.3),
            0.0,
            0.0,
            0.0,
            col,
        ));
        vertices.push(Vertex::new(
            c.x + cs * (rx - th).max(0.0),
            c.y + sn * (ry - th * 0.3).max(0.0),
            0.0,
            0.0,
            0.0,
            col,
        ));
    }
    let mut indices = Vec::with_capacity(n * 6);
    for k in 0..n {
        let (a, b, c2, d) = (
            (2 * k) as u16,
            (2 * k + 1) as u16,
            (2 * ((k + 1) % n)) as u16,
            (2 * ((k + 1) % n) + 1) as u16,
        );
        indices.extend([a, b, c2, b, d, c2]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
pub(super) fn rot(p: Vec2, a: f32) -> Vec2 {
    let (s, c) = a.sin_cos();
    vec2(p.x * c - p.y * s, p.x * s + p.y * c)
}
pub(super) fn additive(_: &Gfx, f: impl FnOnce()) {
    set_blend(GfxBlend::Add);
    f();
    set_blend(GfxBlend::Alpha);
}
