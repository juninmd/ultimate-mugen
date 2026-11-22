use super::*;

const CHIP_H: f32 = 20.0;
const CHIP_PAD: f32 = 12.0;
const CHIP_GAP: f32 = 8.0;
const CHIP_FONT: f32 = 12.0;
const CHIP_Y: f32 = 628.0;
const CHIP_X: f32 = 340.0;

impl App {
    pub(crate) fn draw_select_tags(&self, profile: &ume_native::CharProfile, accent: Color) {
        let mut x = CHIP_X;
        if let Some((label, rgb)) = profile
            .alignment
            .as_deref()
            .and_then(crate::profile::alignment_label)
        {
            let col = Color::new(rgb[0], rgb[1], rgb[2], 1.0);
            x = self.draw_chip(x, label, col);
        }
        if let Some(el) = profile.element.as_deref() {
            let text = match profile.weakness.as_deref() {
                Some(w) => format!("{} / -{}", el.to_uppercase(), w.to_uppercase()),
                None => el.to_uppercase(),
            };
            x = self.draw_chip(x, &text, accent);
        }
        for tag in profile.tags.iter().take(crate::profile_tags::MAX_SHOWN) {
            let text = crate::profile_tags::label(tag);
            x = self.draw_chip(x, &text, accent);
        }
    }

    fn draw_chip(&self, x: f32, text: &str, color: Color) -> f32 {
        let width = self.ui.measure(text, CHIP_FONT, true) + CHIP_PAD * 2.0;
        self.ui.rect(x, CHIP_Y, width, CHIP_H, alpha(color, 0.22));
        self.ui.rect(x, CHIP_Y, 3.0, CHIP_H, color);
        self.ui.text(text, x + CHIP_PAD, CHIP_Y + 14.5, CHIP_FONT, WHITE, Align::Left, true);
        x + width + CHIP_GAP
    }
}
