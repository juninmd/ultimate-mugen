use super::compile::render_frame;
use super::forms::form_palette;
use super::effects::FrameArt;
use super::types::{NativeChar, NativeFrame};
use ume_engine::chardef::Sprite;
use serde_json::Value;
use std::collections::HashMap;

const FIRST_VARIANT_GROUP: i32 = 2;
const MAX_WORKERS: usize = 8;

pub(super) struct VariantPalettes(Vec<(i32, Value)>);

impl VariantPalettes {
    pub(super) fn new(nc: &NativeChar, base: Option<&Value>, awakening: Option<&Value>, with_images: bool) -> Self {
        let mut variants = VariantPalettes(Vec::new());
        if !with_images {
            return variants;
        }
        if let Some(palette) = awakening {
            variants.assign(FIRST_VARIANT_GROUP, palette.clone());
        }
        let forms = nc.abilities.as_ref().map(|spec| spec.forms.as_slice()).unwrap_or_default();
        for (i, form) in forms.iter().enumerate() {
            if let Some(palette) = form_palette(form, base) {
                variants.assign(FIRST_VARIANT_GROUP + i as i32, palette);
            }
        }
        variants
    }

    fn assign(&mut self, group: i32, palette: Value) {
        self.0.retain(|(g, _)| *g != group);
        self.0.push((group, palette));
    }
}

pub(super) struct FrameRaster {
    pub base: FrameArt,
    pub variants: Vec<(i32, FrameArt)>,
}

pub(super) fn sprite_of(render: FrameArt) -> Sprite {
    Sprite { image: render.image, width: render.width, height: render.height, axis_x: render.axis_x, axis_y: render.axis_y, scale: render.scale }
}

fn raster_frame(nc: &NativeChar, frame: &NativeFrame, with_images: bool, palette: Option<&Value>, variants: &VariantPalettes) -> FrameRaster {
    let base = render_frame(nc, frame, with_images, palette);
    let recolored = if frame.effect.is_some() { &[][..] } else { variants.0.as_slice() };
    // Variants keep only image and placement (`sprite_of`), so box fitting is skipped.
    let variants = recolored.iter().map(|(group, palette)| (*group, super::pixel::load_art(nc, frame, Some(palette), false).unwrap_or_else(|| render_frame(nc, frame, true, Some(palette))))).collect();
    FrameRaster { base, variants }
}

fn worker_count(jobs: usize) -> usize {
    if cfg!(target_arch = "wasm32") {
        return 1;
    }
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).min(jobs.max(1)).min(MAX_WORKERS)
}

pub(super) fn raster_frames(
    nc: &NativeChar,
    jobs: Vec<(String, &NativeFrame)>,
    with_images: bool,
    palette: Option<&Value>,
    variants: &VariantPalettes,
) -> HashMap<String, FrameRaster> {
    let workers = worker_count(jobs.len());
    if workers <= 1 {
        return jobs.into_iter().map(|(key, frame)| (key, raster_frame(nc, frame, with_images, palette, variants))).collect();
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done = std::sync::Mutex::new(HashMap::with_capacity(jobs.len()));
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some((key, frame)) = jobs.get(i) else { break };
                let raster = raster_frame(nc, frame, with_images, palette, variants);
                done.lock().unwrap().insert(key.clone(), raster);
            });
        }
    });
    done.into_inner().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_later_palette_replaces_the_same_group() {
        let mut variants = VariantPalettes(Vec::new());
        variants.assign(2, Value::from("awakening"));
        variants.assign(3, Value::from("second form"));
        variants.assign(2, Value::from("first form"));
        assert_eq!(variants.0.iter().map(|(group, _)| *group).collect::<Vec<_>>(), vec![3, 2]);
        assert_eq!(variants.0[1].1, Value::from("first form"));
    }

    #[test]
    fn image_only_variants_match_the_fully_fitted_sprite() {
        let fighter: NativeChar = serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
        let frame: NativeFrame = serde_json::from_value(serde_json::json!({"pose": "punch1", "ticks": 1, "hit": {"part": "fist"}})).unwrap();
        let mut variants = VariantPalettes(Vec::new());
        variants.assign(2, fighter.palettes[0].clone());
        let (_, fast) = raster_frame(&fighter, &frame, true, None, &variants).variants.remove(0);
        let full = render_frame(&fighter, &frame, true, Some(&fighter.palettes[0]));
        let (fast, full) = (sprite_of(fast), sprite_of(full));
        let pixels = |s: &Sprite| s.image.as_ref().map(|i| (i.w, i.h, i.rgba.clone()));
        assert_eq!(pixels(&fast), pixels(&full), "recolored pixels are unchanged");
        assert_eq!((fast.width, fast.height, fast.axis_x, fast.axis_y, fast.scale), (full.width, full.height, full.axis_x, full.axis_y, full.scale));
    }

    #[test]
    fn worker_count_is_capped_and_never_zero() {
        assert_eq!(worker_count(0), 1);
        assert!(worker_count(1000) <= MAX_WORKERS);
    }
}
