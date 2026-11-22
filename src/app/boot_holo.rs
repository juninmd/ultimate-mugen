use super::*;
use crate::i18n::tr3;

const MAX_CARDS: usize = 240;
const BUDGET_SECS: f64 = 1.6;
const SLICE_SECS: f64 = 0.012;
const CARDS_PER_SEC: f64 = 45.0;

impl App {
    pub(super) async fn boot_mosaic(&mut self) {
        let plan: Vec<(usize, usize, usize)> = (0..self.reg.chars.len())
            .filter(|&i| !self.reg.chars[i].boss)
            .take(MAX_CARDS)
            .map(|i| (0, 0, i))
            .collect();
        let label = tr3("Preparando lutadores…", "Preparing fighters…", "Preparando luchadores…");
        let started = get_time();
        let mut done = 0;
        let mut shown = 0;
        while shown < plan.len() && get_time() - started < BUDGET_SECS {
            let slice = get_time();
            while done < plan.len() && get_time() - slice < SLICE_SECS {
                self.warm_portrait(plan[done].2);
                done += 1;
            }
            shown = done.min(((get_time() - started) * CARDS_PER_SEC) as usize);
            self.ui.begin();
            let p = 0.9 + 0.1 * shown as f32 / plan.len() as f32;
            self.holo_mosaic_screen(&plan, shown, p, label);
            next_frame().await;
        }
        self.ui.begin();
        self.holo_mosaic_screen(&plan, plan.len(), 1.0, tr3("Pronto!", "Ready!", "¡Listo!"));
        next_frame().await;
    }

    fn warm_portrait(&self, index: usize) {
        let entry = &self.reg.chars[index];
        if entry.mugen {
            entry.ensure_portrait();
        } else {
            let _ = entry.tile_portrait(1);
        }
    }
}
