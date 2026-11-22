use crate::app::Screen;
use crate::controls::nav;
use crate::gfx::*;
use crate::i18n::tr3;
use crate::poly;
use crate::ui::{alpha, Align, Ui};

const MIN_TOUCH_PX: f32 = 44.0;
const EDGE_PX: f32 = 10.0;

#[derive(Clone, Copy)]
pub struct BackChip {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn has_chip(screen: &Screen) -> bool {
    matches!(
        screen,
        Screen::Select { .. }
            | Screen::Stages { .. }
            | Screen::Tower { .. }
            | Screen::SurvivalOver { .. }
            | Screen::Controls
            | Screen::CharTuning { .. }
            | Screen::Mugen { .. }
            | Screen::Market
            | Screen::Replays { .. }
            | Screen::PlayerCard { .. }
            | Screen::Story { .. }
            | Screen::Settings { .. }
    )
}

#[cfg(target_arch = "wasm32")]
fn insets() -> [f32; 4] {
    crate::web_input::safe_insets()
}

#[cfg(not(target_arch = "wasm32"))]
fn insets() -> [f32; 4] {
    [0.0; 4]
}

fn layout_in(s: f32, ox: f32, oy: f32, screen_w: f32, screen_h: f32, safe: [f32; 4]) -> BackChip {
    let [_, _, bottom, left] = safe;
    let h_px = MIN_TOUCH_PX.max(40.0 * s);
    let w_px = (h_px * 2.9).max(MIN_TOUCH_PX * 2.6);
    let x_px = left + EDGE_PX;
    let y_px = screen_h - bottom - EDGE_PX - h_px;
    BackChip {
        x: (x_px.min(screen_w - w_px) - ox) / s,
        y: (y_px - oy) / s,
        w: w_px / s,
        h: h_px / s,
    }
}

pub fn layout(ui: &Ui) -> BackChip {
    layout_in(ui.s, ui.ox, ui.oy, screen_width(), screen_height(), insets())
}

impl BackChip {
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }

    pub fn draw(&self, ui: &Ui, hot: bool) {
        ui.adapted_plate(
            self.x,
            self.y,
            self.w,
            self.h,
            hot.then(|| crate::tweaks::accent(0.9)),
        );
        let cy = self.y + self.h * 0.5;
        let a = self.h * 0.2;
        let ax = self.x + self.h * 0.5;
        let arrow = ui.pts(vec![
            vec2(ax + a * 0.5, cy - a),
            vec2(ax - a * 0.7, cy),
            vec2(ax + a * 0.5, cy + a),
        ]);
        poly::solid(&arrow, alpha(WHITE, 0.95));
        ui.text(
            tr3("Voltar", "Back", "Volver"),
            self.x + self.h + (self.w - self.h) * 0.45,
            cy + self.h * 0.16,
            self.h * 0.44,
            WHITE,
            Align::Center,
            true,
        );
    }
}

impl crate::app::App {
    pub(crate) fn back_chip_visible(&self) -> bool {
        has_chip(&self.screen) && !self.c.touch.enabled && self.trans <= 0.0
    }

    pub(crate) fn back_chip_input(&mut self) {
        if !self.back_chip_visible() {
            return;
        }
        let chip = layout(&self.ui);
        let p = self.ui.to_virtual(mouse_position().into());
        if is_mouse_button_pressed(MouseButton::Left) && chip.contains(p) {
            swallow_pointer();
            self.audio.sfx("back");
            self.c.nav.press(nav::BACK);
        }
    }

    pub(crate) fn back_chip_draw(&self) {
        if !self.back_chip_visible() {
            return;
        }
        let chip = layout(&self.ui);
        let p = self.ui.to_virtual(mouse_position().into());
        chip.draw(&self.ui, chip.contains(p));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_keeps_a_touch_sized_target_at_any_scale() {
        for s in [0.3_f32, 0.5, 1.0, 2.0] {
            let c = layout_in(s, 0.0, 0.0, 1280.0 * s, 720.0 * s, [0.0, 0.0, 20.0, 30.0]);
            assert!(c.h * s >= MIN_TOUCH_PX - 0.01 && c.w * s >= MIN_TOUCH_PX * 2.0);
            assert!(c.x * s >= 30.0 && (c.y + c.h) * s <= 720.0 * s - 20.0);
        }
    }
}
