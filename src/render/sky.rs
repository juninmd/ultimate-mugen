use crate::gfx::*;
use super::Gfx;
use ume_formats::stage::MugenStage;
use std::sync::Arc;
use ume_engine::chardef::ImageData;
use ume_engine::matchs::Match;
use ume_formats::stage::{SkyKind, StageSky};

const SIZE: u32 = 512;

#[derive(Default)]
pub struct SkyCache {
    made: Option<(SkyKind, [u32; 3], Arc<ImageData>)>,
}

impl SkyCache {
    pub fn draw(&mut self, g: &mut Gfx, m: &Match, stage: &MugenStage) {
        let Some(sky) = stage.sky.as_ref() else { return };
        let tex = self.texture(sky);
        let t = m.game_time as f32 * sky.speed / 60.0;
        let pulse = 0.5 + 0.5 * (t * 1.3).sin();
        let alpha = sky.intensity;
        let s = g.view_scale;
        let (w, h) = (m.view_w, m.view_h);
        let tex2d = g.texture(&tex, false);
        let draw = |x: f32, y: f32, dw: f32, dh: f32, a: f32| {
            draw_texture_ex(
                &tex2d,
                g.ox + x * s,
                g.oy + y * s,
                Color::new(1.0, 1.0, 1.0, a.clamp(0.0, 1.0)),
                DrawTextureParams { dest_size: Some(vec2(dw * s + 0.5, dh * s + 0.5)), ..Default::default() },
            );
        };
        match sky.kind {
            SkyKind::Stars => {
                set_blend(GfxBlend::Add);
                draw(0.0, 0.0, w, h, alpha * (0.55 + 0.45 * pulse));
            }
            SkyKind::GodRays => {
                set_blend(GfxBlend::Add);
                draw(0.0, 0.0, w, h, alpha * 0.5 * (0.7 + 0.3 * pulse));
            }
            SkyKind::Aurora => {
                set_blend(GfxBlend::Add);
                let drift = (t * 24.0).rem_euclid(w);
                let mut x = -drift;
                while x < w {
                    draw(x, 0.0, w * 0.5, h * 0.6, alpha * (0.6 + 0.4 * pulse));
                    x += w * 0.5;
                }
            }
            SkyKind::Clouds => {
                set_blend(GfxBlend::Alpha);
                let drift = (t * 10.0).rem_euclid(w);
                let mut x = -drift;
                while x < w {
                    draw(x, h * 0.12, w * 0.5, h * 0.4, alpha);
                    x += w * 0.5;
                }
            }
        }
        set_blend(GfxBlend::Alpha);
    }

    fn texture(&mut self, sky: &StageSky) -> Arc<ImageData> {
        let key = (sky.kind, sky.color.map(|c| (c * 255.0) as u32));
        if let Some((k, c, t)) = &self.made {
            if *k == key.0 && *c == key.1 {
                return t.clone();
            }
        }
        let rgba = make_pixels(sky.kind, sky.color);
        let tex = ImageData::new(SIZE, SIZE, rgba);
        self.made = Some((key.0, key.1, tex.clone()));
        tex
    }
}

fn hash(x: u32, y: u32, seed: u32) -> f32 {
    let mut h = x.wrapping_mul(374_761_393).wrapping_add(y.wrapping_mul(668_265_263)) ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}

/// Pixels of the sky texture: RGBA, `SIZE` x `SIZE`, tinted by `color`.
pub fn make_pixels(kind: SkyKind, color: [f32; 3]) -> Vec<u8> {
    let n = SIZE as usize;
    let mut out = vec![0u8; n * n * 4];
    let tau = std::f32::consts::TAU;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (fx, fy) = (x as f32 / SIZE as f32, y as f32 / SIZE as f32);
            let a = match kind {
                SkyKind::Aurora => {
                    let centre = 0.25 + 0.12 * (fx * tau * 2.0).sin() + 0.05 * (fx * tau * 5.0).sin();
                    let band = (-((fy - centre) * (fy - centre)) / 0.006).exp();
                    band * (0.35 + 0.65 * hash(x / 4, y / 16, 7))
                }
                SkyKind::Stars => {
                    let r = hash(x, y, 3);
                    if r > 0.994 { 0.5 + 0.5 * (r - 0.994) / 0.006 } else { 0.0 }
                }
                SkyKind::GodRays => {
                    let (dx, dy) = (fx - 0.5, fy);
                    let ang = dx.atan2(dy.max(0.001));
                    let rays = (0.5 + 0.5 * (ang * 9.0).sin()).powi(4);
                    let dist = (dx * dx + dy * dy).sqrt();
                    rays * (1.0 - dist * 1.4).clamp(0.0, 1.0)
                }
                SkyKind::Clouds => {
                    let mut v = 0.0f32;
                    for b in 0..14u32 {
                        let cx = hash(b, 1, 11);
                        let cy = 0.35 + 0.4 * hash(b, 2, 12);
                        let rad = 0.05 + 0.08 * hash(b, 3, 13);
                        let dx = (fx - cx).abs().min(1.0 - (fx - cx).abs());
                        let d2 = (dx * dx + (fy - cy) * (fy - cy)) / (rad * rad);
                        v += (-d2).exp();
                    }
                    v.clamp(0.0, 1.0) * 0.9
                }
            };
            let i = (y as usize * n + x as usize) * 4;
            out[i] = (color[0] * 255.0) as u8;
            out[i + 1] = (color[1] * 255.0) as u8;
            out[i + 2] = (color[2] * 255.0) as u8;
            out[i + 3] = (a.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_draws_something_and_stays_in_range() {
        for kind in [SkyKind::Aurora, SkyKind::GodRays, SkyKind::Stars, SkyKind::Clouds] {
            let px = make_pixels(kind, [1.0, 0.5, 0.25]);
            assert_eq!(px.len(), (SIZE * SIZE * 4) as usize);
            let lit = px.chunks_exact(4).filter(|p| p[3] > 0).count();
            assert!(lit > 0, "{kind:?} is empty");
            assert_eq!(px[0], 255);
        }
    }

    #[test]
    fn aurora_and_clouds_tile_horizontally() {
        for kind in [SkyKind::Aurora, SkyKind::Clouds] {
            let px = make_pixels(kind, [1.0, 1.0, 1.0]);
            let n = SIZE as usize;
            let row = |y: usize| (0..n).map(|x| px[(y * n + x) * 4 + 3]).collect::<Vec<u8>>();
            for y in [10usize, 100, 200] {
                let r = row(y);
                assert!((r[0] as i32 - r[n - 1] as i32).abs() < 40, "{kind:?} seam at row {y}");
            }
        }
    }
}
