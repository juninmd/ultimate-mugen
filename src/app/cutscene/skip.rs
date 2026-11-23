use crate::app::App;
use crate::gfx::*;
use crate::i18n::tr3;
use crate::ui::*;

pub(crate) const SKIP_BOX: (f32, f32, f32, f32) = (1000.0, 8.0, 150.0, 26.0);
const SKIP_HIT_BOX: (f32, f32, f32, f32) = (985.0, 0.0, 180.0, 44.0);

impl App {
    pub(crate) fn skip_chip(&mut self) -> bool {
        let (x, y, w, h) = SKIP_BOX;
        let fill = if self.hover_rect(x, y, w, h) {
            hex("#4a3478")
        } else {
            hex("#1c1433")
        };
        self.ui.rect(x, y, w, h, fill);
        self.ui.rect_lines(x, y, w, h, 1.5, alpha(WHITE, 0.6));
        self.ui.text(
            tr3("PULAR · ESC/START", "SKIP · ESC/START", "SALTAR · ESC/START"),
            x + w / 2.0,
            y + 18.0,
            13.0,
            WHITE,
            Align::Center,
            false,
        );
        self.tapped_in(
            SKIP_HIT_BOX.0,
            SKIP_HIT_BOX.1,
            SKIP_HIT_BOX.2,
            SKIP_HIT_BOX.3,
        )
    }

    pub(crate) fn tap_any(&self) -> bool {
        is_mouse_button_pressed(MouseButton::Left)
            || touches()
                .iter()
                .any(|t| t.phase == TouchPhase::Started)
    }

    pub(crate) fn tapped_in(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let inside = |p: Vec2| {
            let v = self.ui.to_virtual(p);
            v.x >= x && v.x <= x + w && v.y >= y && v.y <= y + h
        };
        let dpi = dpi_scale().max(0.01);
        (is_mouse_button_pressed(MouseButton::Left) && inside(mouse_position().into()))
            || touches()
                .iter()
                .any(|t| t.phase == TouchPhase::Started && inside(t.position / dpi))
    }
}
