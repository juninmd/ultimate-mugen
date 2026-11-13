use super::fields::{apply, SECTIONS};
use super::model::{Labels, Tweaks};
use toml::{Table, Value};

pub const DEFAULT_TEXT: &str = "[theme]\nbackground = \"#0b0716\"\nglow_bottom = \"#ff4d6e\"\nglow_left = \"#21e6ff\"\nglow_right = \"#ffcc33\"\n\n[hud]\np1_tint = \"#4db8ff\"\np2_tint = \"#ff6185\"\nbar_offset_y = 0\n\n# [rules]\n# rounds_to_win = 2\n# round_seconds = 99\n# life_percent = 100\n# game_speed = 1.0\n\n# [menu]\n# arcade = \"Fight!\"\n";

fn closest<'a>(name: &str, options: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    options.into_iter().find(|o| o.starts_with(&name[..1.min(name.len())]) && (o.contains(name) || name.contains(o) || o.len().abs_diff(name.len()) <= 2))
}

pub fn parse(text: &str) -> (Tweaks, Labels, Vec<String>) {
    let mut t = Tweaks::default();
    let mut labels = Labels::new();
    let mut warnings = Vec::new();
    let root: Table = match text.parse() {
        Ok(r) => r,
        Err(e) => {
            let line = e.span().map(|s| text[..s.start.min(text.len())].matches('\n').count() + 1);
            let at = line.map(|l| format!("line {l}: ")).unwrap_or_default();
            warnings.push(format!("tweaks.toml {at}{}", e.message()));
            return (t, labels, warnings);
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
        for (key, v) in table {
            if !keys.contains(&key.as_str()) {
                let hint = closest(key, keys.iter().copied()).map(|k| format!(" (did you mean `{k}`?)")).unwrap_or_default();
                warnings.push(format!("[{name}] unknown key `{key}`{hint}"));
            } else if let Err(w) = apply(name, key, v, &mut t, &mut labels) {
                warnings.push(w);
            }
        }
    }
    (t, labels, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_text_equals_defaults() {
        let (t, l, w) = parse(DEFAULT_TEXT);
        assert_eq!(t, Tweaks::default());
        assert!(l.is_empty() && w.is_empty());
    }

    #[test]
    fn overrides_and_short_hex() {
        let (t, _, w) = parse("[hud]\np1_tint = \"#f00\"\nbar_offset_y = 6\n");
        assert_eq!(t.p1_tint, [255, 0, 0]);
        assert_eq!(t.bar_offset_y, 6.0);
        assert!(w.is_empty());
    }

    #[test]
    fn rules_and_labels() {
        let (t, l, w) = parse("[rules]\nround_seconds = 0\ngame_speed = 1.2\nrounds_to_win = 99\n[menu]\narcade = \"Fight!\"\n");
        assert_eq!(t.rules.round_seconds, Some(0));
        assert_eq!(t.rules.game_speed, Some(1.2));
        assert_eq!(t.rules.rounds_to_win, None);
        assert_eq!(l, vec![("menu.arcade".to_string(), "Fight!".to_string())]);
        assert_eq!(w.len(), 1);
        assert!(w[0].contains("1 to 9"));
    }

    #[test]
    fn accessibility_section() {
        let (t, _, w) = parse("[accessibility]\nhud_opacity = 0.5\ncolorblind = true\nreduce_motion = true\n");
        assert_eq!(t.hud_opacity, 0.5);
        let (s, _, _) = parse("[accessibility]\nhud_scale = 1.2\n");
        assert_eq!(s.hud_scale, 1.2);
        assert!(t.colorblind && t.reduce_motion);
        assert!(w.is_empty());
        let (t, _, w) = parse("[accessibility]\nhud_opacity = 0.1\ncolorblind = 1\n");
        assert_eq!(t.hud_opacity, 1.0);
        assert!(!t.colorblind);
        assert_eq!(w.len(), 2);
    }

    #[test]
    fn typos_and_bad_values_warn_but_keep_defaults() {
        let (t, _, w) = parse("[hud]\np1_tin = \"#f00\"\np2_tint = 5\n[hudd]\na = 1\n");
        assert_eq!(t, Tweaks::default());
        assert_eq!(w.len(), 3);
        assert!(w.iter().any(|m| m.contains("did you mean `p1_tint`")));
        assert!(w.iter().any(|m| m.contains("did you mean [hud]")));
    }

    #[test]
    fn syntax_error_reports_line() {
        let (_, _, w) = parse("[hud]\np1_tint = \n");
        assert!(w[0].contains("line 2"));
    }
}
