use super::*;
use crate::match_stats::{clock_text, snapshot};

impl App {
    pub(super) fn draw_results_stats(&self) {
        let Some(sess) = self.session.as_ref() else {
            return;
        };
        let stats = snapshot();
        let life_pct = |side: usize| -> i32 {
            let (life, max) = sess
                .m
                .roots()
                .filter(|f| f.side == side)
                .fold((0.0f32, 0.0f32), |a, f| (a.0 + f.life.max(0.0), a.1 + f.life_max));
            if max > 0.0 {
                (life / max * 100.0).round() as i32
            } else {
                0
            }
        };
        let chips = [
            (crate::i18n::t("results.time"), clock_text(stats.ticks)),
            (
                crate::i18n::t("results.best_combo"),
                format!("{}  -  {}", stats.best_combo[0], stats.best_combo[1]),
            ),
            (
                crate::i18n::t("results.damage"),
                format!(
                    "{}  -  {}",
                    stats.damage_dealt[0].round() as i32,
                    stats.damage_dealt[1].round() as i32
                ),
            ),
            (
                crate::i18n::t("results.life_left"),
                format!("{}%  -  {}%", life_pct(0), life_pct(1)),
            ),
        ];
        let (x0, y, w, h, gap) = (480.0, 318.0, 150.0, 52.0, 10.0);
        for (i, (label, value)) in chips.iter().enumerate() {
            let x = x0 + i as f32 * (w + gap);
            self.ui.rect(x, y, w, h, Color::new(0.04, 0.03, 0.1, 0.55));
            self.ui.rect(x, y, w, 2.0, Color::new(1.0, 1.0, 1.0, 0.5));
            self.ui
                .text(label, x + w * 0.5, y + 19.0, 13.0, hex("#9fb4d6"), Align::Center, false);
            self.ui
                .text(value, x + w * 0.5, y + 42.0, 20.0, WHITE, Align::Center, true);
        }
    }

    pub(super) fn results_title(&self, winner: usize, lead: usize, done: bool) -> String {
        if done {
            return crate::i18n::t("results.arcade_complete").to_string();
        }
        let name = |idx: usize| self.reg.chars[idx].name.to_uppercase();
        match self.team.as_ref() {
            Some(t) => {
                let first = t.members[winner.min(1)].first().copied().unwrap_or(lead);
                let word = self.reg.chars[first].name.split_whitespace().next().unwrap_or("").to_uppercase();
                crate::i18n::tf("results.team_wins", &[("{name}", &word)])
            }
            None => crate::i18n::tf("results.wins", &[("{name}", &name(lead))]),
        }
    }

    pub(super) fn results_panel(&self, accent: Color) {
        let k = self.results_enter();
        let (x, y, w, h) = (160.0, 200.0 + (1.0 - k) * 24.0, 960.0, 300.0);
        self.ui
            .gradient_v(x, y, w, h, alpha(accent, 0.55 * k), Color::new(0.06, 0.04, 0.14, 0.92 * k));
        crate::menu_art::glass(&self.ui, x, y, w, h, k);
        self.ui.rect(x, y, 6.0, h, alpha(accent, k));
        self.ui.rect(x, y, w, 3.0, alpha(accent, 0.9 * k));
    }

    fn results_enter(&self) -> f32 {
        let t = (self.screen_t / 0.3).clamp(0.0, 1.0);
        1.0 - (1.0 - t).powi(3)
    }

    pub(super) fn results_buttons(&self, items: &[&str], cur: usize) -> (usize, bool) {
        let k = self.results_enter();
        let (x0, y, h, gap) = (480.0, 382.0, 52.0, 10.0);
        let span = 4.0 * 150.0 + 3.0 * gap;
        let n = items.len().max(1) as f32;
        let w = (span - gap * (n - 1.0)) / n;
        let (mut cur, mut clicked) = (cur, false);
        for (i, it) in items.iter().enumerate() {
            let x = x0 + i as f32 * (w + gap);
            if self.hover_rect(x, y, w, h) && mouse_delta_position().length() > 0.0 {
                cur = i;
            }
            if self.clicked_rect(x, y, w, h) {
                cur = i;
                clicked = true;
            }
            let on = i == cur;
            let fill = if on {
                let pulse = 0.9 + 0.1 * (self.ui.t * 5.0).sin();
                Color::new(1.0 * pulse, 0.30 * pulse, 0.43 * pulse, k)
            } else {
                Color::new(1.0, 1.0, 1.0, 0.08 * k)
            };
            self.ui.rect(x, y, w, h, fill);
            self.ui.rect(x, y + h - 2.0, w, 2.0, Color::new(1.0, 1.0, 1.0, if on { 0.6 * k } else { 0.14 * k }));
            self.ui.text(
                it,
                x + w * 0.5,
                y + 34.0,
                20.0,
                Color::new(1.0, 1.0, 1.0, if on { k } else { 0.82 * k }),
                Align::Center,
                true,
            );
        }
        (cur, clicked)
    }
}
