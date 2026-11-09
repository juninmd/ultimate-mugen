use toml::Table;

const DEFAULT: &str = include_str!("../../assets/postfx.toml");
const FILE: &str = "postfx.toml";

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub bloom: f32,
    pub threshold: f32,
    pub tonemap: f32,
    pub exposure: f32,
    pub split: f32,
    pub shadows: [f32; 3],
    pub highlights: [f32; 3],
    pub shockwave: f32,
    pub heat: f32,
    pub chroma: f32,
    pub vignette: f32,
    pub grain: f32,
    pub grade_level: u8,
    pub shockwave_level: u8,
    pub heat_level: u8,
    pub bloom_level: u8,
}

fn number(t: &Table, section: &str, key: &str, fallback: f32) -> f32 {
    t.get(section)
        .and_then(|s| s.get(key))
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
        .map_or(fallback, |v| v as f32)
}

fn triple(t: &Table, section: &str, key: &str, fallback: [f32; 3]) -> [f32; 3] {
    let Some(items) = t.get(section).and_then(|s| s.get(key)).and_then(|v| v.as_array()) else {
        return fallback;
    };
    let mut out = fallback;
    for (slot, item) in out.iter_mut().zip(items) {
        if let Some(v) = item.as_float().or_else(|| item.as_integer().map(|i| i as f64)) {
            *slot = v as f32;
        }
    }
    out
}

fn level(t: &Table, key: &str, fallback: u8) -> u8 {
    number(t, "quality", key, fallback as f32).clamp(0.0, 3.0) as u8
}

impl Look {
    pub fn parse(text: &str) -> Look {
        let t: Table = text.parse().unwrap_or_default();
        Look {
            bloom: number(&t, "bloom", "intensity", 1.0).clamp(0.0, 3.0),
            threshold: number(&t, "bloom", "threshold", 0.58).clamp(0.05, 0.95),
            tonemap: number(&t, "grade", "tonemap", 0.0).clamp(0.0, 1.0),
            exposure: number(&t, "grade", "exposure", 1.0).clamp(0.4, 2.0),
            split: number(&t, "grade", "split", 0.0).clamp(0.0, 1.0),
            shadows: triple(&t, "grade", "shadows", [1.0; 3]),
            highlights: triple(&t, "grade", "highlights", [1.0; 3]),
            shockwave: number(&t, "distortion", "shockwave", 0.0).clamp(0.0, 3.0),
            heat: number(&t, "distortion", "heat", 0.0).clamp(0.0, 3.0),
            chroma: number(&t, "lens", "chroma", 1.0).clamp(0.0, 3.0),
            vignette: number(&t, "lens", "vignette", 1.0).clamp(0.0, 3.0),
            grain: number(&t, "lens", "grain", 0.035).clamp(0.0, 0.2),
            grade_level: level(&t, "grade_from_level", 1),
            shockwave_level: level(&t, "shockwave_from_level", 1),
            heat_level: level(&t, "heat_from_level", 2),
            bloom_level: level(&t, "bloom_from_level", 2),
        }
    }

    pub fn load() -> Look {
        match crate::storage::read(FILE) {
            Some(text) => Look::parse(&text),
            None => Look::parse(DEFAULT),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_default_parses_with_grading_on() {
        let l = Look::parse(DEFAULT);
        assert!(l.tonemap > 0.0 && l.shockwave > 0.0 && l.heat > 0.0);
        assert_eq!(l.shadows.len(), 3);
    }

    #[test]
    fn garbage_text_falls_back_to_neutral_values() {
        let l = Look::parse("not = [valid");
        assert_eq!((l.tonemap, l.shockwave, l.exposure), (0.0, 0.0, 1.0));
    }

    #[test]
    fn values_are_clamped_and_integers_accepted() {
        let l = Look::parse("[grade]\ntonemap = 9\nexposure = 1\n[quality]\nheat_from_level = 1");
        assert_eq!((l.tonemap, l.exposure, l.heat_level), (1.0, 1.0, 1));
    }
}
