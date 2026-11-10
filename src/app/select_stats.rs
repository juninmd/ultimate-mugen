use super::*;
use std::cell::RefCell;
use std::collections::HashMap;

pub(crate) const SORT_MODES: usize = 5;

#[derive(Clone, Copy, Default)]
pub(crate) struct CharStats {
    pub force: f32,
    pub life: f32,
    pub speed: f32,
}

thread_local! {
    static CACHE: RefCell<HashMap<String, CharStats>> = RefCell::new(HashMap::new());
}

pub(crate) fn stats_from(attack: f64, life: f64, speed: f64) -> CharStats {
    CharStats {
        force: (attack * life / 1000.0) as f32,
        life: life as f32,
        speed: speed as f32,
    }
}

fn fraction(v: f32, full: f32) -> f32 {
    (v / full).clamp(0.0, 1.0)
}

impl App {
    pub(crate) fn char_stats(&self, i: usize) -> CharStats {
        let key = self.reg.chars[i].key.clone();
        if let Some(s) = CACHE.with(|c| c.borrow().get(&key).copied()) {
            return s;
        }
        let (attack, life, speed) = self.reg.chars[i].stats();
        let stats = stats_from(attack, life, speed);
        CACHE.with(|c| c.borrow_mut().insert(key, stats));
        stats
    }

    pub(crate) fn sort_view(&self, view: &mut [usize]) {
        let mode = self.settings.sort % SORT_MODES;
        if mode == 0 {
            return;
        }
        if mode == 4 {
            view.sort_by(|&a, &b| self.reg.chars[a].name.to_uppercase().cmp(&self.reg.chars[b].name.to_uppercase()));
            return;
        }
        let key = |s: CharStats| match mode {
            1 => s.force,
            2 => s.life,
            _ => s.speed,
        };
        view.sort_by(|&a, &b| {
            key(self.char_stats(b))
                .partial_cmp(&key(self.char_stats(a)))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub(crate) fn sort_label(&self) -> &'static str {
        [
            crate::i18n::t("select.sort_roster"),
            crate::i18n::t("select.sort_force"),
            crate::i18n::t("select.sort_life"),
            crate::i18n::t("select.sort_speed"),
            crate::i18n::t("select.sort_name"),
        ][self.settings.sort % SORT_MODES]
    }

    pub(crate) fn handle_select_keys(&mut self) {
        if is_key_pressed(KeyCode::G) {
            self.cycle_grid();
        }
        if is_key_pressed(KeyCode::O) {
            self.settings.sort = (self.settings.sort + 1) % SORT_MODES;
            self.settings.save();
            self.audio.sfx("select");
        }
    }

    pub(crate) fn draw_tile_force(&self, cx: f32, cy: f32, cw: f32, ch: f32, i: usize) {
        let s = self.char_stats(i);
        let f = fraction(s.force, 200.0);
        self.ui.rect(cx, cy + ch - 3.0, cw, 3.0, Color::new(0.0, 0.0, 0.0, 0.5));
        self.ui.rect(cx, cy + ch - 3.0, cw * f, 3.0, hex("#ffcc33"));
    }

    pub(crate) fn draw_stat_gauges(&self, px: f32, y: f32, pw: f32, i: usize) {
        let s = self.char_stats(i);
        let rows = [
            (crate::i18n::t("select.stat_force"), fraction(s.force, 200.0), hex("#ffcc33")),
            (crate::i18n::t("select.stat_life"), fraction(s.life, 2000.0), hex("#3fd17a")),
            (crate::i18n::t("select.stat_speed"), fraction(s.speed, 6.0), hex("#3fa9ff")),
        ];
        let third = pw / 3.0;
        self.ui.rect(px + 8.0, y + 1.0, pw - 16.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.12));
        for (k, (label, f, col)) in rows.iter().enumerate() {
            let x = px + k as f32 * third + 6.0;
            self.ui.text(label, x, y - 2.0, 11.0, WHITE, Align::Left, true);
            self.ui.rect(x + 34.0, y - 9.0, third - 46.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.6));
            self.ui.rect(x + 34.0, y - 9.0, (third - 46.0) * f, 6.0, *col);
        }
    }
}
