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
}
