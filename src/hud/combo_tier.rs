use crate::gfx::Color;
use crate::ui::Align;

pub struct ComboRating {
    pub title: &'static str,
    pub color: Color,
}

pub fn rating(hits: i32, a: f32) -> Option<ComboRating> {
    match hits {
        15.. => Some(ComboRating {
            title: "CLIMAX!!",
            color: Color::new(1.0, 0.25, 0.35, a),
        }),
        10..=14 => Some(ComboRating {
            title: "SUPERB!",
            color: Color::new(1.0, 0.55, 0.15, a),
        }),
        6..=9 => Some(ComboRating {
            title: "GREAT!",
            color: Color::new(1.0, 0.85, 0.2, a),
        }),
        3..=5 => Some(ComboRating {
            title: "GOOD",
            color: Color::new(0.4, 0.85, 1.0, a),
        }),
        _ => None,
    }
}

pub fn draw_badges<F>(outlined: &F, x: f32, cy0: f32, hits: i32, countered: bool, a: f32, al: Align)
where
    F: Fn(&str, f32, f32, f32, Color, Align),
{
    if countered {
        outlined(
            "COUNTER!",
            x,
            cy0 + 66.0,
            7.5,
            Color::new(1.0, 0.45, 0.25, a),
            al,
        );
    }
    if let Some(tier) = rating(hits, a) {
        let y = if countered { cy0 + 73.0 } else { cy0 + 68.0 };
        outlined(tier.title, x, y, 6.5, tier.color, al);
    }
}
