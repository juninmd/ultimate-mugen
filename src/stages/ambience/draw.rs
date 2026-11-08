use super::{Beam, Glow, Mist, MoteKind, Motes, Profile};
use crate::gfx::*;
use crate::texgen::hash;

/// Illustration placement on screen (view units, before the view scale) plus the camera state ambient layers react to.
#[derive(Clone, Copy)]
pub struct ImageRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub view_w: f32,
    pub view_h: f32,
    pub cam_x: f32,
}

struct Frame {
    rect: ImageRect,
    ox: f32,
    oy: f32,
    s: f32,
    t: f32,
    density: f32,
}

fn rgb(c: u32, a: f32) -> Color {
    Color::new(
        ((c >> 16) & 255) as f32 / 255.0,
        ((c >> 8) & 255) as f32 / 255.0,
        (c & 255) as f32 / 255.0,
        a,
    )
}

fn wave(t: f32, period: f32, phase: f32) -> f32 {
    (t * std::f32::consts::TAU / period.max(0.1) + phase).sin() * 0.5 + 0.5
}

fn blob(cx: f32, cy: f32, rx: f32, ry: f32, inner: Color, outer: Color) {
    const SEGMENTS: usize = 12;
    let mut vertices = Vec::with_capacity(SEGMENTS + 1);
    vertices.push(Vertex::new(cx, cy, 0.0, 0.0, 0.0, inner));
    for i in 0..SEGMENTS {
        let a = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        vertices.push(Vertex::new(
            cx + a.cos() * rx,
            cy + a.sin() * ry,
            0.0,
            0.0,
            0.0,
            outer,
        ));
    }
    let mut indices = Vec::with_capacity(SEGMENTS * 3);
    for i in 0..SEGMENTS {
        indices.extend([0u16, 1 + i as u16, 1 + ((i + 1) % SEGMENTS) as u16]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

fn quad(p: [(f32, f32); 4], c: [Color; 4]) {
    let vertices = (0..4)
        .map(|i| Vertex::new(p[i].0, p[i].1, 0.0, 0.0, 0.0, c[i]))
        .collect();
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}

fn draw_mist(f: &Frame, m: &Mist) {
    let r = &f.rect;
    let span = r.view_w * 1.6;
    let rx = r.view_w * 0.34;
    let ry = (r.view_h * m.height * 0.5).max(4.0);
    let travel = f.t * m.speed - r.cam_x * m.depth * 0.5;
    for i in 0..m.blobs as usize {
        let seed = hash(i as u32, 7);
        let x = ((i as f32 + seed * 0.6) / m.blobs as f32 * span + travel).rem_euclid(span)
            - (span - r.view_w) * 0.5
            - rx * 0.4;
        let y = r.view_h * m.v
            + (f.t * 0.35 + seed * 6.28).sin() * ry * 0.18
            + (hash(i as u32, 3) - 0.5) * ry * 0.5;
        let a = m.alpha * (0.65 + 0.35 * wave(f.t, 11.0 + seed * 9.0, seed * 6.28));
        blob(
            f.ox + x * f.s,
            f.oy + y * f.s,
            rx * (0.7 + seed * 0.6) * f.s,
            ry * f.s,
            rgb(m.color, a),
            rgb(m.color, 0.0),
        );
    }
}

fn draw_mote(f: &Frame, m: &Motes, i: usize) {
    let r = &f.rect;
    let (h1, h2, h3) = (hash(i as u32, 11), hash(i as u32, 23), hash(i as u32, 37));
    let band_h = (m.band[1] - m.band[0]) * r.view_h;
    let span = r.view_w * 1.2;
    let cycle = (h2 - m.rise * f.t / band_h.max(1.0)).rem_euclid(1.0);
    let y = r.view_h * m.band[0] + cycle * band_h;
    let sway = (f.t * (0.6 + h3) + h1 * 6.28).sin() * m.sway;
    let x = (h1 * span + m.drift * f.t - r.cam_x * m.depth * 0.6).rem_euclid(span)
        - (span - r.view_w) * 0.5
        + sway;
    let fade = (cycle * (1.0 - cycle) * 4.0).clamp(0.0, 1.0);
    let twinkle = if m.kind == MoteKind::Firefly || m.kind == MoteKind::Spark {
        0.35 + 0.65 * wave(f.t, 1.4 + h3 * 2.0, h2 * 6.28)
    } else {
        1.0
    };
    let a = m.alpha * fade * twinkle;
    let (px, py, sz) = (
        f.ox + x * f.s,
        f.oy + y * f.s,
        m.size * (0.6 + h3 * 0.8) * f.s,
    );
    match m.kind {
        MoteKind::Dust | MoteKind::Snow => draw_circle(px, py, sz * 0.7, rgb(m.color, a)),
        MoteKind::Ember | MoteKind::Spark | MoteKind::Firefly => {
            blob(
                px,
                py,
                sz * 2.6,
                sz * 2.6,
                rgb(m.color, a * 0.5),
                rgb(m.color, 0.0),
            );
            draw_circle(px, py, sz * 0.6, rgb(0xffffff, a));
        }
        MoteKind::Petal | MoteKind::Leaf => {
            let ang = f.t * (1.0 + h3) + h1 * 6.28;
            let (dx, dy) = (ang.cos() * sz, ang.sin() * sz * 0.45);
            quad(
                [
                    (px - dx, py - dy),
                    (px, py - sz * 0.35),
                    (px + dx, py + dy),
                    (px, py + sz * 0.35),
                ],
                [rgb(m.color, a); 4],
            );
        }
        MoteKind::Bird => {
            let flap = (f.t * (5.0 + h3 * 3.0) + h1 * 6.28).sin() * sz * 0.5;
            draw_line(
                px - sz,
                py + flap,
                px,
                py,
                (sz * 0.18).max(1.0),
                rgb(m.color, a),
            );
            draw_line(
                px,
                py,
                px + sz,
                py + flap,
                (sz * 0.18).max(1.0),
                rgb(m.color, a),
            );
        }
    }
}

fn draw_glow(f: &Frame, g: &Glow) {
    let r = &f.rect;
    let pulse = 1.0
        - g.flicker
            * (0.5 * wave(f.t, g.period, g.u * 9.0) + 0.5 * wave(f.t * 3.7, g.period, g.v * 5.0));
    let rad = g.radius * r.w * (0.92 + 0.08 * pulse);
    blob(
        f.ox + (r.x + g.u * r.w) * f.s,
        f.oy + (r.y + g.v * r.h) * f.s,
        rad * f.s,
        rad * f.s * 0.8,
        rgb(g.color, g.alpha * pulse),
        rgb(g.color, 0.0),
    );
}

fn draw_beam(f: &Frame, b: &Beam) {
    let r = &f.rect;
    let breathe = 0.55 + 0.45 * wave(f.t, b.period, b.u * 12.0);
    let top = r.x + b.u * r.w - r.cam_x * 0.02;
    let (half, foot) = (b.width * r.w * 0.5, top + b.slant * r.w);
    let (y0, y1) = (r.y, r.y + r.h * 0.9);
    let p = |x: f32, y: f32| (f.ox + x * f.s, f.oy + y * f.s);
    let (strong, clear) = (rgb(b.color, b.alpha * breathe), rgb(b.color, 0.0));
    quad(
        [
            p(top - half * 0.4, y0),
            p(top + half * 0.4, y0),
            p(foot + half, y1),
            p(foot - half, y1),
        ],
        [strong, strong, clear, clear],
    );
}

/// Draws the back (`front == false`) or front half of the ambience. `density` scales particle counts (effects setting).
pub fn draw_profile(
    profile: &Profile,
    rect: ImageRect,
    scale: f32,
    origin: (f32, f32),
    game_time: i32,
    density: f32,
    front: bool,
) {
    let f = Frame {
        rect,
        ox: origin.0,
        oy: origin.1,
        s: scale,
        t: game_time as f32 / 60.0,
        density: density.clamp(0.0, 1.3),
    };
    if f.density <= 0.0 || !super::enabled() {
        return;
    }
    if !front {
        set_blend(GfxBlend::Add);
        profile.beams.iter().for_each(|b| draw_beam(&f, b));
        profile.glows.iter().for_each(|g| draw_glow(&f, g));
        set_blend(GfxBlend::Alpha);
    }
    profile
        .mist
        .iter()
        .filter(|m| m.front == front)
        .for_each(|m| draw_mist(&f, m));
    for m in profile.motes.iter().filter(|m| m.front == front) {
        let n = ((m.count as f32 * f.density).round() as usize).min(m.count as usize);
        (0..n).for_each(|i| draw_mote(&f, m, i));
    }
}
