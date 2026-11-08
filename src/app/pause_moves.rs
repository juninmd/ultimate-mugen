use super::*;
use ume_engine::input::k;

impl App {
    pub(super) fn pause_moves(&mut self, x: f32, y: f32) {
        let s = self.session.as_ref().unwrap();
        let f = &s.m.fighters[0];
        self.ui.text(
            &format!(
                "{} — {}",
                crate::i18n::t("pause.moves"),
                f.def.display_name.to_uppercase()
            ),
            x + 400.0,
            y + 52.0,
            22.0,
            hex("#22e8ff"),
            Align::Left,
            true,
        );
        const PER_PAGE: usize = 9;
        let total = f.def.movelist.len();
        let pages = total.div_ceil(PER_PAGE).max(1);
        if total > 0 {
            if self.c.pressed(nav::RIGHT) {
                self.move_cur = (self.move_cur + 1) % total;
            }
            if self.c.pressed(nav::LEFT) {
                self.move_cur = (self.move_cur + total - 1) % total;
            }
            self.move_cur = self.move_cur.min(total - 1);
            self.move_page = self.move_cur / PER_PAGE;
        }
        let fighter_name = f.def.display_name.clone();
        let held_slots = self.c.pad_info.first().map_or(0, |p| p.bits & k::SLOT_MASK)
            | [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
                .into_iter()
                .zip(k::SLOTS)
                .filter(|(kc, _)| is_key_down(*kc))
                .fold(0, |b, (_, bit)| b | bit);
        let fresh_slots = held_slots & !self.c.shortcuts.prev[0];
        self.c.shortcuts.prev[0] = held_slots;
        if let (Some(mv), Some(slot)) = (
            f.def.movelist.get(self.move_cur),
            k::SLOTS.iter().position(|bit| fresh_slots & bit != 0),
        ) {
            self.c.shortcuts.assign(&fighter_name, slot, &mv.command);
            self.c.shortcuts.save();
        }
        if total == 0 {
            self.ui.text(
                crate::i18n::t("pause.no_moves"),
                x + 400.0,
                y + 90.0,
                18.0,
                WHITE,
                Align::Left,
                false,
            );
        }
        for (i, mv) in f
            .def
            .movelist
            .iter()
            .enumerate()
            .skip(self.move_page * PER_PAGE)
            .take(PER_PAGE)
        {
            let yy = y + 78.0 + (i - self.move_page * PER_PAGE) as f32 * 46.0;
            if self.clicked_rect(x + 390.0, yy, 630.0, 40.0) {
                self.move_cur = i;
            }
            self.ui.rect(
                x + 390.0,
                yy,
                630.0,
                40.0,
                if i == self.move_cur {
                    Color::new(1.0, 0.3, 0.43, 0.35)
                } else {
                    Color::new(1.0, 1.0, 1.0, if i % 2 == 0 { 0.06 } else { 0.03 })
                },
            );
            if let Some(slot) = self.c.shortcuts.slot_of(&fighter_name, &mv.command) {
                self.ui.text(
                    &format!("[{}]", slot + 1),
                    x + 930.0,
                    yy + 27.0,
                    19.0,
                    hex("#22e8ff"),
                    Align::Right,
                    true,
                );
            }
            self.ui.text(
                &mv.name.chars().take(26).collect::<String>(),
                x + 402.0,
                yy + 27.0,
                19.0,
                WHITE,
                Align::Left,
                false,
            );
            if mv.cost > 0.0 {
                let cost_label = if mv.drain {
                    format!("{:.0}/s PWR", mv.cost)
                } else if mv.cost >= 1000.0 && mv.cost % 1000.0 == 0.0 {
                    let b = (mv.cost / 1000.0) as i32;
                    if b == 1 {
                        "1 BAR".to_string()
                    } else {
                        format!("{b} BARS")
                    }
                } else {
                    format!("{:.0} PWR", mv.cost)
                };
                self.ui.text(
                    &cost_label,
                    x + 1010.0,
                    yy + 27.0,
                    15.0,
                    hex("#ffd13b"),
                    Align::Right,
                    false,
                );
            }
            let c = &self.c;
            self.ui
                .input_icons(&mv.input, x + 690.0, yy + 34.0, 28.0, |b| c.face(b));
        }
        if pages > 1 {
            self.ui.rect(
                x + 880.0,
                y + 20.0,
                60.0,
                44.0,
                Color::new(1.0, 1.0, 1.0, 0.12),
            );
            self.ui.rect(
                x + 950.0,
                y + 20.0,
                60.0,
                44.0,
                Color::new(1.0, 1.0, 1.0, 0.12),
            );
            self.ui.arrow(x + 910.0, y + 42.0, 26.0, (-1.0, 0.0), WHITE);
            self.ui.arrow(x + 980.0, y + 42.0, 26.0, (1.0, 0.0), WHITE);
            if self.clicked_rect(x + 880.0, y + 20.0, 60.0, 44.0) {
                self.move_cur = ((self.move_page + pages - 1) % pages) * PER_PAGE;
            }
            if self.clicked_rect(x + 950.0, y + 20.0, 60.0, 44.0) {
                self.move_cur = ((self.move_page + 1) % pages) * PER_PAGE;
            }
            self.ui.text(
                &format!(
                    "{} {}/{}",
                    crate::i18n::t("pause.page"),
                    self.move_page + 1,
                    pages
                ),
                x + 400.0,
                y + 505.0,
                16.0,
                hex("#a49cc0"),
                Align::Left,
                false,
            );
        }
        if total > 0 {
            self.ui.text(
                crate::i18n::tr("◀ ▶ escolha um golpe e aperte 1-4 / L1 L2 R1 R2 para atribuir um atalho", "◀ ▶ select a move, then press 1-4 / L1 L2 R1 R2 to bind it to a shortcut"),
                x + 400.0,
                y + 505.0 - if pages > 1 { 24.0 } else { 0.0 },
                16.0,
                hex("#a49cc0"),
                Align::Left,
                false,
            );
        }
    }
}
