use super::*;

mod actions;
use crate::match_stats::{clock_text, snapshot};

pub(crate) fn performance_rank(life_pct: i32, best_combo: i32, ticks: i64) -> (&'static str, Color) {
    let mut pts = 0i32;
    if life_pct >= 75 {
        pts += 3;
    } else if life_pct >= 45 {
        pts += 2;
    } else if life_pct >= 20 {
        pts += 1;
    }
    if best_combo >= 6 {
        pts += 2;
    } else if best_combo >= 3 {
        pts += 1;
    }
    if ticks <= 45 * 60 {
        pts += 1;
    }
    match pts {
        5.. => ("S", hex("#ffcc33")),
        3..=4 => ("A", hex("#4ce0ff")),
        2 => ("B", hex("#6be89b")),
        _ => ("C", hex("#c9c2e2")),
    }
}

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
        let winner_side = if life_pct(0) >= life_pct(1) { 0 } else { 1 };
        let (rank, rank_color) =
            performance_rank(life_pct(winner_side), stats.best_combo[winner_side], stats.ticks);
        let (rx, ry, rw, rh) = (1034.0, 212.0, 72.0, 72.0);
        self.ui.rect(rx, ry, rw, rh, Color::new(0.05, 0.03, 0.12, 0.85));
        self.ui.rect(rx, ry, rw, 3.0, rank_color);
        self.ui.text(
            crate::i18n::tr3("NOTA", "RANK", "RANGO"),
            rx + rw * 0.5,
            ry + 18.0,
            11.0,
            hex("#a49cc0"),
            Align::Center,
            true,
        );
        self.ui
            .text_bold(rank, rx + rw * 0.5, ry + 58.0, 42.0, rank_color, BLACK, Align::Center);
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
            self.ui.adapted_row(x, y, w, h, k > 0.5);
            if on && k > 0.5 {
                self.ui.adapted_highlight(("res.x", "res.y"), x, y, w, h);
            }
            let icon = if items.len() == 3 && i > 0 { i + 1 } else { i };
            actions::draw_label(&self.ui, icon, it, (x, y, w), if on { k } else { 0.82 * k });
        }
        (cur, clicked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_rank_scales_from_c_to_s() {
        assert_eq!(performance_rank(90, 8, 25 * 60).0, "S");
        assert_eq!(performance_rank(60, 4, 50 * 60).0, "A");
        assert_eq!(performance_rank(25, 3, 60 * 60).0, "B");
        assert_eq!(performance_rank(10, 1, 80 * 60).0, "C");
    }
}
