use toml::{Table, Value};

pub const DEFAULT_TEXT: &str = "[theme]\nbackground = \"#0b0716\"\nglow_bottom = \"#ff4d6e\"\nglow_left = \"#21e6ff\"\nglow_right = \"#ffcc33\"\n\n[hud]\np1_tint = \"#4db8ff\"\np2_tint = \"#ff6185\"\n";

const SECTIONS: [(&str, &[&str]); 2] = [
    ("theme", &["background", "glow_bottom", "glow_left", "glow_right"]),
    ("hud", &["p1_tint", "p2_tint"]),
];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tweaks {
    pub background: [u8; 3],
    pub glow_bottom: [u8; 3],
    pub glow_left: [u8; 3],
    pub glow_right: [u8; 3],
    pub p1_tint: [u8; 3],
    pub p2_tint: [u8; 3],
}

impl Default for Tweaks {
    fn default() -> Self {
        Tweaks {
            background: [0x0b, 0x07, 0x16],
            glow_bottom: [0xff, 0x4d, 0x6e],
            glow_left: [0x21, 0xe6, 0xff],
            glow_right: [0xff, 0xcc, 0x33],
            p1_tint: [0x4d, 0xb8, 0xff],
            p2_tint: [0xff, 0x61, 0x85],
        }
    }
}

impl Tweaks {
    pub fn tint(&self, side: usize) -> [f32; 3] {
        let c = if side == 0 { self.p1_tint } else { self.p2_tint };
        [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]
    }
}

fn color(text: &str) -> Option<[u8; 3]> {
    let h = text.trim().trim_start_matches('#');
    let full: String = match h.len() {
        3 => h.chars().flat_map(|c| [c, c]).collect(),
        6 => h.to_string(),
        _ => return None,
    };
    let v = u32::from_str_radix(&full, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

fn closest<'a>(name: &str, options: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    options.into_iter().find(|o| o.starts_with(&name[..1.min(name.len())]) && (o.contains(name) || name.contains(o) || o.len().abs_diff(name.len()) <= 2))
}

fn read_section(name: &str, keys: &[&str], table: &Table, warnings: &mut Vec<String>, mut set: impl FnMut(&str, [u8; 3])) {
    for (key, value) in table {
        if !keys.contains(&key.as_str()) {
            let hint = closest(key, keys.iter().copied()).map(|k| format!(" (did you mean `{k}`?)")).unwrap_or_default();
            warnings.push(format!("[{name}] unknown key `{key}`{hint}"));
            continue;
        }
        match value.as_str().and_then(color) {
            Some(c) => set(key, c),
            None => warnings.push(format!("[{name}] `{key}` must be a color like \"#ff8800\" or \"#f80\"")),
        }
    }
}

pub fn parse(text: &str) -> (Tweaks, Vec<String>) {
    let mut t = Tweaks::default();
    let mut warnings = Vec::new();
    let root: Table = match text.parse() {
        Ok(r) => r,
        Err(e) => {
            let line = e.span().map(|s| text[..s.start.min(text.len())].matches('\n').count() + 1);
            let at = line.map(|l| format!("line {l}: ")).unwrap_or_default();
            warnings.push(format!("tweaks.toml {at}{}", e.message()));
            return (t, warnings);
        }
    };
    for (name, value) in &root {
        let Some((_, keys)) = SECTIONS.iter().find(|(s, _)| s == name) else {
            let hint = closest(name, SECTIONS.iter().map(|(s, _)| *s)).map(|s| format!(" (did you mean [{s}]?)")).unwrap_or_default();
            warnings.push(format!("unknown section [{name}]{hint}"));
            continue;
        };
        let Value::Table(table) = value else {
            warnings.push(format!("`{name}` must be a [{name}] section"));
            continue;
        };
        read_section(name, keys, table, &mut warnings, |key, c| match key {
            "background" => t.background = c,
            "glow_bottom" => t.glow_bottom = c,
            "glow_left" => t.glow_left = c,
            "glow_right" => t.glow_right = c,
            "p1_tint" => t.p1_tint = c,
            _ => t.p2_tint = c,
        });
    }
    (t, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_text_equals_defaults() {
        let (t, w) = parse(DEFAULT_TEXT);
        assert_eq!(t, Tweaks::default());
        assert!(w.is_empty());
    }

    #[test]
    fn overrides_and_short_hex() {
        let (t, w) = parse("[hud]\np1_tint = \"#f00\"\n");
        assert_eq!(t.p1_tint, [255, 0, 0]);
        assert_eq!(t.p2_tint, Tweaks::default().p2_tint);
        assert!(w.is_empty());
    }

    #[test]
    fn typos_and_bad_values_warn_but_keep_defaults() {
        let (t, w) = parse("[hud]\np1_tin = \"#f00\"\np2_tint = 5\n[hudd]\na = 1\n");
        assert_eq!(t, Tweaks::default());
        assert_eq!(w.len(), 3);
        assert!(w[0].contains("did you mean `p1_tint`"));
        assert!(w.iter().any(|m| m.contains("did you mean [hud]")));
    }

    #[test]
    fn syntax_error_reports_line() {
        let (t, w) = parse("[hud]\np1_tint = \n");
        assert_eq!(t, Tweaks::default());
        assert!(w[0].contains("line 2"));
    }
}
