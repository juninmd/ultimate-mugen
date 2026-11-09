use super::advantage::{AdvantageKind, AdvantageResult};
use crate::gfx::*;
use crate::poly;
use crate::ui::{hex, Align};

pub(super) fn draw<P: Fn(f32, f32) -> Vec2, T: Fn(&str, f32, f32, f32, Color, Align, bool)>(
    res: &AdvantageResult,
    p: &P,
    text: &T,
    s: f32,
    w: f32,
) {
    if res.display_ticks == 0 {
        return;
    }
    let alpha = (res.display_ticks as f32 / 20.0).min(1.0);
    let (bw, bh) = (112.0, 13.0);
    let (bx, by) = (w / 2.0 - bw / 2.0, 42.0);

    let bg_pts = poly::slanted(bx, by, bw, bh, 2.5);
    let screen_pts: Vec<Vec2> = bg_pts.into_iter().map(|q| p(q.x, q.y)).collect();
    poly::solid(&screen_pts, Color::new(0.04, 0.06, 0.12, 0.88 * alpha));

    let (border_col, badge_col) = match res.kind {
        AdvantageKind::Knockdown => (Color::new(1.0, 0.75, 0.1, 0.9 * alpha), hex("#ffa502")),
        _ if res.frames > 0 => (Color::new(0.22, 0.94, 0.49, 0.9 * alpha), hex("#2ed573")),
        _ if res.frames < 0 => (Color::new(1.0, 0.28, 0.34, 0.9 * alpha), hex("#ff4757")),
        _ => (Color::new(0.85, 0.85, 0.9, 0.9 * alpha), hex("#ffffff")),
    };
    poly::outline(&screen_pts, 0.65 * s, border_col);

    let frame_str = match res.kind {
        AdvantageKind::Knockdown => "+KD".to_string(),
        _ if res.frames > 0 => format!("+{}", res.frames),
        _ if res.frames < 0 => format!("{}", res.frames),
        _ => "±0".to_string(),
    };

    let status_str = match res.kind {
        AdvantageKind::Knockdown => crate::i18n::tr3("DERRUBADA", "KNOCKDOWN", "DERRIBO"),
        AdvantageKind::Guard if res.frames <= -5 => {
            crate::i18n::tr3("PUNÍVEL", "PUNISHABLE", "CASTIGABLE")
        }
        AdvantageKind::Guard if res.frames >= 0 => crate::i18n::tr3("SEGURO", "SAFE", "SEGURO"),
        _ if res.frames > 0 => crate::i18n::tr3("VANTAGEM", "ADVANTAGE", "VENTAJA"),
        _ => crate::i18n::tr3("DESVANT.", "DISADV.", "DESVENT."),
    };

    let side_tag = if res.attacker_side == 0 { "P1" } else { "P2" };
    text(
        side_tag,
        bx + 7.0,
        by + 8.5,
        5.0,
        Color::new(0.7, 0.8, 1.0, alpha),
        Align::Left,
        true,
    );
    text(
        &frame_str,
        bx + 36.0,
        by + 9.5,
        7.5,
        badge_col,
        Align::Center,
        true,
    );
    text(
        status_str,
        bx + 78.0,
        by + 8.5,
        4.8,
        Color::new(1.0, 1.0, 1.0, 0.9 * alpha),
        Align::Center,
        true,
    );
}
