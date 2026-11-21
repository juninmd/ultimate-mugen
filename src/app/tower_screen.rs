use crate::app::{App, Screen};
use crate::controls::nav;
use crate::gfx::*;
use crate::i18n::tr3;
use crate::tower::{daily, load_daily_record, tower_theme, TowerPhase};
use crate::ui::*;

impl App {
    pub(crate) fn tower_screen(&mut self, dt: f32) {
        let Screen::Tower { t } = self.screen else { return };
        let Some(run) = self.tower.clone() else {
            self.screen = Screen::Menu { cur: 11 };
            return;
        };
        let (theme_title, rgb) = tower_theme(run.seed);
        let accent = Color::new(rgb[0], rgb[1], rgb[2], 1.0);
        let rec = load_daily_record(&run.date_key);
        let reveal = (t * 3.0).min(1.0);
        let pulse = 0.72 + 0.28 * (t * 4.5).sin().abs();

        crate::screen_art::ScreenArt::Tower.background(&self.ui);
        self.ui.motes(36, 1.1);
        self.ui.adapted_header(
            tr3("TORRE DIÁRIA", "DAILY TOWER", "TORRE DIARIA"),
            &format!("{} · {}", run.date_key, daily::countdown_label()),
            VW,
        );

        let (lx, ly, lw, lh) = (56.0, 108.0, 460.0, 540.0);
        crate::menu_art::glass(&self.ui, lx, ly, lw, lh, reveal);
        self.ui.rect(lx, ly, lw * reveal, 4.0, accent);

        self.ui.text(
            tr3("DESAFIO DE HOJE", "TODAY'S CHALLENGE", "DESAFÍO DE HOY"),
            lx + 24.0,
            ly + 40.0,
            15.0,
            accent,
            Align::Left,
            true,
        );
        self.ui.text_bold(theme_title, lx + 24.0, ly + 82.0, 32.0, WHITE, BLACK, Align::Left);
        let reset_line = format!(
            "{}: {}",
            tr3("PRÓXIMA TORRE EM", "NEXT TOWER IN", "PRÓXIMA TORRE EN"),
            daily::countdown_label()
        );
        self.ui.text(&reset_line, lx + 24.0, ly + 116.0, 16.0, hex("#c9c2e2"), Align::Left, false);

        if rec.cleared {
            self.ui.rect(lx + 24.0, ly + 132.0, 210.0, 28.0, Color::new(0.2, 0.75, 0.42, 0.85));
            self.ui.text(
                tr3("CONCLUÍDA HOJE", "CLEARED TODAY", "COMPLETADA HOY"),
                lx + 129.0,
                ly + 152.0,
                14.0,
                WHITE,
                Align::Center,
                true,
            );
        }

        let hero_name = self.reg.chars.get(self.sel.0).map(|c| c.name.as_str()).unwrap_or("P1");
        self.ui.text(
            &format!("{}: {}", tr3("LUTADOR", "FIGHTER", "LUCHADOR"), hero_name.to_uppercase()),
            lx + 24.0,
            ly + 206.0,
            18.0,
            WHITE,
            Align::Left,
            true,
        );
        self.ui.text(
            &format!("{}: {}  ·  {}: {}", tr3("PONTOS", "SCORE", "PUNTOS"), run.score, tr3("RECORDE", "BEST", "RÉCORD"), rec.best_score),
            lx + 24.0,
            ly + 238.0,
            17.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );

        if let Some(cur_floor) = run.current_floor() {
            let mrgb = cur_floor.modifier.color();
            let mc = Color::new(mrgb[0], mrgb[1], mrgb[2], 1.0);
            self.ui.rect(lx + 24.0, ly + 272.0, lw - 48.0, 110.0, Color::new(0.08, 0.05, 0.16, 0.85));
            self.ui.rect(lx + 24.0, ly + 272.0, 5.0, 110.0, mc);
            self.ui.text(
                &format!(
                    "{} {}/{} · {}",
                    tr3("ANDAR", "FLOOR", "PISO"),
                    cur_floor.floor + 1,
                    run.floors.len(),
                    cur_floor.modifier.label()
                ),
                lx + 42.0,
                ly + 306.0,
                18.0,
                mc,
                Align::Left,
                true,
            );
            self.ui.text(cur_floor.modifier.desc(), lx + 42.0, ly + 338.0, 15.0, WHITE, Align::Left, false);
            let foe_name = self.reg.chars.get(cur_floor.foe_idx).map(|c| c.name.as_str()).unwrap_or("CPU");
            self.ui.text(
                &format!("VS {} · CPU {}/8", foe_name.to_uppercase(), cur_floor.ai_level),
                lx + 42.0,
                ly + 366.0,
                15.0,
                hex("#c9c2e2"),
                Align::Left,
                true,
            );
        }

        let (status_title, action_label, status_col) = match run.phase {
            TowerPhase::Ladder => (
                tr3("SUBA A TORRE", "CLIMB THE TOWER", "SUBE LA TORRE"),
                tr3("INICIAR LUTA [ENTER]", "START FIGHT [ENTER]", "INICIAR PELEA [ENTER]"),
                accent,
            ),
            TowerPhase::Defeated => (
                tr3("DERROTADO NO ANDAR", "DEFEATED ON FLOOR", "DERROTADO EN EL PISO"),
                tr3("TENTAR NOVAMENTE [ENTER]", "RETRY FLOOR [ENTER]", "REINTENTAR [ENTER]"),
                hex("#ff5c77"),
            ),
            TowerPhase::Victory => (
                tr3("TORRE CONQUISTADA!", "TOWER CONQUERED!", "¡TORRE CONQUISTADA!"),
                tr3("VOLTAR AO MENU [ENTER]", "BACK TO MENU [ENTER]", "VOLVER AL MENÚ [ENTER]"),
                hex("#ffcc33"),
            ),
        };
        self.ui.text_bold(status_title, lx + 24.0, ly + 440.0, 24.0, status_col, BLACK, Align::Left);
        let btn = (lx + 24.0, ly + 468.0, lw - 48.0, 48.0);
        let btn_hot = self.hover_rect(btn.0, btn.1, btn.2, btn.3);
        self.ui.rect(btn.0, btn.1, btn.2, btn.3, Color::new(status_col.r, status_col.g, status_col.b, if btn_hot { 0.95 } else { 0.8 * pulse }));
        self.ui.text_bold(action_label, btn.0 + btn.2 * 0.5, btn.1 + 31.0, 18.0, hex("#12051f"), WHITE, Align::Center);

        let (rx, ry, rw, rh) = (548.0, 108.0, 676.0, 540.0);
        crate::menu_art::glass(&self.ui, rx, ry, rw, rh, reveal);
        let count = run.floors.len().max(1);
        let row_h = (rh - 36.0) / count as f32;
        for (idx, floor) in run.floors.iter().enumerate() {
            let visual_slot = count - 1 - idx;
            let y = ry + 18.0 + visual_slot as f32 * row_h;
            let cleared = idx < run.current || run.phase == TowerPhase::Victory;
            let active = idx == run.current && run.phase != TowerPhase::Victory;
            let mrgb = floor.modifier.color();
            let mc = Color::new(mrgb[0], mrgb[1], mrgb[2], 1.0);
            let bg = if active {
                Color::new(accent.r * 0.35, accent.g * 0.35, accent.b * 0.35, 0.9)
            } else if cleared {
                Color::new(0.12, 0.32, 0.22, 0.55)
            } else {
                Color::new(1.0, 1.0, 1.0, 0.06)
            };
            self.ui.rect(rx + 18.0, y + 4.0, rw - 36.0, row_h - 10.0, bg);
            self.ui.rect(rx + 18.0, y + 4.0, 6.0, row_h - 10.0, if floor.is_boss { hex("#ffcc33") } else { mc });

            let tag = if floor.is_boss {
                tr3("CHEFE", "BOSS", "JEFE").to_string()
            } else {
                format!("#{}", idx + 1)
            };
            let foe = self.reg.chars.get(floor.foe_idx).map(|c| c.name.as_str()).unwrap_or("CPU");
            let stage = self.reg.stages.get(floor.stage_idx).map(|s| s.name.as_str()).unwrap_or("");
            self.ui.text(&tag, rx + 36.0, y + 36.0, 20.0, if floor.is_boss { hex("#ffcc33") } else { WHITE }, Align::Left, true);
            self.ui.text(&foe.to_uppercase(), rx + 124.0, y + 34.0, 22.0, if cleared { hex("#8fe3a8") } else { WHITE }, Align::Left, true);
            self.ui.text(stage, rx + 124.0, y + 58.0, 14.0, hex("#a49cc0"), Align::Left, false);

            self.ui.text(floor.modifier.label(), rx + rw - 34.0, y + 34.0, 15.0, mc, Align::Right, true);
            let badge = if cleared {
                tr3("CONCLUÍDO", "CLEARED", "SUPERADO")
            } else if active {
                tr3("PRÓXIMO >", "NEXT >", "SIGUIENTE >")
            } else {
                tr3("BLOQUEADO", "LOCKED", "BLOQUEADO")
            };
            self.ui.text(badge, rx + rw - 34.0, y + 58.0, 14.0, if active { accent } else { hex("#a49cc0") }, Align::Right, active);
        }

        let back = self.clicked_rect(56.0, 668.0, 150.0, 40.0);
        let back_hot = self.hover_rect(56.0, 668.0, 150.0, 40.0);
        self.ui.adapted_button(56.0, 668.0, 150.0, 40.0, tr3("Voltar", "Back", "Volver"), back_hot);

        if back || self.c.pressed(nav::BACK) || is_key_pressed(KeyCode::Escape) {
            self.tower_quit();
            return;
        }
        if (self.c.pressed(nav::OK | nav::START) && t > 0.25) || self.clicked_rect(btn.0, btn.1, btn.2, btn.3) {
            self.audio.sfx("confirm");
            self.tower_confirm();
            return;
        }
        self.screen = Screen::Tower { t: t + dt };
    }
}
