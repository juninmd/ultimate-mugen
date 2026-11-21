use crate::gfx::*;
use crate::i18n::tr3;
use crate::tweaks::Tweaks;
use crate::ui::{hex, Align, Ui};

pub(super) fn draw(ui: &Ui, prefs: &Tweaks) {
    ui.rect(320.0, 404.0, 640.0, 210.0, Color::new(0.0, 0.0, 0.0, 0.22));
    ui.text(
        tr3(
            "Prévia do controle",
            "Control preview",
            "Vista previa del control",
        ),
        640.0,
        432.0,
        18.0,
        hex("#c9c2e2"),
        Align::Center,
        false,
    );
    let stick_x = if prefs.touch_left_handed {
        850.0
    } else {
        430.0
    };
    let cluster_x = if prefs.touch_left_handed {
        440.0
    } else {
        760.0
    };
    let stick = ui.pts(vec![vec2(stick_x, 530.0)])[0];
    let radius = 42.0 * prefs.touch_scale * ui.s;
    draw_circle(
        stick.x,
        stick.y,
        radius,
        Color::new(1.0, 1.0, 1.0, 0.12 * prefs.touch_opacity),
    );
    draw_circle_lines(
        stick.x,
        stick.y,
        radius,
        2.0,
        Color::new(1.0, 1.0, 1.0, 0.5 * prefs.touch_opacity),
    );
    let labels = ["P", "P+", "II", "K", "K+", "S"];
    for (i, label) in labels.iter().enumerate() {
        let x = cluster_x + (i % 3) as f32 * 54.0;
        let y = 500.0 + (i / 3) as f32 * 54.0;
        let p = ui.pts(vec![vec2(x, y)])[0];
        let color = if i < 2 {
            Color::new(0.2, 0.55, 1.0, 0.5 * prefs.touch_opacity)
        } else {
            Color::new(1.0, 0.27, 0.4, 0.5 * prefs.touch_opacity)
        };
        draw_circle(p.x, p.y, 18.0 * prefs.touch_scale * ui.s, color);
        ui.text(label, x, y + 6.0, 12.0, WHITE, Align::Center, true);
    }
}
