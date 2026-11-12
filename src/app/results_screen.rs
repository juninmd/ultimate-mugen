use super::*;

impl App {
    pub(super) async fn results(&mut self) {
        let Screen::Results { winner, mut cur } = self.screen else {
            return;
        };
        if let Some(s) = self.session.as_mut() {
            s.draw(&mut self.g, &self.ui);
        }
        // survival: win advances to the next opponent, loss ends the run
        if matches!(self.mode, Mode::Survival | Mode::Story) && self.replaying.is_none() {
            if winner == 0 {
                self.run_win();
                return;
            }
            self.run_loss();
            return;
        }
        // arcade progression
        if self.mode == Mode::Arcade && winner == 0 {
            let boss = self.boss_idx();
            let mut next_action = None;
            if let Some((ladder, i)) = self.arcade.as_mut() {
                *i += 1;
                if *i < ladder.len() {
                    let is_boss = Some(ladder[*i]) == boss;
                    let boss_cnt = if is_boss {
                        ladder[..=*i].iter().filter(|&&x| Some(x) == boss).count()
                    } else {
                        0
                    };
                    next_action = Some((*i, ladder[*i], is_boss, boss_cnt, false));
                } else if *i == ladder.len()
                    && !self.ending_shown
                    && boss.is_some_and(|b| ladder.last() == Some(&b))
                {
                    self.ending_shown = true;
                    next_action = Some((*i, 0, false, 0, true));
                }
            }
            if let Some((idx, next_char, is_boss, boss_cnt, is_ending)) = next_action {
                self.audio.stop_music();
                if is_ending {
                    let hero_id = self.player_char_id();
                    let boss_entry_id = boss
                        .map(|b| {
                            self.reg.chars[b]
                                .id
                                .strip_prefix("umf:")
                                .unwrap_or(&self.reg.chars[b].id)
                                .to_string()
                        })
                        .unwrap_or_default();
                    self.boss_progression.on_clear(&hero_id, &boss_entry_id);
                    self.screen = Screen::Cutscene {
                        kind: 5,
                        page: 0,
                        t: 0.0,
                    };
                    return;
                }
                self.sel.1 = next_char;
                let st_spirit = self.reg.stages.iter().position(|s| s.id.contains("spirit"));
                self.sel.2 = if boss_cnt >= 2 {
                    st_spirit.unwrap_or(0)
                } else if is_boss {
                    self.boss_stage_idx(next_char)
                } else {
                    (idx * 3 + 1) % self.reg.stages.len()
                };
                if self.coop {
                    if let Some(t) = self.team.as_mut() {
                        t.members[1] = vec![next_char];
                    }
                }
                self.screen = match (is_boss, boss_cnt) {
                    (true, 3..) => Screen::Cutscene {
                        kind: 4,
                        page: 0,
                        t: 0.0,
                    },
                    (true, 2) => Screen::Cutscene {
                        kind: 3,
                        page: 0,
                        t: 0.0,
                    },
                    (true, _) => Screen::Cutscene {
                        kind: 1,
                        page: 0,
                        t: 0.0,
                    },
                    _ => Screen::Vs { t: 0.0, step: 0 },
                };
                return;
            }
        }
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.02, 0.01, 0.06, 0.6),
        );
        self.ui.motes(70, 1.6);
        let w = if winner == 0 { self.sel.0 } else { self.sel.1 };
        let e = &self.reg.chars[w];
        self.results_panel(alpha(rgb(e.color), 1.0));
        let done =
            self.mode == Mode::Arcade && self.arcade.as_ref().is_some_and(|(l, i)| *i >= l.len());
        let title = self.results_title(winner, w, done);
        let pal = if winner == 1 && self.sel.0 == self.sel.1 {
            2
        } else {
            1
        };
        if let Some(d) = e.peek(pal) {
            let t = self.preview_t;
            self.draw_char(
                &d,
                if d.anims.contains_key(&181) { 181 } else { 0 },
                t,
                330.0,
                480.0,
                260.0,
                false,
            );
        }
        self.ui.text_bold(
            &title,
            480.0,
            300.0,
            44.0,
            hex("#ffcc33"),
            BLACK,
            Align::Left,
        );
        self.draw_results_stats();
        let can_replay = self.last_fight.is_some();
        let items: Vec<&str> = if can_replay {
            vec![
                crate::i18n::t("results.rematch"),
                crate::i18n::t("results.replay"),
                crate::i18n::t("results.characters"),
                crate::i18n::t("results.menu"),
            ]
        } else {
            vec![
                crate::i18n::t("results.rematch"),
                crate::i18n::t("results.characters"),
                crate::i18n::t("results.menu"),
            ]
        };
        let nitems = items.len();
        if self.c.pressed(nav::RIGHT) {
            cur = (cur + 1) % nitems;
        }
        if self.c.pressed(nav::LEFT) {
            cur = (cur + nitems - 1) % nitems;
        }
        let mut ok = self.c.pressed(nav::OK | nav::START);
        let (picked, clicked) = self.results_buttons(&items, cur);
        cur = picked;
        ok = ok || clicked;
        self.screen = Screen::Results { winner, cur };
        if ok {
            self.audio.stop_music();
            if done {
                self.screen = self.arcade_return_to_select();
                return;
            }
            // with a replay available the menu is Revanche / Replay / Personagens / Menu; without it Revanche / Personagens / Menu
            let choice = if can_replay {
                cur
            } else if cur >= 1 {
                cur + 1
            } else {
                0
            };
            match choice {
                0 => {
                    self.replaying = None;
                    if let Some((l, i)) = self.arcade.as_mut() {
                        *i = (*i).min(l.len().saturating_sub(1));
                    }
                    self.reset_team();
                    self.start_fight().await;
                }
                1 => {
                    self.start_replay().await;
                }
                2 => {
                    self.replaying = None;
                    self.session = None;
                    self.team = None;
                    self.team_pick.clear();
                    self.screen = Screen::Select {
                        cur: [self.sel.0, self.sel.1],
                        picking: 0,
                        picked: [None, None],
                    };
                }
                _ => {
                    self.replaying = None;
                    self.session = None;
                    self.team = None;
                    self.screen = Screen::Menu { cur: 0 };
                }
            }
        }
    }
}
