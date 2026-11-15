use super::menu_icons::ICON_OF;
use super::*;

impl App {
    pub(super) fn menu(&mut self) {
        let Screen::Menu { cur } = &mut self.screen else {
            return;
        };
        let mut cur_v = *cur;
        if self.c.pressed(nav::DOWN) {
            cur_v = (cur_v + 1) % MENU.len();
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::UP) {
            cur_v = (cur_v + MENU.len() - 1) % MENU.len();
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::LEFT) {
            self.ai_level = (self.ai_level - 1).max(1);
        }
        if self.c.pressed(nav::RIGHT) {
            self.ai_level = (self.ai_level + 1).min(8);
        }
        let mut chosen = if self.c.pressed(nav::OK | nav::START) {
            Some(cur_v)
        } else {
            None
        };
        self.menu_focus.settle(cur_v, get_frame_time());
        self.menu_backdrop(cur_v);
        let ui = &self.ui;
        let accent = crate::menu_art::accent_of(cur_v);
        self.menu_chrome(accent);
        let n = MENU.len();
        let step = 456.0 / n as f32;
        for (i, (key, _)) in MENU.iter().enumerate() {
            let label = self.menu_text(key);
            let k_in = ((self.screen_t - 0.05 - i as f32 * 0.04) / 0.28).clamp(0.0, 1.0);
            let k_in = 1.0 - (1.0 - k_in).powi(3);
            let (x, y, w, h) = (
                40.0 - (1.0 - k_in) * 120.0,
                196.0 + i as f32 * step,
                520.0,
                step - 4.0,
            );
            if self.hover_rect(x, y, w, h) && (mouse_delta_position().length() > 0.0) {
                cur_v = i;
            }
            if self.clicked_rect(x, y, w, h) {
                cur_v = i;
                chosen = Some(i);
            }
            let lit = self.menu_focus.level(i);
            let off = 22.0 * lit;
            let pulse = 0.8 + 0.2 * (ui.t * 5.0).sin();
            if lit < 0.995 {
                ui.quad(
                    [vec2(x + off + 18.0, y), vec2(x + off + w - 80.0, y), vec2(x + off + w - 98.0, y + h), vec2(x + off, y + h)],
                    Color::new(1.0, 1.0, 1.0, 0.07 * k_in * (1.0 - lit)),
                );
            }
            if lit > 0.005 {
                let reach = menu_focus::mix(w - 80.0, w, lit);
                ui.quad(
                    [vec2(x + off + 18.0, y), vec2(x + off + reach, y), vec2(x + off + reach - 18.0, y + h), vec2(x + off, y + h)],
                    Color::new(accent.r * pulse, accent.g * pulse, accent.b * pulse, 0.92 * k_in * lit),
                );
                ui.quad(
                    [vec2(x + off - 14.0, y), vec2(x + off - 4.0, y), vec2(x + off - 18.0, y + h), vec2(x + off - 28.0, y + h)],
                    Color::new(1.0, 1.0, 1.0, k_in * lit),
                );
            }
            let (light, ink) = (Color::new(1.0, 1.0, 1.0, 0.82 * k_in), hex("#12051f"));
            let label_color = Color::new(
                menu_focus::mix(light.r, ink.r, lit),
                menu_focus::mix(light.g, ink.g, lit),
                menu_focus::mix(light.b, ink.b, lit),
                menu_focus::mix(light.a, k_in, lit),
            );
            self.menu_icon(ICON_OF[i], x + off + 38.0, y + h * 0.5, label_color);
            ui.text(&label, x + off + 70.0, y + h * 0.72, menu_focus::mix(22.0, 24.0, lit), label_color, Align::Left, true);
        }
        self.menu_info(cur_v, accent);
        self.screen = Screen::Menu { cur: cur_v };
        if self.c.pressed(nav::BACK) {
            self.screen = Screen::Title;
        }
        if let Some(i) = chosen {
            let mode = [Mode::Story, Mode::Arcade, Mode::Survival, Mode::Cpu, Mode::Training];
            match i {
                0..=4 => {
                    self.set_mode(mode[i]);
                    self.go(Screen::Select {
                        cur: [self.sel.0, self.sel.1],
                        picking: 0,
                        picked: [None, None],
                    });
                }
                5 => self.go(Screen::Mugen { view: 0, page: 0 }),
                6 => self.go(Screen::Market),
                7 => {
                    self.settings_back = None;
                    self.go(Screen::Settings {
                        tab: 0,
                        cur: 0,
                        from_fight: false,
                    })
                }
                8 => {
                    self.refresh_replays();
                    self.go(Screen::Replays { cur: 0 })
                }
                _ => self.quit = true,
            }
        }
    }
}
