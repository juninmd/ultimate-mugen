use crate::app::App;
use crate::gfx::*;
use crate::ui::*;

const FEATURED: [&str; 16] = [
    "kaito", "suzume", "bruno", "akio", "shion", "daichi", "jin", "lian", "garan", "marlow", "kaname", "volta", "sylene",
    "haruki", "valdric", "oboro",
];
const STAGE_CX: f32 = 860.0;
const STAGE_BOTTOM: f32 = 545.0;
const FIT_W: f32 = 560.0;
const FIT_H: f32 = 460.0;
const SLIDE: f32 = 90.0;

impl App {
    pub(super) fn menu_backdrop(&mut self, cur: usize) {
        self.menu_art.select(cur);
        self.menu_art.draw(&self.ui);
        let (prev, now, e) = self.menu_art.transition();
        self.menu_fighter_load(now, e);
        if e < 1.0 {
            self.menu_fighter_draw(prev, 1.0 - e, SLIDE * e);
        }
        self.menu_fighter_draw(now, e, -SLIDE * (1.0 - e));
    }

    fn menu_fighter_index(&self, item: usize) -> Option<usize> {
        let id = FEATURED[item % FEATURED.len()];
        self.reg.chars.iter().position(|c| !c.mugen && c.id.ends_with(id))
    }

    fn menu_fighter_load(&mut self, item: usize, e: f32) {
        let n = FEATURED.len();
        let wanted = [item, (item + 1) % n, (item + n - 1) % n];
        let count = if e >= 1.0 { 3 } else { 1 };
        for &slot in wanted.iter().take(count) {
            let Some(i) = self.menu_fighter_index(slot) else { continue };
            if self.reg.chars[i].peek(1).is_none() && !self.reg.chars[i].broken.get() {
                let _ = self.reg.chars[i].load_preview(1);
                return;
            }
        }
    }

    fn menu_fighter_draw(&mut self, item: usize, alpha_k: f32, slide: f32) {
        if alpha_k <= 0.01 {
            return;
        }
        let Some(i) = self.menu_fighter_index(item) else { return };
        let Some(def) = self.reg.chars[i].peek(1) else { return };
        let col = rgb(self.reg.chars[i].color);
        let s = self.ui.s;
        let cx = STAGE_CX + slide;
        glow(self.ui.x(cx), self.ui.y(STAGE_BOTTOM - FIT_H * 0.5), 300.0 * s, alpha(col, 0.42 * alpha_k));
        draw_ellipse(self.ui.x(cx), self.ui.y(STAGE_BOTTOM + 6.0), 150.0 * s, 16.0 * s, 0.0, Color::new(0.0, 0.0, 0.0, 0.45 * alpha_k));
        self.char_tint = Color::new(1.0, 1.0, 1.0, alpha_k);
        let t = self.preview_t;
        self.draw_char_fit(&def, 0, t, cx, STAGE_BOTTOM, FIT_W, FIT_H, true);
        self.char_tint = WHITE;
    }
}

impl crate::menu_art::MenuArt {
    pub fn transition(&self) -> (usize, usize, f32) {
        (self.prev, self.cur, self.fade * self.fade * (3.0 - 2.0 * self.fade))
    }
}
