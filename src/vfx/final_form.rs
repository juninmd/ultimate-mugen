use super::{quality, Vfx};
use crate::gfx::*;
use ume_engine::fighter::Fighter;

const BOLTS: usize = 12;
const SEGMENTS: usize = 5;

fn hash(i: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9E37_79B1) ^ 0x85EB_CA6B;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

pub(super) fn is_final(f: &Fighter) -> bool {
    f.form > 0 && (f.form as usize >= f.form_count() || f.form >= 3)
}

fn surge(f: &Fighter) -> f32 {
    if f.form_time >= 120 {
        return 1.0;
    }
    1.0 + 1.6 * (1.0 - f.form_time as f32 / 120.0)
}

fn quad(m: &mut Mesh, a: Vec2, b: Vec2, w: f32, ca: Color, cb: Color) {
    let d = (b - a).normalize_or_zero();
    let n = vec2(-d.y, d.x) * w;
    let base = m.vertices.len() as u16;
    for (p, c) in [(a - n, ca), (a + n, ca), (b + n, cb), (b - n, cb)] {
        m.vertices.push(Vertex::new(p.x, p.y, 0.0, 0.0, 0.0, c));
    }
    m.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

impl Vfx {
    pub(super) fn final_burst(&mut self, f: &Fighter, col: [f32; 3]) {
        let (x, y, _) = Self::chest_of(f);
        let feet = f.pos.y + f.lane;
        for (i, r) in [120.0f32, 210.0, 320.0, 440.0].iter().enumerate() {
            self.ring(x, y, *r, 0.7 + i as f32 * 0.18, 3.4 - i as f32 * 0.6, col);
        }
        self.ring(f.pos.x, feet - 1.0, 380.0, 1.2, 4.0, [1.0, 1.0, 1.0]);
        self.glow(x, y, 120.0, 0.9, col);
        self.glow(x, y, 46.0, 0.5, [1.0, 1.0, 1.0]);
        self.light(x, y, 360.0, 1.4, col);
        self.spark_burst(x, y, 1.0, quality::count(48), 13.0, col, std::f32::consts::PI);
        self.sheets.final_form(f.pos.x, x, y, feet);
        if quality::level() >= 1 {
            self.cracks(f.pos.x, feet, 1.0, 96.0);
            self.cracks(f.pos.x, feet, -1.0, 96.0);
        }
        self.trauma = (self.trauma + 0.2).min(0.9);
        self.radial = 0.9;
    }

    pub(super) fn draw_final_aura(&self, tp: &dyn Fn(f32, f32) -> Vec2, s: f32, f: &Fighter, col: [f32; 3], hgt: f32) {
        let k = surge(f);
        let body = tp(f.pos.x, f.pos.y - hgt * 0.38);
        let rx = 30.0 * s * (1.0 + 0.2 * (k - 1.0));
        let ry = hgt * 0.72 * s;
        let pulse = 0.85 + 0.15 * (self.clock * 9.0).sin();
        let mut halo = Mesh { vertices: Vec::with_capacity(34), indices: Vec::with_capacity(96), texture: None };
        halo.vertices.push(Vertex::new(body.x, body.y, 0.0, 0.0, 0.0, Color::new(1.0, 1.0, 1.0, 0.65 * pulse)));
        for i in 0..32 {
            let a = i as f32 / 32.0 * std::f32::consts::TAU;
            let wob = 1.0 + 0.12 * (a * 5.0 + self.clock * 7.0).sin();
            halo.vertices.push(Vertex::new(
                body.x + a.cos() * rx * 2.6 * wob,
                body.y + a.sin() * ry * wob,
                0.0, 0.0, 0.0,
                Color::new(col[0], col[1], col[2], 0.0),
            ));
            halo.indices.extend([0u16, 1 + i as u16, 1 + ((i + 1) % 32) as u16]);
        }
        draw_mesh(&halo);
        let mut m = Mesh { vertices: Vec::with_capacity(BOLTS * SEGMENTS * 8), indices: Vec::with_capacity(BOLTS * SEGMENTS * 12), texture: None };
        let tick = (self.clock * 16.0) as u32;
        let bolts = ((BOLTS as f32 * (0.55 + 0.45 * quality::level() as f32 / 2.0)) as usize).max(3);
        for b in 0..bolts {
            let seed = tick.wrapping_mul(131).wrapping_add(b as u32 * 977);
            if hash(seed ^ 0xA5) < 0.04 {
                continue;
            }
            let a0 = hash(seed) * std::f32::consts::TAU;
            let mut p = vec2(body.x + a0.cos() * rx * 0.9, body.y + a0.sin() * ry * 0.6);
            let reach = (0.55 + hash(seed ^ 0x33) * 0.7) * k.min(1.6);
            let out = vec2(a0.cos() * 1.4, a0.sin());
            for sg in 0..SEGMENTS {
                let t = sg as f32 / SEGMENTS as f32;
                let jx = (hash(seed ^ (sg as u32 * 17 + 3)) - 0.5) * 13.0 * s;
                let jy = (hash(seed ^ (sg as u32 * 29 + 7)) - 0.5) * 13.0 * s;
                let step = vec2(out.x * rx * 0.2 * reach + jx, out.y * ry * 0.09 * reach + jy);
                let q = p + step;
                let fade = 1.0 - t;
                quad(&mut m, p, q, 6.5 * s * fade + 1.0, Color::new(col[0], col[1], col[2], 0.35 * fade), Color::new(col[0], col[1], col[2], 0.25 * fade));
                quad(&mut m, p, q, 2.2 * s * fade + 0.6, Color::new(1.0, 1.0, 1.0, 0.95 * fade), Color::new(1.0, 1.0, 1.0, 0.7 * fade));
                p = q;
            }
        }
        draw_mesh(&m);
    }
}
