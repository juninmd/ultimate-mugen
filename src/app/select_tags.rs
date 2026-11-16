use super::*;

const CHIP_H: f32 = 20.0;
const CHIP_PAD: f32 = 12.0;
const CHIP_GAP: f32 = 8.0;
const CHIP_FONT: f32 = 12.0;
const CHIP_Y: f32 = 628.0;
const CHIP_X: f32 = 340.0;

impl App {
    pub(crate) fn draw_select_tags(&self, tags: &[String], accent: Color) {
        let mut x = CHIP_X;
        for tag in tags.iter().take(crate::profile_tags::MAX_SHOWN) {
            let text = crate::profile_tags::label(tag);
            let width = self.ui.measure(&text, CHIP_FONT, true) + CHIP_PAD * 2.0;
            self.ui.rect(x, CHIP_Y, width, CHIP_H, alpha(accent, 0.22));
            self.ui.rect(x, CHIP_Y, 3.0, CHIP_H, accent);
            self.ui.text(
                &text,
                x + CHIP_PAD,
                CHIP_Y + 14.5,
                CHIP_FONT,
                WHITE,
                Align::Left,
                true,
            );
            x += width + CHIP_GAP;
        }
    }
}
