use super::*;
use crate::i18n::tr3;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

static WARM_PENDING: AtomicBool = AtomicBool::new(false);
static WARM_CURSOR: AtomicUsize = AtomicUsize::new(0);

pub(super) fn request_warm() {
    WARM_CURSOR.store(0, Relaxed);
    WARM_PENDING.store(true, Relaxed);
}

const MAX_CARDS: usize = 240;
const BUDGET_SECS: f64 = 1.6;
const SLICE_SECS: f64 = 0.012;
const CARDS_PER_SEC: f64 = 45.0;
const EXTERNAL_TIMEOUT: f64 = 20.0;

impl App {
    fn gate_plan(&self) -> Vec<(usize, usize, usize)> {
        (0..self.reg.chars.len())
            .filter(|&i| !self.reg.chars[i].boss)
            .take(MAX_CARDS)
            .map(|i| (0, 0, i))
            .collect()
    }

    pub(crate) fn holo_gate_draw(&mut self, p: f32, label: &str) -> bool {
        let plan = self.gate_plan();
        let slice = get_time();
        let mut cur = WARM_CURSOR.load(Relaxed).min(plan.len());
        while cur < plan.len() && get_time() - slice < SLICE_SECS {
            self.warm_portrait(plan[cur].2);
            cur += 1;
        }
        WARM_CURSOR.store(cur, Relaxed);
        self.holo_mosaic_screen(&plan, cur, p, label);
        cur >= plan.len()
    }

    pub(super) fn holo_gate_frame(&mut self) -> bool {
        if matches!(self.screen, Screen::Fight | Screen::Vs { .. } | Screen::Results { .. } | Screen::Cutscene { .. }) {
            return false;
        }
        #[cfg(not(target_arch = "wasm32"))]
        let import = (!self.import_jobs.is_empty()).then(|| self.import_gate_label());
        #[cfg(target_arch = "wasm32")]
        let import: Option<(f32, String)> = None;
        if import.is_none() && !WARM_PENDING.load(Relaxed) {
            return false;
        }
        let importing = import.is_some();
        let (p, label) = import.unwrap_or((1.0, tr3("Preparando lutadores…", "Preparing fighters…", "Preparando luchadores…").into()));
        let warmed = self.holo_gate_draw(p, &label);
        if !importing && warmed {
            WARM_PENDING.store(false, Relaxed);
        }
        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn import_gate_label(&self) -> (f32, String) {
        let (done, total, cur) = match &self.import_progress {
            Some(p) => (
                p.done.load(Relaxed),
                p.total.load(Relaxed),
                p.current_name.lock().map(|s| s.clone()).unwrap_or_default(),
            ),
            None => (0, 0, String::new()),
        };
        let ratio = if total > 0 { (done as f32 / total as f32).clamp(0.0, 1.0) } else { 0.15 + 0.15 * (self.ui.t * 3.0).sin().abs() };
        let head = tr3("Importando recursos MUGEN", "Importing MUGEN resources", "Importando recursos MUGEN");
        let name = if cur.is_empty() { String::new() } else { format!(" · {cur}") };
        (ratio, format!("{head}{name} ({done}/{total})"))
    }

    async fn boot_external(&mut self) {
        let started = get_time();
        while self.market.loading_installed() && get_time() - started < EXTERNAL_TIMEOUT {
            self.market.poll();
            if self.market.has_registry_work() && self.market.apply_to(&mut self.reg) {
                self.after_registry_change();
            }
            let t = (get_time() - started) as f32;
            self.ui.begin();
            let label = tr3("Carregando pacotes instalados…", "Loading installed packs…", "Cargando paquetes instalados…");
            self.holo_gate_draw(0.9 * (1.0 - 1.0 / (1.0 + t)), label);
            next_frame().await;
        }
    }

    pub(super) async fn boot_mosaic(&mut self) {
        self.boot_external().await;
        let plan = self.gate_plan();
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
        let menu_label = tr3("Preparando o menu…", "Preparing the menu…", "Preparando el menú…");
        for item in 0..MENU.len() {
            self.menu_fighter_load(item, 1.0);
            self.ui.begin();
            self.menu_fighter_draw(item, 0.02, 0.0);
            self.holo_mosaic_screen(&plan, plan.len(), 1.0, menu_label);
            next_frame().await;
        }
        self.ui.begin();
        self.holo_mosaic_screen(&plan, plan.len(), 1.0, tr3("Pronto!", "Ready!", "¡Listo!"));
        next_frame().await;
        WARM_PENDING.store(false, Relaxed);
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
