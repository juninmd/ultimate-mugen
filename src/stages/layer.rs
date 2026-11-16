use super::*;

pub(super) type Painter<'a> = &'a dyn Fn(&mut Pixmap, Transform, f32, f32, &mut Rng);
/// `cap` limits the pixels-per-unit of soft / distant layers (memory at 4K); `front` layers draw over the fighters.
#[allow(clippy::too_many_arguments)]
pub(super) fn lay(
    cap: f32,
    front: bool,
    w: f32,
    h: f32,
    y: f32,
    delta: f32,
    seed: u32,
    dy: f32,
    paint: Painter,
) -> Layer {
    // pixels per game unit follows the art-quality setting (4K → 2x), capped so no layer exceeds 8192 px (GPU texture limit)
    let r = ume_engine::native::rig::art_res()
        .min(cap)
        .min(8192.0 / w.max(h).max(1.0));
    let mut pm = Pixmap::new(((w * r) as u32).max(1), ((h * r) as u32).max(1)).unwrap();
    let mut rng = Rng(seed);
    paint(&mut pm, Transform::from_scale(r, r), w, h, &mut rng);
    // crop the transparent rows at the top: skipping them saves fill-rate every frame (far silhouette layers are half empty)
    let (pw, ph) = (pm.width() as usize, pm.height() as usize);
    let first = pm
        .data()
        .chunks(pw * 4)
        .position(|row| row.chunks(4).any(|px| px[3] > 3))
        .unwrap_or(ph.saturating_sub(1));
    let first = first.saturating_sub(2);
    if first > 4 && !front {
        let cropped = pm.data()[first * pw * 4..].to_vec();
        if let Some(sz) = tiny_skia::IntSize::from_wh(pw as u32, (ph - first) as u32) {
            if let Some(c) = Pixmap::from_vec(cropped, sz) {
                return Layer {
                    image: pixmap_to_image(&c),
                    w,
                    h,
                    y,
                    delta,
                    dy,
                    front,
                    dynamic: false,
                    trim: first as f32 / r,
                };
            }
        }
    }
    Layer {
        image: pixmap_to_image(&pm),
        w,
        h,
        y,
        delta,
        dy,
        front,
        dynamic: false,
        trim: 0.0,
    }
}
pub(super) fn layer(w: f32, h: f32, y: f32, delta: f32, seed: u32, dy: f32, paint: Painter) -> Layer {
    lay(99.0, false, w, h, y, delta, seed, dy, paint)
}
/// Transparent placeholder: carries only the parallax transform of animated content drawn by `stagefx`.
pub(super) fn dynamic(w: f32, h: f32, y: f32, delta: f32, dy: f32, front: bool) -> Layer {
    Layer {
        image: ImageData::new(1, 1, vec![0, 0, 0, 0]),
        w,
        h,
        y,
        delta,
        dy,
        front,
        dynamic: true,
        trim: 0.0,
    }
}
