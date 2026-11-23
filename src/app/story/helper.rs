use crate::app::{App, Team};
use crate::gfx::*;
use crate::story::{text, helper, prefight, HelperPick, StoryRun};
use crate::ui::*;

impl App {
    pub(crate) fn story_helper_candidates(&self, run: &StoryRun) -> Vec<usize> {
        let hero = self.sel.0;
        let player = self.player_char_id();
        let boss = run.stage_boss_id(&player);
        (0..self.reg.chars.len())
            .filter(|&i| {
                let entry = &self.reg.chars[i];
                let id = entry.id.strip_prefix("umf:").unwrap_or(&entry.id);
                i != hero
                    && id != boss
                    && !entry.mugen
                    && !entry.boss
                    && !entry.broken.get()
                    && !id.starts_with("npc_")
            })
            .collect()
    }

    pub(super) fn story_assemble_team(&mut self, run: &StoryRun, hero: usize, foes: Vec<usize>) {
        let helper = helper::active(run).and_then(|id| self.story_char(id));
        let squad: Vec<usize> = std::iter::once(hero).chain(helper).collect();
        self.team_pal = squad.iter().enumerate().map(|(k, _)| if k == 0 { self.side_pal(0) } else { 1 }).collect();
        self.team = Some(Team::new(squad, foes, true));
    }

    pub(crate) fn story_prefight_line(&self, run: &StoryRun) -> String {
        let hero = self.reg.chars.get(self.sel.0).map_or("", |e| e.name.as_str());
        let player = self.player_char_id();
        let boss = self.story_char(run.stage_boss_id(&player)).map_or("", |i| self.reg.chars[i].name.as_str());
        let (speaker, text) = prefight::line(run.prefight);
        format!("{}: {}", prefight::speaker_name(speaker, hero, boss), prefight::fill(text, hero, boss))
    }

    pub(super) fn story_boss_sub(&self, run: &StoryRun) -> String {
        let title = text::stage_title(run.stage).to_string();
        let ally = helper::active(run)
            .and_then(|id| self.story_char(id))
            .map(|i| self.reg.chars[i].name.as_str());
        match ally {
            Some(name) => format!("{title} · {}", helper::joins(name)),
            None => title,
        }
    }

    pub(crate) fn story_helper_input(&mut self, run: &StoryRun, confirmed: bool) {
        let candidates = self.story_helper_candidates(run);
        let mut cursor = run.helper_cursor.min(candidates.len());
        let mut moved = false;
        if self.c.pressed(crate::controls::nav::LEFT) || self.clicked_rect(0.0, 440.0, VW * 0.3, 120.0) {
            cursor = helper::step(cursor, candidates.len(), false);
            moved = true;
        } else if self.c.pressed(crate::controls::nav::RIGHT) || self.clicked_rect(VW * 0.7, 440.0, VW * 0.3, 120.0) {
            cursor = helper::step(cursor, candidates.len(), true);
            moved = true;
        }
        let accept = confirmed || self.clicked_rect(VW * 0.3, 440.0, VW * 0.4, 120.0);
        let Some(run) = self.story.as_mut() else { return };
        run.helper_cursor = cursor;
        if moved {
            self.audio.sfx("select");
        } else if accept {
            run.helper = match cursor.checked_sub(1).and_then(|k| candidates.get(k)) {
                Some(&i) => {
                    let id = self.reg.chars[i].id.clone();
                    HelperPick::Chosen(id.strip_prefix("umf:").unwrap_or(&id).to_string())
                }
                None => HelperPick::Declined,
            };
            self.audio.sfx("confirm");
        }
    }

    pub(crate) fn draw_story_helper_pick(&self, run: &StoryRun, accent: Color) {
        let candidates = self.story_helper_candidates(run);
        let cursor = run.helper_cursor.min(candidates.len());
        let picked = cursor.checked_sub(1).and_then(|k| candidates.get(k)).copied();
        let label = match picked {
            Some(i) => self.reg.chars[i].name.to_uppercase(),
            None => helper::none_label().to_string(),
        };
        if let Some(profile) = picked.and_then(|i| crate::profile::of(&self.reg.chars[i])) {
            let nationality = format!(
                "{}: {}",
                crate::profile::nationality_label(),
                profile.nationality.pick(crate::profile::lang())
            )
            .to_uppercase();
            let flag_x = (VW - self.ui.measure(&nationality, 13.0, true)) / 2.0 - 32.0;
            self.ui.text(&nationality, VW / 2.0, 556.0, 13.0, WHITE, Align::Center, true);
            self.ui.flag(&profile.country, flag_x, 542.0, 24.0, 1.0);
        }
        self.ui.text(helper::hint(), VW / 2.0, 462.0, 16.0, WHITE, Align::Center, false);
        self.ui.text("<", VW * 0.22, 510.0, 34.0, accent, Align::Center, true);
        self.ui.text(">", VW * 0.78, 510.0, 34.0, accent, Align::Center, true);
        self.ui.text(&label, VW / 2.0, 510.0, 30.0, WHITE, Align::Center, true);
    }
}
