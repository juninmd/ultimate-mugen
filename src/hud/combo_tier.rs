use crate::gfx::Color;
use crate::ui::Align;

pub struct ComboRating {
    pub title: &'static str,
    pub color: Color,
}

pub fn shown_damage(life_before_combo: f32, life: f32, engine_damage: f32) -> f32 {
    (life_before_combo - life).max(engine_damage).max(0.0)
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

pub fn draw_badges<P, F>(
    p: &P,
    s: f32,
    outlined: &F,
    x: f32,
    cy0: f32,
    hits: i32,
    countered: bool,
    a: f32,
    al: Align,
) where
    P: Fn(f32, f32) -> crate::gfx::Vec2,
    F: Fn(&str, f32, f32, f32, Color, Align),
{
    if countered {
        super::badge_art::draw_combo_plate(p, s, x, cy0 + 119.0, al, a);
        outlined(
            "COUNTER!",
            x,
            cy0 + 119.0,
            7.5,
            Color::new(1.0, 0.45, 0.25, a),
            al,
        );
    }
    if let Some(tier) = rating(hits, a) {
        let y = if countered { cy0 + 129.0 } else { cy0 + 119.0 };
        super::badge_art::draw_combo_plate(p, s, x, y, al, a);
        outlined(tier.title, x, y, 6.5, tier.color, al);
    }
}

#[cfg(test)]
mod tests {
    use super::shown_damage;

    #[test]
    fn combo_damage_counts_life_lost_even_when_the_engine_total_was_reset() {
        assert_eq!(shown_damage(1000.0, 820.0, 0.0), 180.0);
    }

    #[test]
    fn combo_damage_keeps_the_engine_total_when_it_is_larger() {
        assert_eq!(shown_damage(1000.0, 1000.0, 45.0), 45.0);
    }

    #[test]
    fn combo_damage_never_goes_negative_when_life_regenerates() {
        assert_eq!(shown_damage(900.0, 950.0, 0.0), 0.0);
    }
}
