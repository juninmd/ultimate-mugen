use serde_json::Value;

pub(super) struct Recolor { pairs: Vec<([[f32; 3]; 2], bool)>, keep: Vec<[f32; 3]>, restrict_hair: bool }

impl Recolor {
    pub(super) fn new(colors: Option<&Value>, palette: &Value) -> Option<Self> {
        let base = colors?.as_object()?;
        let overrides = palette.get("colors").unwrap_or(palette).as_object()?;
        let mut keys: Vec<&String> = base.keys().collect();
        keys.sort();
        keys.truncate(256);
        let mut pairs = vec![];
        let mut overridden: Vec<[f32; 3]> = vec![];
        let mut keep = vec![];
        for key in keys {
            let Some(source) = base.get(key).and_then(Value::as_str).and_then(rgb) else { continue };
            match overrides.get(key).and_then(Value::as_str).and_then(rgb) {
                Some(target) if target != source => {
                    overridden.push(source);
                    pairs.push(([source, target], key == "hair"));
                }
                _ => keep.push(source),
            }
        }
        keep.retain(|color| !overridden.contains(color));
        keep.truncate(256);
        let restrict_hair = palette.get("_awakening").and_then(Value::as_bool).unwrap_or(false);
        (!pairs.is_empty()).then_some(Recolor { pairs, keep, restrict_hair })
    }

    pub(super) fn apply(&self, rgba: &mut [u8]) {
        self.apply_in_region(rgba, 0, None);
    }

    pub(super) fn apply_in_region(&self, rgba: &mut [u8], width: u32, region: Option<[u32; 4]>) {
        let mut cache: std::collections::HashMap<u32, [u8; 3], std::hash::BuildHasherDefault<ColourHasher>> = Default::default();
        let mut last: Option<(u32, [u8; 3])> = None;
        for (index, pixel) in rgba.chunks_exact_mut(4).enumerate() {
            if pixel[3] == 0 { continue; }
            let key = [pixel[0], pixel[1], pixel[2]];
            let allow_hair = !self.restrict_hair || region.is_some_and(|[x1, y1, x2, y2]| width > 0 && (index as u32 % width) >= x1 && (index as u32 % width) < x2 && (index as u32 / width) >= y1 && (index as u32 / width) < y2);
            let packed = u32::from_le_bytes([key[0], key[1], key[2], allow_hair as u8]);
            let mapped = match last {
                Some((previous, mapped)) if previous == packed => mapped,
                _ => *cache.entry(packed).or_insert_with(|| self.swap(key, allow_hair)),
            };
            last = Some((packed, mapped));
            pixel[..3].copy_from_slice(&mapped);
        }
    }

    fn swap(&self, [r, g, b]: [u8; 3], allow_hair: bool) -> [u8; 3] {
        let pixel = [r as f32, g as f32, b as f32];
        let fit = |source: &[f32; 3]| -> Option<(f32, f32)> {
            let energy = source.iter().map(|c| c * c).sum::<f32>().max(1.0);
            let shade = pixel.iter().zip(source).map(|(p, s)| p * s).sum::<f32>() / energy;
            if !(0.10..=2.4).contains(&shade) { return None; }
            let residual = pixel.iter().zip(source).map(|(p, s)| (p - s * shade).powi(2)).sum::<f32>().sqrt();
            (residual <= 60.0 * shade.clamp(0.45, 1.0)).then_some((residual, shade))
        };
        let mut swap: Option<(f32, [f32; 3], f32, bool)> = None;
        for ([source, target], is_hair) in &self.pairs {
            if *is_hair && !allow_hair { continue; }
            if let Some((residual, shade)) = fit(source) {
                if swap.is_none_or(|(score, _, _, _)| residual < score) { swap = Some((residual, *target, shade, *is_hair)); }
            }
        }
        let mut untouched = f32::INFINITY;
        for source in &self.keep {
            if let Some((residual, _)) = fit(source) {
                if residual < untouched { untouched = residual; }
            }
        }
        match swap {
            Some((residual, target, shade, is_hair)) if residual <= untouched => {
                let shade = if self.restrict_hair && is_hair { (0.35 + shade * 0.8).min(1.0) } else { shade };
                [
                (target[0] * shade).round().clamp(0.0, 255.0) as u8,
                (target[1] * shade).round().clamp(0.0, 255.0) as u8,
                (target[2] * shade).round().clamp(0.0, 255.0) as u8,
                ]
            },
            _ => [r, g, b],
        }
    }
}

#[derive(Default)]
struct ColourHasher(u64);

impl std::hash::Hasher for ColourHasher {
    fn finish(&self) -> u64 { self.0 }
    fn write(&mut self, bytes: &[u8]) { for byte in bytes { self.0 = (self.0 ^ *byte as u64).wrapping_mul(0x100000001b3); } }
    fn write_u32(&mut self, value: u32) { self.0 = (value as u64).wrapping_mul(0x9E3779B97F4A7C15); }
}

fn rgb(hex: &str) -> Option<[f32; 3]> {
    let digits = hex.trim().trim_start_matches('#').as_bytes();
    let pair = |bytes: &[u8]| -> Option<f32> { u8::from_str_radix(std::str::from_utf8(bytes).ok()?, 16).ok().map(f32::from) };
    let shade = |byte: u8| -> Option<f32> { char::from(byte).to_digit(16).map(|digit| (digit * 17) as f32) };
    match digits.len() {
        3 => Some([shade(digits[0])?, shade(digits[1])?, shade(digits[2])?]),
        len if len >= 6 => Some([pair(&digits[0..2])?, pair(&digits[2..4])?, pair(&digits[4..6])?]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::pixel::try_load;
    use crate::native::types::{NativeChar, NativeFrame};

    #[test]
    fn awakening_hair_is_confined_to_the_authored_region() {
        let colors = serde_json::json!({"hair":"#3a2418", "skin":"#e2a97e"});
        let palette = serde_json::json!({"hair":"#f6c83e", "_awakening":true});
        let recolor = Recolor::new(Some(&colors), &palette).unwrap();
        let mut pixels = vec![58, 36, 24, 255, 58, 36, 24, 255];
        recolor.apply_in_region(&mut pixels, 2, Some([0, 0, 1, 1]));
        assert_eq!(&pixels[..4], &[246, 200, 62, 255]);
        assert_eq!(&pixels[4..], &[58, 36, 24, 255]);
        let mut missing = vec![58, 36, 24, 255];
        recolor.apply_in_region(&mut missing, 1, None);
        assert_eq!(missing, [58, 36, 24, 255]);
    }

    #[test]
    fn palette_swap_rebuilds_a_base_colour_along_its_own_shading() {
        let colors = serde_json::json!({"colors": {"top": "#2f8f9d", "pants": "#2a2f3a"}});
        let palette = serde_json::json!({"top": "#b8433a"});
        let recolor = Recolor::new(Some(&colors["colors"]), &palette).unwrap();
        let mut pixels = vec![47, 143, 157, 255, 23, 71, 78, 255, 200, 30, 30, 255];
        recolor.apply(&mut pixels);
        assert_eq!(&pixels[0..3], &[184, 67, 58], "the declared base colour becomes the palette colour");
        assert!((pixels[4] as i32 - 91).abs() <= 1 && (pixels[5] as i32 - 33).abs() <= 1, "a half-shaded pixel keeps its shade: {pixels:?}");
        assert_eq!(&pixels[8..11], &[200, 30, 30], "colours the palette does not name stay untouched");
        assert!(pixels.chunks_exact(4).all(|pixel| pixel[3] == 255), "alpha is never rewritten");
    }

    #[test]
    fn palette_swap_leaves_a_colour_the_palette_does_not_name() {
        let colors = serde_json::json!({"colors": {"top": "#2f8f9d", "skin": "#e2a97e"}});
        let palette = serde_json::json!({"top": "#b8433a"});
        let recolor = Recolor::new(Some(&colors["colors"]), &palette).unwrap();
        let skin = [226, 169, 126, 255];
        let mut pixels = skin.to_vec();
        recolor.apply(&mut pixels);
        assert_eq!(pixels, skin, "skin is not in the palette, so it must stay byte-identical");
    }

    #[test]
    fn palette_swap_keeps_the_silhouette_and_replaces_the_gi() {
        let fighter: NativeChar = serde_json::from_str(include_str!("../../../../assets/chars/kaito.json")).unwrap();
        let frame: NativeFrame = serde_json::from_value(serde_json::json!({"pose": "idle0", "ticks": 1})).unwrap();
        let base = try_load(&fighter, &frame, None).unwrap().image.unwrap();
        let swapped = try_load(&fighter, &frame, Some(&fighter.palettes[0])).unwrap().image.unwrap();
        assert_eq!((base.w, base.h), (swapped.w, swapped.h));
        assert!(base.rgba.chunks_exact(4).zip(swapped.rgba.chunks_exact(4)).all(|(before, after)| before[3] == after[3]), "the silhouette is preserved");
        let count = |rgba: &[u8], test: &dyn Fn(&[u8]) -> bool| rgba.chunks_exact(4).filter(|pixel| pixel[3] > 0 && test(pixel)).count();
        let teal = |pixel: &[u8]| pixel[1] as i32 > pixel[0] as i32 + 40 && pixel[2] as i32 > pixel[0] as i32 + 40;
        let red = |pixel: &[u8]| pixel[0] as i32 > pixel[1] as i32 + 40 && pixel[0] as i32 > pixel[2] as i32 + 40;
        assert!(count(&base.rgba, &teal) > 1_000, "kaito's gi starts teal");
        assert!(count(&swapped.rgba, &teal) * 20 < count(&base.rgba, &teal), "palette 2 removes the teal gi");
        assert!(count(&swapped.rgba, &red) > count(&base.rgba, &red) + 1_000, "palette 2 paints the gi red");
    }

    #[test]
    fn palette_swap_without_overrides_decodes_the_same_pixels() {
        let fighter: NativeChar = serde_json::from_str(include_str!("../../../../assets/chars/kaito.json")).unwrap();
        let frame: NativeFrame = serde_json::from_value(serde_json::json!({"pose": "idle0", "ticks": 1})).unwrap();
        let plain = try_load(&fighter, &frame, None).unwrap().image.unwrap();
        let empty = serde_json::json!({});
        let same = try_load(&fighter, &frame, Some(&empty)).unwrap().image.unwrap();
        assert_eq!(plain.rgba, same.rgba);
    }

    #[test]
    fn colour_parser_accepts_short_and_long_hex() {
        assert_eq!(rgb("#f00"), Some([255.0, 0.0, 0.0]));
        assert_eq!(rgb("#00ff00"), Some([0.0, 255.0, 0.0]));
        assert_eq!(rgb("#0000ffff"), Some([0.0, 0.0, 255.0]));
        assert_eq!(rgb("#12"), None);
        assert_eq!(rgb("nope"), None);
    }
}
