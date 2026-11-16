//! List tabs of the player card: fighters, match history, rivals and achievements.
use super::*;
use crate::i18n::tr3;
use crate::player_card::{achievements, matches_word};

const LIST_TOP: f32 = 212.0;

impl App {
    fn empty_hint(&self) {
        self.ui.text(
            tr3("Jogue uma partida para preencher esta aba", "Play a match to fill this tab", "Juega una partida para llenar esta pestaña"),
            VW / 2.0,
            VH / 2.0,
            22.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
    }

    pub(super) fn roster_tab(&mut self) {
        let list: Vec<(String, u32)> = self.player.top_roster(12).into_iter().map(|(id, c)| (id.to_string(), c)).collect();
        if list.is_empty() {
            return self.empty_hint();
        }
        for (n, (id, count)) in list.iter().enumerate() {
            let (x, y) = (40.0 + (n % 4) as f32 * 306.0, LIST_TOP + (n / 4) as f32 * 124.0);
            let color = self.fighter_index(id).map(|i| rgb(self.reg.chars[i].color)).unwrap_or(hex("#5a648c"));
            self.ui.adapted_glass(x, y, 280.0, 110.0, color, 1.0);
            self.ui.text(&self.fighter_name(id), x + 18.0, y + 48.0, 24.0, WHITE, Align::Left, true);
            self.ui.text(
                &format!("{count} {}", matches_word(*count)),
                x + 18.0,
                y + 82.0,
                18.0,
                hex("#e6e0f5"),
                Align::Left,
                false,
            );
            self.ui.text(&format!("#{}", n + 1), x + 262.0, y + 30.0, 18.0, hex("#ffcc33"), Align::Right, true);
        }
    }

    pub(super) fn matches_tab(&mut self) {
        let p = &self.player;
        let summary = format!(
            "{} {}  ·  {} {}  ·  {} {}",
            p.matches,
            tr3("partidas", "matches", "partidas"),
            p.wins,
            tr3("vitórias", "wins", "victorias"),
            p.matches - p.wins,
            tr3("derrotas", "losses", "derrotas"),
        );
        self.ui.text(&summary, 40.0, LIST_TOP, 22.0, hex("#ffcc33"), Align::Left, true);
        let sets: Vec<(bool, String, String)> = self
            .player
            .sets
            .iter()
            .map(|s| (s.won, self.fighter_name(&s.fighter), self.fighter_name(&s.rival)))
            .collect();
        if sets.is_empty() {
            return self.empty_hint();
        }
        for (i, (won, fighter, rival)) in sets.iter().enumerate() {
            let y = LIST_TOP + 40.0 + i as f32 * 40.0;
            self.ui.adapted_row(40.0, y, 1200.0, 34.0, true);
            let tint = if *won { hex("#2ecc71") } else { hex("#ff4d6d") };
            self.ui.adapted_plate(56.0, y + 5.0, 110.0, 24.0, Some(alpha(tint, 0.8)));
            let verdict = if *won { tr3("VITÓRIA", "WIN", "VICTORIA") } else { tr3("DERROTA", "LOSS", "DERROTA") };
            self.ui.text(verdict, 111.0, y + 22.0, 14.0, WHITE, Align::Center, true);
            self.ui.text(&format!("{fighter}  vs  {rival}"), 190.0, y + 23.0, 19.0, WHITE, Align::Left, false);
        }
    }

    pub(super) fn friends_tab(&mut self) {
        let mut rivals: Vec<(String, [u32; 2])> = self.player.rivals.iter().map(|(k, v)| (k.clone(), *v)).collect();
        rivals.sort_by(|a, b| (b.1[0] + b.1[1]).cmp(&(a.1[0] + a.1[1])).then(a.0.cmp(&b.0)));
        rivals.truncate(8);
        self.ui.text(
            tr3("Adversários frequentes (neste aparelho)", "Frequent rivals (this device)", "Rivales frecuentes (este dispositivo)"),
            40.0,
            LIST_TOP,
            22.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        if rivals.is_empty() {
            return self.empty_hint();
        }
        for (i, (id, [won, lost])) in rivals.iter().enumerate() {
            let y = LIST_TOP + 30.0 + i as f32 * 52.0;
            self.ui.adapted_row(40.0, y, 1200.0, 44.0, true);
            self.ui.text(&self.fighter_name(id), 62.0, y + 29.0, 22.0, WHITE, Align::Left, true);
            let record = format!(
                "{} {won}  ·  {} {lost}",
                tr3("Vitórias", "Wins", "Victorias"),
                tr3("Derrotas", "Losses", "Derrotas"),
            );
            self.ui.text(&record, 1220.0, y + 29.0, 19.0, hex("#e6e0f5"), Align::Right, false);
        }
    }

    pub(super) fn achievements_tab(&mut self) {
        let list = achievements(&self.player);
        for (i, a) in list.iter().enumerate() {
            let y = LIST_TOP + i as f32 * 96.0;
            self.ui.adapted_row(40.0, y, 1200.0, 84.0, true);
            self.ui.text(a.title, 66.0, y + 36.0, 26.0, hex("#ffcc33"), Align::Left, true);
            self.ui.text(a.goal, 66.0, y + 66.0, 18.0, hex("#a49cc0"), Align::Left, false);
            self.ui.adapted_level(640.0, y + 32.0, 3, a.level());
            let progress = match a.next_tier() {
                Some(t) => format!("{} / {}", a.value, t),
                None => format!("{} · MAX", a.value),
            };
            self.ui.text(&progress, 1220.0, y + 52.0, 24.0, WHITE, Align::Right, true);
        }
    }
}
