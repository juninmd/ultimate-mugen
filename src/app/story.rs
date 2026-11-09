#[path = "story/advance.rs"]
mod advance;

use crate::app::{App, Mode, Screen, Team};
use crate::gfx::*;
use crate::session::Session;
use crate::story::{text, Next, Phase, StoryRun};
use crate::ui::*;

pub(super) const BOSS_LIFE_FACTOR: f32 = 1.5;
pub(super) const BOSS_ATTACK_FACTOR: f32 = 1.1;
pub(super) const FOE_START_LANES: [f32; 4] = [-10.0, 6.0, -3.0, 8.0];

fn story_scroll(sess: &mut Session, def: &crate::story::StageDef, fight: usize) {
    let Some(strip) = crate::stages::story_strip(def.arena) else { return };
    let center = crate::story::fight_center(def, fight);
    let span = crate::story::strip_span(def);
    if let crate::stages::StageArt::Illustrated { image, span: art_span, .. } = &mut sess.stage.art {
        *image = strip;
        *art_span = span;
    } else {
        return;
    }
    let stage = &mut sess.m.stage;
    stage.bound_left = center - crate::story::FIGHT_HALF_RANGE;
    stage.bound_right = center + crate::story::FIGHT_HALF_RANGE;
    stage.start_x = center;
    stage.p1x = center + sess.stage.info.p1x;
    stage.p2x = center + sess.stage.info.p2x;
    sess.m.lanes.target_x = center;
    sess.m.lanes.bound_span = crate::story::FIGHT_HALF_RANGE;
    sess.m.camera.x = center;
    for fighter in sess.m.fighters.iter_mut().filter(|f| !f.is_helper) {
        fighter.pos.x += center;
    }
}

impl App {
    pub(super) fn story_char(&self, id: &str) -> Option<usize> {
        (0..self.reg.chars.len()).find(|&i| {
            let entry = &self.reg.chars[i];
            entry.id.strip_prefix("umf:").unwrap_or(&entry.id) == id && !entry.broken.get()
        })
    }

    pub(super) fn story_foes(&self, run: &StoryRun) -> Vec<usize> {
        let player = self.player_char_id();
        run.foes(&player).into_iter().filter_map(|id| self.story_char(id)).collect()
    }

    pub(crate) fn story_begin(&mut self, me: usize) {
        self.story = Some(StoryRun::default());
        self.survival = None;
        self.arcade = None;
        self.sel.0 = me;
        self.ending_shown = false;
        self.story_prepare_fight();
        self.screen = Screen::Story { t: 0.0 };
    }

    pub(crate) fn story_prepare_fight(&mut self) {
        let Some(run) = self.story.clone() else { return };
        self.story_prefetch_stage();
        let foes = self.story_foes(&run);
        let Some(&first) = foes.first() else { return };
        let me = self.sel.0;
        let arena = run.stage_def().arena;
        crate::stages::want_story_strip(arena);
        self.sel.1 = first;
        self.sel.2 = self.reg.stages.iter().position(|s| s.id.contains(arena)).unwrap_or(self.sel.2);
        self.team = Some(Team::new(vec![me], foes, true));
        self.team_pal = vec![self.side_pal(0)];
        self.session = None;
    }

    fn story_fight_score(&self) -> i64 {
        self.session
            .as_ref()
            .map(|s| {
                s.m.fighters
                    .iter()
                    .filter(|f| !f.is_helper && f.side == 0)
                    .map(|f| f.score_total as i64)
                    .sum()
            })
            .unwrap_or(0)
    }

    fn story_remaining_life(&self) -> f32 {
        self.session
            .as_ref()
            .and_then(|s| s.m.fighters.iter().find(|f| !f.is_helper && f.side == 0))
            .map(|f| if f.life_max > 0.0 { f.life / f.life_max } else { 1.0 })
            .unwrap_or(1.0)
    }

    pub(crate) fn story_win(&mut self) {
        let gained = self.story_fight_score();
        let remaining = self.story_remaining_life();
        let Some(run) = self.story.as_mut() else { return };
        let next = run.won_fight(gained, remaining);
        match next {
            Next::Wave => {
                if self.story_advance_in_place() {
                    self.screen = Screen::Fight;
                } else {
                    self.audio.stop_music();
                    self.story_prepare_fight();
                    self.screen = Screen::Vs { t: 1.8, step: 0 };
                }
            }
            Next::StageCleared | Next::Finished => {
                self.audio.stop_music();
                self.session = None;
                self.screen = Screen::Story { t: 0.0 };
            }
        }
    }

    pub(crate) fn story_loss(&mut self) {
        let gained = self.story_fight_score();
        if let Some(run) = self.story.as_mut() {
            run.lost_fight(gained);
        }
        self.audio.stop_music();
        self.session = None;
        self.screen = Screen::Story { t: 0.0 };
    }

    pub(crate) fn story_level(&self) -> Option<i32> {
        if self.mode != Mode::Story {
            return None;
        }
        self.story.as_ref().map(|run| run.ai_level(self.ai_level))
    }

    pub(crate) fn story_is_playing(&self) -> bool {
        self.mode == Mode::Story && self.story.is_some() && self.replaying.is_none()
    }

    pub(crate) fn story_tune(&self, sess: &mut Session) {
        let Some(run) = self.story.as_ref().filter(|_| self.story_is_playing()) else { return };
        sess.m.rounds_to_win = 1;
        let accent = run.stage_def().accent;
        let def = run.stage_def();
        sess.m.lanes = ume_engine::lane::LaneRules { active: true, far: def.lane_far, near: def.lane_near, show_foe_hud: run.is_boss_fight(), target_x: 0.0, bound_span: 0.0 };
        sess.team = None;
        story_scroll(sess, def, run.fight_index());
        for (slot, fighter) in sess.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 1).enumerate() {
            fighter.lane = FOE_START_LANES[slot % FOE_START_LANES.len()].clamp(def.lane_far, def.lane_near);
        }
        let hero_life_max = sess
            .m
            .fighters
            .iter()
            .find(|f| !f.is_helper && f.side == 0)
            .map(|f| f.life_max)
            .unwrap_or(1000.0);
        if !run.is_boss_fight() {
            for fighter in sess.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 1) {
                let ratio = crate::story::villain_life_ratio(&fighter.def.id);
                fighter.life_max = (hero_life_max * ratio).max(1.0);
                fighter.life = fighter.life_max;
            }
            let label = text::wave_label(run.wave, run.stage_def().waves.len(), false);
            sess.vfx.banner(&text::stage_label(run.stage), &format!("{label} · {}", text::go()), accent);
            return;
        }
        for fighter in sess.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 1) {
            fighter.life_max *= BOSS_LIFE_FACTOR;
            fighter.life = fighter.life_max;
            fighter.attack_mul *= BOSS_ATTACK_FACTOR;
        }
        sess.vfx.banner(text::boss_banner(), &text::stage_title(run.stage).to_string(), accent);
    }

    pub(crate) fn story_hud(&self) {
        let Some(run) = self.story.as_ref() else { return };
        let line = text::hud(
            run.stage,
            run.wave,
            run.stage_def().waves.len(),
            run.is_boss_fight(),
            run.score,
        );
        self.ui.rect(VW / 2.0 - 230.0, 652.0, 460.0, 44.0, Color::new(0.02, 0.01, 0.06, 0.6));
        self.ui.text(&line, VW / 2.0, 674.0, 17.0, hex("#ffcc33"), Align::Center, true);
        self.ui.text(
            &text::credits_line(run.credits),
            VW / 2.0,
            690.0,
            12.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
    }

    pub(crate) fn story_snapshot(&self) -> Option<(usize, usize, u8, i64, String)> {
        let run = self.story.as_ref()?;
        Some((run.stage, run.wave, run.credits, run.score, format!("{:?}", run.phase)))
    }

    pub(crate) fn story_jump_to_stage(&mut self, stage: usize) {
        let Some(run) = self.story.as_mut() else { return };
        run.stage = stage.min(crate::story::STAGES.len() - 1);
        self.story_prepare_fight();
    }

    pub(crate) fn story_life(&self) -> [f32; 2] {
        self.story.as_ref().map(|run| [run.life, 1.0]).unwrap_or([1.0, 1.0])
    }

    pub(crate) fn story_quit(&mut self) {
        self.story = None;
        self.session = None;
        self.team = None;
        self.audio.stop_music();
        self.screen = Screen::Menu { cur: 0 };
    }

    pub(crate) fn story_show_intro(&mut self) {
        if let Some(run) = self.story.as_mut() {
            run.phase = Phase::Intro;
        }
        self.screen = Screen::Story { t: 1.0 };
    }

    pub(crate) fn story_resume(&mut self) {
        let Some(run) = self.story.as_mut() else { return };
        match run.phase {
            Phase::Defeated if run.can_continue() => run.continue_fight(),
            Phase::Defeated | Phase::Victory => {
                self.story_quit();
                return;
            }
            Phase::Cleared => run.phase = Phase::Intro,
            Phase::Intro => {}
        }
        self.story_prepare_fight();
        self.screen = Screen::Vs { t: 0.0, step: 0 };
    }
}
