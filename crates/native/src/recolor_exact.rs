use serde_json::Value;
use std::collections::HashMap;

pub(super) type ExactMap = HashMap<[u8; 3], [u8; 3]>;

pub(super) fn exact_map(palette: &Value) -> Option<ExactMap> {
    let entries = palette.get("exact")?.as_object()?;
    let map: ExactMap = entries
        .iter()
        .filter_map(|(from, to)| Some((hex(from)?, hex(to.as_str()?)?)))
        .filter(|(from, to)| from != to)
        .collect();
    (!map.is_empty()).then_some(map)
}

pub(super) fn apply_exact(map: &ExactMap, rgba: &mut [u8]) {
    for pixel in rgba.chunks_exact_mut(4) {
        if pixel[3] == 0 { continue; }
        if let Some(target) = map.get(&[pixel[0], pixel[1], pixel[2]]) {
            pixel[..3].copy_from_slice(target);
        }
    }
}

fn hex(text: &str) -> Option<[u8; 3]> {
    let digits = text.trim().trim_start_matches('#');
    if digits.len() != 6 || !digits.is_ascii() { return None; }
    let byte = |range: std::ops::Range<usize>| u8::from_str_radix(&digits[range], 16).ok();
    Some([byte(0..2)?, byte(2..4)?, byte(4..6)?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_only_the_exact_colours_the_palette_names() {
        let palette = serde_json::json!({"exact": {"#102030": "#ff0000", "#102031": "#102031", "bad": "#000000"}});
        let map = exact_map(&palette).unwrap();
        assert_eq!(map.len(), 1);
        let mut pixels = vec![16, 32, 48, 255, 16, 32, 49, 255, 16, 32, 48, 0];
        apply_exact(&map, &mut pixels);
        assert_eq!(pixels, [255, 0, 0, 255, 16, 32, 49, 255, 16, 32, 48, 0], "near colours and transparent pixels stay untouched");
    }

    #[test]
    fn shading_ramps_do_not_bleed_into_each_other() {
        let palette = serde_json::json!({"exact": {"#323232": "#aa0000", "#646464": "#00aa00"}});
        let map = exact_map(&palette).unwrap();
        let mut pixels = vec![50, 50, 50, 255, 100, 100, 100, 255];
        apply_exact(&map, &mut pixels);
        assert_eq!(pixels, [170, 0, 0, 255, 0, 170, 0, 255]);
    }

    #[test]
    fn absent_or_empty_exact_block_is_not_an_exact_palette() {
        assert!(exact_map(&serde_json::json!({"top": "#ffffff"})).is_none());
        assert!(exact_map(&serde_json::json!({"exact": {}})).is_none());
    }
}
