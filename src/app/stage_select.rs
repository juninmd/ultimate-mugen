use super::*;

impl App {
    pub(super) fn stages(&mut self) {
        let Screen::Stages { mut cur } = self.screen else {
            return;
        };
        let n = self.reg.stages.len() + 1;
        let cols = 4usize;
        let total_rows = n.div_ceil(cols);
        if self.c.pressed(nav::RIGHT) {
            cur = (cur + 1) % n;
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::LEFT) {
            cur = (cur + n - 1) % n;
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::DOWN) {
            cur = (cur + cols).min(n - 1);
        }
        if self.c.pressed(nav::UP) {
            cur = cur.saturating_sub(cols);
        }
        if is_key_pressed(KeyCode::PageDown) {
            cur = (cur + cols * VIEW_ROWS).min(n - 1);
        }
        if is_key_pressed(KeyCode::PageUp) {
            cur = cur.saturating_sub(cols * VIEW_ROWS);
        }
        let wheel = mouse_wheel().1;
        if wheel > 0.0 {
            cur = cur.saturating_sub(cols);
        } else if wheel < 0.0 {
            cur = (cur + cols).min(n - 1);
        }
        let crow = cur / cols;
        if crow < self.stage_row {
            self.stage_row = crow;
        }
        if crow >= self.stage_row + VIEW_ROWS {
            self.stage_row = crow + 1 - VIEW_ROWS;
        }
        self.stage_row = self.stage_row.min(total_rows.saturating_sub(VIEW_ROWS));
        let mut ok = self.c.pressed(nav::OK | nav::START);
        self.ui.background();
        self.ui.text(
            crate::i18n::t("select.choose_stage"),
            40.0,
            56.0,
            30.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        self.ui.text(
            &format!("{}/{} · PgUp/PgDn · {}", cur + 1, n, crate::i18n::tr3("roda do mouse", "mouse wheel", "rueda del ratón")),
            VW - 40.0,
            56.0,
            18.0,
            hex("#a49cc0"),
            Align::Right,
            false,
        );
        let (cw, ch) = (290.0, 134.0);
        for i in (self.stage_row * cols)..((self.stage_row + VIEW_ROWS) * cols).min(n) {
            let (x, y) = (
                40.0 + (i % cols) as f32 * (cw + 13.0),
                88.0 + ((i / cols) - self.stage_row) as f32 * (ch + 14.0),
            );
            if self.clicked_rect(x, y, cw, ch) {
                if cur == i {
                    ok = true;
                }
                cur = i;
            }
            let (name, th, mugen) = if i == 0 {
                (
                    crate::i18n::t("select.random").to_string(),
                    [[255, 47, 208], [34, 232, 255], [255, 226, 58]],
                    false,
                )
            } else {
                let s = &self.reg.stages[i - 1];
                (s.name.clone(), s.thumb, s.mugen)
            };
            self.ui
                .gradient_v(x, y, cw, ch * 0.5, rgb(th[0]), rgb(th[1]));
            self.ui
                .gradient_v(x, y + ch * 0.5, cw, ch * 0.5, rgb(th[1]), rgb(th[2]));
            if let Some(image) = i
                .checked_sub(1)
                .and_then(|index| self.reg.stage_preview(index))
            {
                let texture = self.g.texture(&image, true);
                draw_texture_ex(
                    &texture,
                    self.ui.x(x),
                    self.ui.y(y),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(cw * self.ui.s, ch * self.ui.s)),
                        source: Some(Rect::new(
                            0.0,
                            (image.h as f32 - image.w as f32 * ch / cw) * 0.5,
                            image.w as f32,
                            image.w as f32 * ch / cw,
                        )),
                        ..Default::default()
                    },
                );
            }
            self.ui
                .rect(x, y + ch - 30.0, cw, 30.0, Color::new(0.0, 0.0, 0.0, 0.6));
            self.ui.text(
                &name.chars().take(28).collect::<String>(),
                x + 10.0,
                y + ch - 9.0,
                16.0,
                WHITE,
                Align::Left,
                true,
            );
            if mugen {
                self.ui
                    .rect(x + cw - 60.0, y + 6.0, 54.0, 16.0, hex("#22e8ff"));
                self.ui.text(
                    "MUGEN",
                    x + cw - 33.0,
                    y + 19.0,
                    12.0,
                    BLACK,
                    Align::Center,
                    true,
                );
            }
            if cur == i {
                self.ui
                    .rect_lines(x - 3.0, y - 3.0, cw + 6.0, ch + 6.0, 4.0, hex("#ffcc33"));
            }
        }
        if total_rows > VIEW_ROWS {
            let bh = VIEW_ROWS as f32 * (ch + 14.0) - 14.0;
            self.ui
                .rect(VW - 22.0, 88.0, 6.0, bh, Color::new(1.0, 1.0, 1.0, 0.1));
            let th = (bh * VIEW_ROWS as f32 / total_rows as f32).max(18.0);
            self.ui.rect(
                VW - 22.0,
                88.0 + (bh - th) * self.stage_row as f32 / (total_rows - VIEW_ROWS) as f32,
                6.0,
                th,
                hex("#ff4d6d"),
            );
        }
        if ok {
            self.sel.2 = if cur == 0 {
                (get_time() * 7.0) as usize % self.reg.stages.len()
            } else {
                cur - 1
            };
            self.go(Screen::Vs { t: 0.0, step: 0 });
            return;
        }
        if self.c.pressed(nav::BACK) {
            self.screen = Screen::Select {
                cur: [self.sel.0, self.sel.1],
                picking: 0,
                picked: [None, None],
            };
            return;
        }
        self.screen = Screen::Stages { cur };
    }
}
