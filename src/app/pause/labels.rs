use crate::controls::Glyphs;
use crate::gamepad::PadKind;
use ume_engine::chardef::MoveInfo;

pub fn cost_label(mv: &MoveInfo) -> Option<String> {
    if mv.cost <= 0.0 {
        return None;
    }
    Some(if mv.drain {
        format!("{:.0}/s PWR", mv.cost)
    } else if mv.cost >= 1000.0 && mv.cost % 1000.0 == 0.0 {
        match (mv.cost / 1000.0) as i32 {
            1 => "1 BAR".to_string(),
            b => format!("{b} BARS"),
        }
    } else {
        format!("{:.0} PWR", mv.cost)
    })
}

pub fn slot_pad_labels(glyphs: Glyphs) -> [&'static str; 4] {
    match glyphs {
        Glyphs::Pad(PadKind::Xbox) => ["LB", "LT", "RB", "RT"],
        _ => ["L1", "L2", "R1", "R2"],
    }
}

pub fn truncate_to(text: &str, max_w: f32, width_of: &dyn Fn(&str) -> f32) -> String {
    if width_of(text) <= max_w {
        return text.to_string();
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let cut: String = chars.iter().collect::<String>().trim_end().to_string() + "..";
        if width_of(&cut) <= max_w {
            return cut;
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(cost: f32, drain: bool) -> MoveInfo {
        MoveInfo { name: "m".into(), input: "x".into(), command: "m".into(), cost, drain, forms: u32::MAX }
    }

    #[test]
    fn cost_labels() {
        assert_eq!(cost_label(&mv(0.0, false)), None);
        assert_eq!(cost_label(&mv(1000.0, false)).as_deref(), Some("1 BAR"));
        assert_eq!(cost_label(&mv(3000.0, false)).as_deref(), Some("3 BARS"));
        assert_eq!(cost_label(&mv(250.0, false)).as_deref(), Some("250 PWR"));
        assert_eq!(cost_label(&mv(40.0, true)).as_deref(), Some("40/s PWR"));
    }

    #[test]
    fn truncation_respects_the_width() {
        let w = |s: &str| s.chars().count() as f32 * 10.0;
        assert_eq!(truncate_to("short", 100.0, &w), "short");
        let t = truncate_to("Jade Lantern Ultimate", 100.0, &w);
        assert!(t.ends_with("..") && w(&t) <= 100.0);
        assert_eq!(truncate_to("abc", 5.0, &w), "");
    }

    #[test]
    fn pad_labels_follow_the_controller_family() {
        assert_eq!(slot_pad_labels(Glyphs::Pad(PadKind::Xbox))[1], "LT");
        assert_eq!(slot_pad_labels(Glyphs::Keyboard)[3], "R2");
    }
}
