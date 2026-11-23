use super::compile::render_frame;
use super::forms::form_palette;
use super::effects::FrameArt;
use super::recolor::Recolor;
use super::types::{NativeChar, NativeFrame};
use ume_engine::chardef::Sprite;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

const FIRST_VARIANT_GROUP: i32 = 2;
const MAX_WORKERS: usize = 8;

pub(super) struct VariantPalettes {
    colors: Option<Value>,
    recolors: Vec<(i32, Option<Arc<Recolor>>)>,
}

impl VariantPalettes {
    pub(super) fn empty(colors: Option<&Value>) -> Self {
        VariantPalettes { colors: colors.cloned(), recolors: Vec::new() }
    }

    pub(super) fn new(nc: &NativeChar, base: Option<&Value>, awakening: Option<&Value>, with_images: bool) -> Self {
        let mut variants = Self::empty(nc.art.style.get("colors"));
        if !with_images {
            return variants;
        }
        if let Some(palette) = awakening {
            variants.assign(FIRST_VARIANT_GROUP, palette);
        }
        let forms = nc.abilities.as_ref().map(|spec| spec.forms.as_slice()).unwrap_or_default();
        for (i, form) in forms.iter().enumerate() {
            if let Some(palette) = form_palette(form, base) {
                variants.assign(FIRST_VARIANT_GROUP + i as i32, &palette);
            }
        }
        variants
    }

    fn assign(&mut self, group: i32, palette: &Value) {
        self.recolors.retain(|(g, _)| *g != group);
        self.recolors.push((group, Recolor::new(self.colors.as_ref(), palette).map(Arc::new)));
    }
}

pub(super) struct FrameRaster {
    pub base: FrameArt,
    pub variants: Vec<(i32, Sprite)>,
}

pub(super) fn sprite_of(render: &FrameArt) -> Sprite {
    Sprite { image: render.image.clone(), width: render.width, height: render.height, axis_x: render.axis_x, axis_y: render.axis_y, scale: render.scale }
}

fn raster_frame(nc: &NativeChar, frame: &NativeFrame, with_images: bool, palette: Option<&Value>, variants: &VariantPalettes) -> FrameRaster {
    let base = render_frame(nc, frame, with_images, palette);
    let recolored = if frame.effect.is_some() { &[][..] } else { variants.recolors.as_slice() };
    let source = recolored.iter().any(|(_, recolor)| recolor.is_some()).then(|| super::variant_art::locate(nc, frame)).flatten();
    let variants = recolored.iter().map(|(group, recolor)| (*group, super::variant_art::sprite(&base, source.as_ref(), recolor.as_ref()))).collect();
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

    fn kaito() -> NativeChar {
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap()
    }

    fn pixels(sprite: &Sprite) -> Option<(u32, u32, Arc<Vec<u8>>)> {
        sprite.image.as_ref().map(|i| (i.w, i.h, i.rgba().clone()))
    }

    #[test]
    fn a_later_palette_replaces_the_same_group() {
        let fighter = kaito();
        let mut variants = VariantPalettes::empty(fighter.art.style.get("colors"));
        variants.assign(2, &Value::from("not a palette"));
        variants.assign(3, &fighter.palettes[0]);
        variants.assign(2, &fighter.palettes[0]);
        assert_eq!(variants.recolors.iter().map(|(group, _)| *group).collect::<Vec<_>>(), vec![3, 2]);
        assert!(variants.recolors[1].1.is_some(), "the later palette is the one kept");
    }

    #[test]
    fn lazy_variants_match_the_fully_fitted_sprite() {
        let fighter = kaito();
        let frame: NativeFrame = serde_json::from_value(serde_json::json!({"pose": "punch1", "ticks": 1, "hit": {"part": "fist"}})).unwrap();
        let mut variants = VariantPalettes::empty(fighter.art.style.get("colors"));
        variants.assign(2, &fighter.palettes[0]);
        let (_, lazy) = raster_frame(&fighter, &frame, true, None, &variants).variants.remove(0);
        assert!(lazy.image.as_ref().unwrap().is_pending(), "nothing is recolored until the sprite is drawn");
        let full = sprite_of(&render_frame(&fighter, &frame, true, Some(&fighter.palettes[0])));
        assert_eq!(pixels(&lazy), pixels(&full), "recolored pixels are unchanged");
        assert_eq!((lazy.width, lazy.height, lazy.axis_x, lazy.axis_y, lazy.scale), (full.width, full.height, full.axis_x, full.axis_y, full.scale));
    }

    #[test]
    fn a_variant_that_changes_no_pixel_reuses_the_base_pixels() {
        let fighter = kaito();
        let frame: NativeFrame = serde_json::from_value(serde_json::json!({"pose": "idle0", "ticks": 1})).unwrap();
        let mut variants = VariantPalettes::empty(fighter.art.style.get("colors"));
        variants.assign(2, &serde_json::json!({}));
        variants.assign(3, &fighter.palettes[0]);
        let raster = raster_frame(&fighter, &frame, true, None, &variants);
        let pixels = |group: i32| raster.variants.iter().find(|(g, _)| *g == group).and_then(|(_, sprite)| sprite.image.clone()).unwrap().rgba().clone();
        let base = raster.base.image.as_ref().unwrap().rgba();
        assert!(Arc::ptr_eq(&pixels(2), base));
        assert!(!Arc::ptr_eq(&pixels(3), base));
    }

    #[test]
    fn worker_count_is_capped_and_never_zero() {
        assert_eq!(worker_count(0), 1);
        assert!(worker_count(1000) <= MAX_WORKERS);
    }
}
