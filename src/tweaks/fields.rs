use super::model::{Labels, Tweaks};
use toml::Value;

pub const MENU_KEYS: [&str; 16] = [
    "arcade", "story", "survival", "cpu", "versus", "training", "watch", "team_seq",
    "team_simul", "team_tag", "mugen", "market", "controls", "settings", "replays", "quit",
];

pub const SECTIONS: [(&str, &[&str]); 5] = [
    ("theme", &["background", "glow_bottom", "glow_left", "glow_right"]),
    ("hud", &["p1_tint", "p2_tint", "bar_offset_y"]),
    ("accessibility", &["hud_opacity", "colorblind", "reduce_motion"]),
    ("rules", &["rounds_to_win", "round_seconds", "life_percent", "game_speed"]),
    ("menu", &MENU_KEYS),
];

pub fn color(text: &str) -> Option<[u8; 3]> {
    let h = text.trim().trim_start_matches('#');
    let full: String = match h.len() {
        3 => h.chars().flat_map(|c| [c, c]).collect(),
        6 => h.to_string(),
        _ => return None,
    };
    let v = u32::from_str_radix(&full, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

fn number(v: &Value) -> Option<f64> {
    v.as_float().or_else(|| v.as_integer().map(|i| i as f64))
}

fn ranged(v: &Value, lo: f64, hi: f64) -> Option<f64> {
    number(v).filter(|n| (lo..=hi).contains(n))
}

pub fn apply(section: &str, key: &str, v: &Value, t: &mut Tweaks, labels: &mut Labels) -> Result<(), String> {
    let bad_color = || format!("[{section}] `{key}` must be a color like \"#ff8800\" or \"#f80\"");
    let bad_range = |lo: &str, hi: &str| format!("[{section}] `{key}` must be a number from {lo} to {hi}");
    let c = || v.as_str().and_then(color);
    match (section, key) {
        ("theme", "background") => t.background = c().ok_or_else(bad_color)?,
        ("theme", "glow_bottom") => t.glow_bottom = c().ok_or_else(bad_color)?,
        ("theme", "glow_left") => t.glow_left = c().ok_or_else(bad_color)?,
        ("theme", "glow_right") => t.glow_right = c().ok_or_else(bad_color)?,
        ("hud", "p1_tint") => t.p1_tint = c().ok_or_else(bad_color)?,
        ("hud", "p2_tint") => t.p2_tint = c().ok_or_else(bad_color)?,
        ("hud", "bar_offset_y") => t.bar_offset_y = ranged(v, -4.0, 40.0).ok_or_else(|| bad_range("-4", "40"))? as f32,
        ("accessibility", "hud_opacity") => t.hud_opacity = ranged(v, 0.3, 1.0).ok_or_else(|| bad_range("0.3", "1.0"))? as f32,
        ("accessibility", "colorblind") => t.colorblind = v.as_bool().ok_or_else(|| format!("[{section}] `{key}` must be true or false"))?,
        ("accessibility", "reduce_motion") => t.reduce_motion = v.as_bool().ok_or_else(|| format!("[{section}] `{key}` must be true or false"))?,
        ("rules", "rounds_to_win") => t.rules.rounds_to_win = Some(ranged(v, 1.0, 9.0).ok_or_else(|| bad_range("1", "9"))? as i32),
        ("rules", "round_seconds") => t.rules.round_seconds = Some(ranged(v, 0.0, 999.0).ok_or_else(|| bad_range("0 (no limit)", "999"))? as u32),
        ("rules", "life_percent") => t.rules.life_percent = Some(ranged(v, 10.0, 500.0).ok_or_else(|| bad_range("10", "500"))? as u32),
        ("rules", "game_speed") => t.rules.game_speed = Some(ranged(v, 0.5, 2.0).ok_or_else(|| bad_range("0.5", "2.0"))? as f32),
        ("menu", _) => match v.as_str().filter(|s| !s.trim().is_empty()) {
            Some(s) => labels.push((format!("menu.{key}"), s.to_string())),
            None => return Err(format!("[menu] `{key}` must be a non-empty text")),
        },
        _ => {}
    }
    Ok(())
}
