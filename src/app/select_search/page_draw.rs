use super::*;
use crate::i18n::tr3;

impl App {
    pub(super) fn draw_search_page(&self, hits: &[usize], rows: usize) {
        self.ui.background();
        self.ui.text(
            tr3("BUSCAR LUTADOR", "SEARCH FIGHTER", "BUSCAR LUCHADOR"),
            40.0,
            56.0,
            30.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        self.ui
            .rect(40.0, 84.0, 1200.0, 50.0, Color::new(1.0, 1.0, 1.0, 0.1));
        self.ui
            .rect_lines(40.0, 84.0, 1200.0, 50.0, 2.0, hex("#3fa9ff"));
        let blink = if (self.ui.t * 2.0) as i32 % 2 == 0 {
            "_"
        } else {
            " "
        };
        let shown = format!("{}{}", QUERY.with(|q| q.borrow().clone()), blink);
        self.ui
            .text(&shown, 56.0, 119.0, 26.0, WHITE, Align::Left, true);
        let count = format!("{} / {}", hits.len(), self.playable().len());
        self.ui.text(
            &count,
            1240.0,
            70.0,
            16.0,
            hex("#a49cc0"),
            Align::Right,
            false,
        );
        let pick = PICK.with(|p| p.get()).min(hits.len().saturating_sub(1));
        let top = pick.saturating_sub(rows - 1);
        for (row, &i) in hits.iter().skip(top).take(rows).enumerate() {
            let y = ROW_Y + row as f32 * ROW_STEP;
            let on = top + row == pick;
            self.ui.rect(
                40.0,
                y,
                1200.0,
                34.0,
                Color::new(1.0, 1.0, 1.0, if on { 0.18 } else { 0.05 }),
            );
            if on {
                self.ui
                    .rect_lines(40.0, y, 1200.0, 34.0, 2.0, hex("#ffcc33"));
            }
            let e = &self.reg.chars[i];
            self.ui.rect(40.0, y, 6.0, 34.0, rgb(e.color));
            self.ui
                .text(&e.name, 62.0, y + 24.0, 20.0, WHITE, Align::Left, true);
            let origin = if e.mugen {
                "M.U.G.E.N"
            } else {
                tr3("Original", "Original", "Original")
            };
            let credit = format!("{origin} · {} · {}", e.series, e.author);
            self.ui.text(
                &crate::credit_line::truncate(&credit, 60),
                520.0,
                y + 23.0,
                14.0,
                hex("#a49cc0"),
                Align::Left,
                false,
            );
            let s = self.char_stats(i);
            self.ui.text(
                &format!("{:.0} / {:.0} / {:.1}", s.force, s.life, s.speed),
                1224.0,
                y + 23.0,
                14.0,
                hex("#a49cc0"),
                Align::Right,
                false,
            );
        }
        if hits.is_empty() {
            self.ui.text(
                tr3(
                    "Nenhum lutador encontrado",
                    "No fighters found",
                    "Ningún luchador encontrado",
                ),
                40.0,
                190.0,
                20.0,
                hex("#ff4d6d"),
                Align::Left,
                true,
            );
        }
        let tip = tr3(
            "Digite nome, autor ou série · Setas escolhem · Enter aplica · Esc limpa",
            "Type a name, author or series · Arrows pick · Enter applies · Esc clears",
            "Escribe nombre, autor o serie · Flechas eligen · Enter aplica · Esc limpia",
        );
        if !self.osk_shown() {
            self.ui
                .text(tip, 40.0, 700.0, 15.0, hex("#a79fc4"), Align::Left, false);
        }
    }
}
