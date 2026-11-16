//! Card tab: portrait, identity, rank, record tiles, top fighters, recent sets and medals.
use super::player_card_screen::{EDIT_H, EDIT_W, EDIT_X, EDIT_Y};
use super::*;
use crate::i18n::tr3;
use crate::player_card::{achievements, matches_word, rank, rank_name, seeking_label};

const LEFT_X: f32 = 40.0;
const TOP: f32 = 196.0;
const LEFT_W: f32 = 420.0;
const LEFT_H: f32 = 500.0;
const RIGHT_X: f32 = 480.0;
const RIGHT_W: f32 = 760.0;
const CARD_W: f32 = 240.0;
const CARD_H: f32 = 150.0;

impl App {
    fn stat_tile(&self, x: f32, y: f32, w: f32, h: f32, label: &str, value: &str) {
        self.ui.adapted_glass(x, y, w, h, crate::tweaks::accent(0.6), 1.0);
        self.ui.text(label, x + 14.0, y + 24.0, 15.0, hex("#a49cc0"), Align::Left, false);
        self.ui.text(value, x + 14.0, y + h - 14.0, 26.0, WHITE, Align::Left, true);
    }

    pub(super) fn card_tab(&mut self) {
        let main = self.main_fighter();
        let color = main.map(|i| rgb(self.reg.chars[i].color)).unwrap_or(hex("#5a648c"));
        let style = main
            .and_then(|i| crate::profile::of(&self.reg.chars[i]))
            .and_then(|p| p.tags.first())
            .map(|t| crate::profile_tags::label(t))
            .unwrap_or_else(|| "—".into());
        let card = self.player.card.clone();
        let rk = rank(self.player.xp);
        let title = match card.title.trim() {
            "" => rank_name(rk.index).to_string(),
            t => t.to_string(),
        };
        let name = self.player_name();
        let wr = format!("{:.0}%", self.player.win_rate() * 100.0);
        let stats = [
            (tr3("Partidas", "Matches", "Partidas"), self.player.matches.to_string()),
            (tr3("Taxa de vitória", "Win rate", "Victorias"), wr),
            (tr3("Maior sequência", "Best streak", "Mejor racha"), self.player.best_streak.to_string()),
            (tr3("Lutadores usados", "Fighters used", "Luchadores"), self.player.roster.len().to_string()),
        ];
        let top: Vec<(String, u32, Color)> = self
            .player
            .top_roster(3)
            .into_iter()
            .map(|(id, count)| {
                let c = self.fighter_index(id).map(|i| rgb(self.reg.chars[i].color)).unwrap_or(hex("#5a648c"));
                (self.fighter_name(id), count, c)
            })
            .collect();
        let sets: Vec<(bool, String, String)> = self
            .player
            .sets
            .iter()
            .take(5)
            .map(|s| (s.won, self.fighter_name(&s.fighter), self.fighter_name(&s.rival)))
            .collect();
        let medals: Vec<(&str, usize)> = achievements(&self.player).iter().map(|a| (a.title, a.level())).collect();

        self.ui.adapted_glass(LEFT_X, TOP, LEFT_W, LEFT_H, color, 1.0);
        self.ui.adapted_plate(60.0, 216.0, 170.0, 200.0, Some(color));
        if let Some(i) = main {
            self.portrait_fit(i, 66.0, 222.0, 158.0, 188.0);
        }
        let ui = &self.ui;
        ui.text(&name, 250.0, 240.0, 28.0, hex("#ffcc33"), Align::Left, true);
        ui.flag(&card.flag, 250.0, 256.0, 54.0, 1.0);
        ui.text(&card.flag, 316.0, 282.0, 18.0, hex("#a49cc0"), Align::Left, false);
        ui.text(&title, 250.0, 330.0, 20.0, WHITE, Align::Left, true);
        ui.text(&format!("{}: {}", tr3("Estilo", "Style", "Estilo"), style), 250.0, 354.0, 17.0, hex("#a49cc0"), Align::Left, false);
        let seek = seeking_label(card.seeking);
        let chip_w = ui.measure(seek, 16.0, false) + 24.0;
        ui.adapted_plate(250.0, 366.0, chip_w, 28.0, Some(crate::tweaks::accent(0.5)));
        ui.text(seek, 262.0, 385.0, 16.0, WHITE, Align::Left, false);

        ui.text(
            &format!("{} {}", rank_name(rk.index), rk.index + 1),
            60.0,
            430.0,
            20.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        let xp_text = match rk.next {
            Some(next) => format!("XP {}/{}", self.player.xp, next),
            None => "MAX".to_string(),
        };
        ui.text(&xp_text, 430.0, 430.0, 16.0, hex("#a49cc0"), Align::Right, false);
        ui.adapted_meter(60.0, 442.0, 370.0, rk.progress);

        let tiles = [(60.0, 476.0), (250.0, 476.0), (60.0, 562.0), (250.0, 562.0)];
        for ((x, y), (label, value)) in tiles.iter().zip(stats.iter()) {
            self.stat_tile(*x, *y, 180.0, 78.0, label, value);
        }
        self.ui.adapted_row(EDIT_X, EDIT_Y, EDIT_W, EDIT_H, true);
        self.ui.text(
            tr3("Editar card", "Edit card", "Editar ficha"),
            EDIT_X + EDIT_W / 2.0,
            EDIT_Y + EDIT_H * 0.7,
            18.0,
            WHITE,
            Align::Center,
            true,
        );

        for i in 0..3 {
            let x = RIGHT_X + i as f32 * (CARD_W + 20.0);
            let (fname, count, c) = top.get(i).cloned().unwrap_or((String::new(), 0, hex("#2a2140")));
            self.ui.adapted_glass(x, TOP, CARD_W, CARD_H, c, 1.0);
            let label = if fname.is_empty() { tr3("Sem dados", "No data", "Sin datos").to_string() } else { fname };
            self.ui.text(&label, x + 16.0, TOP + 44.0, 22.0, WHITE, Align::Left, true);
            self.ui.text(
                &format!("{count} {}", matches_word(count)),
                x + 16.0,
                TOP + 78.0,
                17.0,
                hex("#e6e0f5"),
                Align::Left,
                false,
            );
        }

        self.ui.text(tr3("Sets recentes", "Recent sets", "Sets recientes"), RIGHT_X, 386.0, 22.0, hex("#ffcc33"), Align::Left, true);
        if sets.is_empty() {
            self.ui.text(tr3("Nenhuma partida ainda", "No matches yet", "Sin partidas aún"), RIGHT_X, 420.0, 18.0, hex("#a49cc0"), Align::Left, false);
        }
        for (i, (won, fighter, rival)) in sets.iter().enumerate() {
            let y = 404.0 + i as f32 * 40.0;
            self.ui.adapted_row(RIGHT_X, y, RIGHT_W, 34.0, true);
            let tint = if *won { hex("#2ecc71") } else { hex("#ff4d6d") };
            self.ui.adapted_plate(RIGHT_X + 12.0, y + 5.0, 110.0, 24.0, Some(alpha(tint, 0.8)));
            let verdict = if *won { tr3("VITÓRIA", "WIN", "VICTORIA") } else { tr3("DERROTA", "LOSS", "DERROTA") };
            self.ui.text(verdict, RIGHT_X + 67.0, y + 22.0, 14.0, WHITE, Align::Center, true);
            self.ui.text(&format!("{fighter}  vs  {rival}"), RIGHT_X + 140.0, y + 23.0, 19.0, WHITE, Align::Left, false);
        }

        self.ui.text(tr3("Medalhas", "Medals", "Medallas"), RIGHT_X, 626.0, 22.0, hex("#ffcc33"), Align::Left, true);
        for (i, (medal, level)) in medals.iter().enumerate() {
            let x = RIGHT_X + i as f32 * 153.5;
            self.ui.adapted_glass(x, 638.0, 146.0, 56.0, crate::tweaks::accent(0.6), 1.0);
            self.ui.text(medal, x + 12.0, 660.0, 14.0, WHITE, Align::Left, true);
            self.ui.adapted_level(x + 12.0, 668.0, 3, *level);
        }
    }
}
