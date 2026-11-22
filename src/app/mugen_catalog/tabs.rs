use super::{GALLERY, QUARANTINE};
use crate::app::App;
use crate::gfx::*;
use crate::i18n::tr3;
use crate::ui::*;

const TAB_X: f32 = 156.0;
const TAB_Y: f32 = 630.0;
const TAB_W: f32 = 220.0;
const TAB_H: f32 = 36.0;
const TAB_STEP: f32 = 230.0;

impl App {
    pub(super) fn catalog_tabs(&self, view: u8, shown: usize) -> u8 {
        let tabs = [
            (GALLERY, tr3("GALERIA", "GALLERY", "GALERÍA")),
            (QUARANTINE, tr3("QUARENTENA", "QUARANTINE", "CUARENTENA")),
        ];
        let mut picked = view;
        for (i, (id, label)) in tabs.into_iter().enumerate() {
            let x = TAB_X + i as f32 * TAB_STEP;
            if self.clicked_rect(x, TAB_Y, TAB_W, TAB_H) {
                picked = id;
            }
            let active = id == view;
            self.ui.adapted_tab(x, TAB_Y, TAB_W, TAB_H, active);
            let color = if active { WHITE } else { hex("#a49cc0") };
            self.ui.text(
                &if active {
                    format!("{label} · {shown}")
                } else {
                    label.to_string()
                },
                x + TAB_W / 2.0,
                TAB_Y + TAB_H * 0.68,
                18.0,
                color,
                Align::Center,
                active,
            );
        }
        picked
    }
}
