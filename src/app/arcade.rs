//! Arcade and Story progression: boss ladder, secret villain unlocks, cutscenes and progression.
use crate::app::{App, Screen, Team};
use crate::controls::nav;
use crate::gfx::*;
use crate::ui::*;

mod route;

impl App {
    pub(crate) fn playable(&self) -> Vec<usize> {
        (0..self.reg.chars.len())
            .filter(|&i| {
                if !self.route_allows(i) {
                    false
                } else if !self.reg.chars[i].boss {
                    true
                } else {
                    let id = self.reg.chars[i].id.strip_prefix("umf:").unwrap_or(&self.reg.chars[i].id);
                    self.boss_progression.is_unlocked(id)
                }
            })
            .collect()
    }

    pub(crate) fn player_char_id(&self) -> String {
        self.reg.chars.get(self.sel.0).map(|c| {
            c.id.strip_prefix("umf:").unwrap_or(&c.id).to_string()
        }).unwrap_or_default()
    }

    pub(crate) fn boss_idx(&self) -> Option<usize> {
        let player = self.player_char_id();
        let target = self.boss_progression.current_boss_id(&player);
        (0..self.reg.chars.len())
            .find(|&i| {
                let id = self.reg.chars[i].id.strip_prefix("umf:").unwrap_or(&self.reg.chars[i].id);
                id == target && !self.reg.chars[i].broken.get()
            })
            .or_else(|| {
                (0..self.reg.chars.len()).find(|&i| {
                    let id = self.reg.chars[i].id.strip_prefix("umf:").unwrap_or(&self.reg.chars[i].id);
                    self.reg.chars[i].boss && !id.starts_with("npc_") && !self.reg.chars[i].broken.get()
                })
            })
    }

    pub(crate) fn boss_stage_idx(&self, boss_char_idx: usize) -> usize {
        let id = self.reg.chars[boss_char_idx].id.strip_prefix("umf:").unwrap_or(&self.reg.chars[boss_char_idx].id);
        let target_stage = crate::boss::BossProgression::boss_stage(id);
        self.reg.stages.iter().position(|s| s.id.contains(target_stage)).unwrap_or(1)
    }

    pub(crate) fn arcade_return_to_select(&mut self) -> Screen {
        self.replaying = None;
        self.session = None;
        self.team = None;
        self.team_pick.clear();
        self.arcade = None;
        let mut cur_p1 = self.sel.0;
        if let Some(oboro) = (0..self.reg.chars.len()).find(|&i| {
            let id = self.reg.chars[i].id.strip_prefix("umf:").unwrap_or(&self.reg.chars[i].id);
            id == "oboro" && self.boss_progression.is_unlocked(id)
        }) {
            if self.boss_progression.clears == 1 {
                cur_p1 = oboro;
                self.sel.0 = oboro;
            }
        }
        self.audio.sfx("select");
        Screen::Select {
            cur: [cur_p1, self.sel.1],
            picking: 0,
            picked: [None, None],
        }
    }

    pub(crate) fn arcade_start(&mut self, me: usize, mate: Option<usize>) {
        let n = self.reg.chars.len();
        let boss = self.boss_idx();
        let mut ladder: Vec<usize> = (0..n)
            .filter(|&i| {
                i != me
                    && Some(i) != mate
                    && !self.reg.chars[i].broken.get()
                    && Some(i) != boss
                    && (!self.reg.chars[i].boss || self.boss_progression.is_unlocked(
                        self.reg.chars[i].id.strip_prefix("umf:").unwrap_or(&self.reg.chars[i].id)
                    ))
            })
            .collect();
        let seed = (get_time() * 1000.0) as usize;
        let ll = ladder.len().max(1);
        ladder.rotate_left(seed % ll);
        ladder.sort_by_key(|&i| self.reg.chars[i].mugen as u8);
        ladder.truncate(8);
        if ladder.is_empty() {
            ladder.push(me);
        }
        let is_oboro = boss.is_some_and(|b| {
            let id = self.reg.chars[b].id.strip_prefix("umf:").unwrap_or(&self.reg.chars[b].id);
            id == "oboro"
        });
        let has_boss = boss
            .filter(|&b| b != me && Some(b) != mate)
            .map(|b| {
                ladder.push(b);
                if is_oboro {
                    ladder.push(b);
                    ladder.push(b);
                }
            })
            .is_some();
        let routed = self.route_ladder(me, mate, &ladder, boss);
        let has_boss = has_boss && routed.is_none();
        if let Some(route_ladder) = routed {
            ladder = route_ladder;
        }
        self.sel = (me, ladder[0], self.sel.2);
        self.team = mate.map(|m| Team::new(vec![me, m], vec![ladder[0]], true));
        self.team_pal = if mate.is_some() {
            vec![self.side_pal(0), self.side_pal(1)]
        } else {
            vec![]
        };
        self.arcade = Some((ladder, 0));
        self.survival = None;
        self.ending_shown = false;
        self.screen = if has_boss {
            Screen::Cutscene {
                kind: 0,
                page: 0,
                t: 0.0,
            }
        } else {
            Screen::Stages { cur: 1 }
        };
    }

    pub(crate) fn draw_boss_toast(&mut self, dt: f32) {
        self.boss_progression.tick_toast(dt);
        if let Some(toast) = &self.boss_progression.toast {
            let (tw, th) = (760.0, 72.0);
            let tx = (screen_width() - tw) / 2.0;
            let ty = 32.0;
            self.ui.rect(tx, ty, tw, th, Color::new(0.04, 0.02, 0.08, 0.94));
            self.ui.rect_lines(tx, ty, tw, th, 2.0, hex("#ffcc33"));
            self.ui.text_bold(&toast.title, tx + tw / 2.0, ty + 28.0, 22.0, hex("#ffcc33"), BLACK, Align::Center);
            self.ui.text(&toast.subtitle, tx + tw / 2.0, ty + 54.0, 16.0, WHITE, Align::Center, false);
            if self.c.pressed(nav::OK | nav::START | nav::UP | nav::DOWN | nav::LEFT | nav::RIGHT) {
                self.boss_progression.toast = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn boss_stage_mapping_matches_character_home() {
        assert_eq!(crate::boss::BossProgression::boss_stage("oboro"), "spirit");
        assert_eq!(crate::boss::BossProgression::boss_stage("garan"), "volcano");
        assert_eq!(crate::boss::BossProgression::boss_stage("valdric"), "neon");
    }

    #[test]
    fn secret_boss_unlock_and_rotation_cycle() {
        let mut prog = crate::boss::BossProgression::default();
        assert!(!prog.is_unlocked("oboro"));
        assert_eq!(prog.current_boss_id("kaito"), "oboro");

        prog.on_clear("kaito", "oboro");
        assert!(prog.is_unlocked("oboro"));
        assert_eq!(prog.clears, 1);
        assert_eq!(prog.current_boss_id("kaito"), "garan");

        prog.on_clear("kaito", "garan");
        assert_eq!(prog.clears, 2);
        assert_eq!(prog.current_boss_id("kaito"), "valdric");

        prog.on_clear("kaito", "valdric");
        assert_eq!(prog.clears, 3);
        assert_eq!(prog.current_boss_id("kaito"), "oboro");
    }

    #[test]
    fn player_choosing_active_boss_avoids_mirror_match() {
        let mut prog = crate::boss::BossProgression::default();
        prog.clears = 1;
        assert_eq!(prog.current_boss_id("kaito"), "garan");
        assert_eq!(prog.current_boss_id("garan"), "valdric");

        prog.clears = 2;
        assert_eq!(prog.current_boss_id("kaito"), "valdric");
        assert_eq!(prog.current_boss_id("valdric"), "garan");
    }
}
