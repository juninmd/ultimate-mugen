use std::path::{Path, PathBuf};
use ume_formats::act::{decode_act, encode_act};

pub struct Preset {
    pub name: &'static str,
    pub hue_shift: f64,
    pub saturation: f64,
    pub brightness: f64,
}

pub const PRESETS: [Preset; 7] = [
    Preset { name: "red", hue_shift: -120.0, saturation: 1.0, brightness: 1.0 },
    Preset { name: "blue", hue_shift: 120.0, saturation: 1.0, brightness: 1.0 },
    Preset { name: "green", hue_shift: 60.0, saturation: 1.0, brightness: 1.0 },
    Preset { name: "violet", hue_shift: 180.0, saturation: 1.0, brightness: 1.0 },
    Preset { name: "dark", hue_shift: 0.0, saturation: 0.9, brightness: 0.65 },
    Preset { name: "pale", hue_shift: 0.0, saturation: 0.6, brightness: 1.25 },
    Preset { name: "gray", hue_shift: 0.0, saturation: 0.0, brightness: 1.0 },
];

pub fn load_act(path: &Path) -> Result<Vec<[u8; 3]>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(decode_act(&bytes).chunks_exact(4).map(|c| [c[0], c[1], c[2]]).collect())
}

pub fn save_act(colors: &[[u8; 3]], path: &Path) -> Result<(), String> {
    let mut rgba = vec![0u8; 1024];
    for (index, color) in colors.iter().take(256).enumerate() {
        rgba[index * 4..index * 4 + 3].copy_from_slice(color);
        rgba[index * 4 + 3] = 255;
    }
    std::fs::write(path, encode_act(&rgba)).map_err(|e| format!("{}: {e}", path.display()))
}

fn rgb_to_hls(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let high = r.max(g).max(b);
    let low = r.min(g).min(b);
    let light = (high + low) / 2.0;
    if high == low {
        return (0.0, light, 0.0);
    }
    let span = high - low;
    let sat = if light <= 0.5 { span / (high + low) } else { span / (2.0 - high - low) };
    let (rc, gc, bc) = ((high - r) / span, (high - g) / span, (high - b) / span);
    let hue = if r == high { bc - gc } else if g == high { 2.0 + rc - bc } else { 4.0 + gc - rc };
    ((hue / 6.0).rem_euclid(1.0), light, sat)
}

fn hue_channel(m1: f64, m2: f64, hue: f64) -> f64 {
    let hue = hue.rem_euclid(1.0);
    if hue < 1.0 / 6.0 {
        m1 + (m2 - m1) * hue * 6.0
    } else if hue < 0.5 {
        m2
    } else if hue < 2.0 / 3.0 {
        m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
    } else {
        m1
    }
}

fn hls_to_rgb(h: f64, l: f64, s: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }
    let m2 = if l <= 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let m1 = 2.0 * l - m2;
    (hue_channel(m1, m2, h + 1.0 / 3.0), hue_channel(m1, m2, h), hue_channel(m1, m2, h - 1.0 / 3.0))
}

pub fn variant_colors(colors: &[[u8; 3]], hue_shift: f64, saturation: f64, brightness: f64, keep: &[usize]) -> Vec<[u8; 3]> {
    colors
        .iter()
        .enumerate()
        .map(|(index, color)| {
            if index == 0 || keep.contains(&index) {
                return *color;
            }
            let (h, l, s) = rgb_to_hls(color[0] as f64 / 255.0, color[1] as f64 / 255.0, color[2] as f64 / 255.0);
            let (r, g, b) = hls_to_rgb((h + hue_shift / 360.0).rem_euclid(1.0), (l * brightness).min(1.0), (s * saturation).min(1.0));
            [(r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8]
        })
        .collect()
}

pub fn write_variants(act: &Path, out_dir: &Path, names: &[String], keep: &[usize]) -> Result<Vec<PathBuf>, String> {
    let base = load_act(act)?;
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut written = Vec::new();
    for preset in PRESETS.iter().filter(|p| names.is_empty() || names.iter().any(|n| n == p.name)) {
        let target = out_dir.join(format!("{}.act", preset.name));
        save_act(&variant_colors(&base, preset.hue_shift, preset.saturation, preset.brightness, keep), &target)?;
        written.push(target);
    }
    Ok(written)
}
