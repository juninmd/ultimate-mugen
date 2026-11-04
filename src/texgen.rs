//! Procedural texture toolkit for the stage art: a tileable fBm noise tile sampled bilinearly (cheap even at 4K) and per-pixel
//! passes on a tiny-skia `Pixmap` — grain (wood fibre, stone, sand), thresholded colour blobs (clouds, moss, stains, rust,
//! water glints) and pattern helpers. Everything is deterministic and runs once when a stage is built (never per frame).
use std::sync::OnceLock;
use tiny_skia::Pixmap;

const NT: usize = 256;

fn tile() -> &'static [f32] {
    static T: OnceLock<Vec<f32>> = OnceLock::new();
    T.get_or_init(|| {
        let mut v = vec![0.0f32; NT * NT];
        let (mut amp, mut tot) = (1.0f32, 0.0f32);
        for (o, cells) in [8usize, 16, 32, 64].iter().enumerate() {
            let cells = *cells;
            let hash = |x: usize, y: usize| -> f32 {
                let mut h = (x as u32).wrapping_mul(374761393)
                    ^ (y as u32).wrapping_mul(668265263)
                    ^ ((o as u32) + 1).wrapping_mul(2246822519);
                h = (h ^ (h >> 13)).wrapping_mul(1274126177);
                ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
            };
            let cs = NT / cells;
            for y in 0..NT {
                for x in 0..NT {
                    let (gx, gy) = (x / cs, y / cs);
                    let (fx, fy) = ((x % cs) as f32 / cs as f32, (y % cs) as f32 / cs as f32);
                    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
                    let g = |a: usize, b: usize| hash(a % cells, b % cells);
                    let top = g(gx, gy) + (g(gx + 1, gy) - g(gx, gy)) * sx;
                    let bot = g(gx, gy + 1) + (g(gx + 1, gy + 1) - g(gx, gy + 1)) * sx;
                    v[y * NT + x] += (top + (bot - top) * sy) * amp;
                }
            }
            tot += amp;
            amp *= 0.5;
        }
        for p in v.iter_mut() {
            *p = (((*p / tot) - 0.5) * 2.0 + 0.5).clamp(0.0, 1.0);
        }
        v
    })
}

/// Bilinear sample of the wrapped noise tile, `u`/`v` in texels. Returns 0..1 (mean ≈ 0.5).
pub fn nz(u: f32, v: f32) -> f32 {
    let t = tile();
    let (u, v) = (u.rem_euclid(NT as f32), v.rem_euclid(NT as f32));
    let (x0, y0) = ((u as usize) & (NT - 1), (v as usize) & (NT - 1));
    let (x1, y1) = ((x0 + 1) & (NT - 1), (y0 + 1) & (NT - 1));
    let (fx, fy) = (u.fract(), v.fract());
    let a = t[y0 * NT + x0] + (t[y0 * NT + x1] - t[y0 * NT + x0]) * fx;
    let b = t[y1 * NT + x0] + (t[y1 * NT + x1] - t[y1 * NT + x0]) * fx;
    a + (b - a) * fy
}
/// Noise at game-unit coordinates with per-axis frequency (texels per unit) and a seed offset.
pub fn nzu(x: f32, y: f32, fx: f32, fy: f32, off: f32) -> f32 {
    nz(x * fx + off * 37.0, y * fy + off * 91.0)
}
/// 1-D noise (ridge lines, shorelines): smooth value along `x`.
pub fn n1(x: f32, seed: f32) -> f32 {
    nz(x, seed * 53.0 + 11.0)
}
pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a).max(1e-5)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
/// Stable per-index hash in 0..1.
pub fn hash(i: u32, k: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9E3779B1) ^ k.wrapping_mul(0x85EBCA6B).wrapping_add(0x27D4EB2F);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A2D39);
    h ^= h >> 15;
    (h & 0xffffff) as f32 / 16777216.0
}

fn rgb(hex: &str) -> [f32; 3] {
    let h = hex.trim_start_matches('#');
    let v = u32::from_str_radix(h, 16).unwrap_or(0x808080);
    if h.len() == 3 {
        [
            ((v >> 8) & 15) as f32 * 17.0,
            ((v >> 4) & 15) as f32 * 17.0,
            (v & 15) as f32 * 17.0,
        ]
    } else {
        [
            ((v >> 16) & 255) as f32,
            ((v >> 8) & 255) as f32,
            (v & 255) as f32,
        ]
    }
}

/// Area in game units: (x, y, w, h). Returns clamped pixel bounds.
fn bounds(pm: &Pixmap, r: f32, a: (f32, f32, f32, f32)) -> (i32, i32, i32, i32) {
    let (pw, ph) = (pm.width() as i32, pm.height() as i32);
    (
        ((a.0 * r) as i32).clamp(0, pw),
        ((a.1 * r) as i32).clamp(0, ph),
        (((a.0 + a.2) * r).ceil() as i32).clamp(0, pw),
        (((a.1 + a.3) * r).ceil() as i32).clamp(0, ph),
    )
}

/// Multiply brightness by `1 + (noise-0.5) * amount` — wood fibre (`fx` small, `fy` large), stone grain, sand.
#[allow(clippy::too_many_arguments)]
pub fn grain(
    pm: &mut Pixmap,
    r: f32,
    area: (f32, f32, f32, f32),
    fx: f32,
    fy: f32,
    amount: f32,
    off: f32,
) {
    let (x0, y0, x1, y1) = bounds(pm, r, area);
    let pw = pm.width() as i32;
    let d = pm.data_mut();
    for py in y0..y1 {
        for px in x0..x1 {
            let i = ((py * pw + px) * 4) as usize;
            let a = d[i + 3] as f32;
            if a == 0.0 {
                continue;
            }
            let k = 1.0 + (nzu(px as f32 / r, py as f32 / r, fx, fy, off) - 0.5) * amount;
            for c in 0..3 {
                d[i + c] = (d[i + c] as f32 * k).clamp(0.0, a) as u8;
            }
        }
    }
}

/// Composite colour `col` where noise exceeds `th` (soft edge `soft`). `light` adds embossed shading (cloud tops, bark, bumpy stone).
/// `masked` keeps the existing alpha (stain only the material, not the empty background).
#[allow(clippy::too_many_arguments)]
pub fn blobs(
    pm: &mut Pixmap,
    r: f32,
    area: (f32, f32, f32, f32),
    fx: f32,
    fy: f32,
    off: f32,
    th: f32,
    soft: f32,
    col: &str,
    strength: f32,
    light: f32,
    masked: bool,
) {
    let c = rgb(col);
    let (x0, y0, x1, y1) = bounds(pm, r, area);
    let pw = pm.width() as i32;
    let d = pm.data_mut();
    for py in y0..y1 {
        for px in x0..x1 {
            let (ux, uy) = (px as f32 / r, py as f32 / r);
            let n = nzu(ux, uy, fx, fy, off);
            let mut a = smooth(th, th + soft, n) * strength;
            if a <= 0.002 {
                continue;
            }
            let i = ((py * pw + px) * 4) as usize;
            let da = d[i + 3] as f32 / 255.0;
            if masked {
                a *= da;
            }
            let sh = if light != 0.0 {
                1.0 + (n - nzu(ux + 1.6, uy + 1.6, fx, fy, off)) * light
            } else {
                1.0
            };
            for k in 0..3 {
                d[i + k] = ((c[k] * sh).clamp(0.0, 255.0) * a + d[i + k] as f32 * (1.0 - a))
                    .clamp(0.0, 255.0) as u8;
            }
            d[i + 3] = (255.0 * a + d[i + 3] as f32 * (1.0 - a)).clamp(0.0, 255.0) as u8;
        }
    }
}

/// Vertical fade of existing pixels (atmospheric haze): blends `col` over the area with alpha going `a0` (top) → `a1` (bottom).
pub fn haze(pm: &mut Pixmap, r: f32, area: (f32, f32, f32, f32), col: &str, a0: f32, a1: f32) {
    let c = rgb(col);
    let (x0, y0, x1, y1) = bounds(pm, r, area);
    let pw = pm.width() as i32;
    let d = pm.data_mut();
    for py in y0..y1 {
        let t = (py - y0) as f32 / (y1 - y0).max(1) as f32;
        let a = a0 + (a1 - a0) * t;
        for px in x0..x1 {
            let i = ((py * pw + px) * 4) as usize;
            let da = d[i + 3] as f32 / 255.0;
            if da <= 0.0 {
                continue;
            }
            let k = a * da;
            for ch in 0..3 {
                d[i + ch] = (c[ch] * k + d[i + ch] as f32 * (1.0 - a)).clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Soft ambient-occlusion / rim light: darkens (or lightens, `amt` < 0) the pixels within `edge` units of a vertical gradient.
pub fn shade_v(pm: &mut Pixmap, r: f32, area: (f32, f32, f32, f32), k0: f32, k1: f32) {
    let (x0, y0, x1, y1) = bounds(pm, r, area);
    let pw = pm.width() as i32;
    let d = pm.data_mut();
    for py in y0..y1 {
        let t = (py - y0) as f32 / (y1 - y0).max(1) as f32;
        let k = k0 + (k1 - k0) * t;
        for px in x0..x1 {
            let i = ((py * pw + px) * 4) as usize;
            let a = d[i + 3] as f32;
            for ch in 0..3 {
                d[i + ch] = (d[i + ch] as f32 * k).clamp(0.0, a) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noise_is_in_range_and_wraps() {
        for i in 0..200 {
            let v = nz(i as f32 * 3.7, i as f32 * 1.3);
            assert!((0.0..=1.0).contains(&v));
        }
        assert!((nz(5.0, 9.0) - nz(5.0 + NT as f32, 9.0 + NT as f32)).abs() < 1e-4);
    }
    #[test]
    fn grain_keeps_alpha() {
        let mut pm = Pixmap::new(32, 32).unwrap();
        pm.fill(tiny_skia::Color::from_rgba8(100, 100, 100, 255));
        grain(&mut pm, 1.0, (0.0, 0.0, 32.0, 32.0), 1.0, 1.0, 0.8, 0.0);
        assert!(pm.pixels().iter().all(|p| p.alpha() == 255));
    }
}
