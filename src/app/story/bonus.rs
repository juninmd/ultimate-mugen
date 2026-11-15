use crate::app::{App, Screen};
use crate::session::Pilot;
use crate::story::text;
use ume_engine::fighter::V2;
use ume_engine::stage_objects::Goal;

const TIME_BONUS_PER_SECOND: i64 = 200;
const BONUS_ACCENT: [f32; 3] = [1.0, 0.82, 0.25];

impl App {
    pub(crate) fn story_bonus_active(&self) -> bool {
        self.story.as_ref().is_some_and(|run| run.bonus)
    }

    pub(crate) fn story_jump_to_boss(&mut self) {
        self.story_jump_to_wave(usize::MAX);
    }

    pub(crate) fn story_jump_to_wave(&mut self, wave: usize) {
        let Some(run) = self.story.as_mut() else {
            return;
        };
        run.wave = wave.min(run.stage_def().waves.len());
        self.story_prepare_fight();
    }

    pub(super) fn story_bonus_reward(&self) -> i64 {
        if !self.story_bonus_active() {
            return 0;
        }
        match self.session.as_ref().map(|s| s.m.objects.goal) {
            Some(Goal::Cleared { ticks_left }) => {
                (ticks_left as i64 + 59) / 60 * TIME_BONUS_PER_SECOND
            }
            _ => 0,
        }
    }

    pub(super) fn story_bonus_begin(&mut self) -> bool {
        if self.replaying.is_some() {
            return false;
        }
        let Some(run) = self
            .story
            .as_mut()
            .filter(|run| !run.bonus && run.is_boss_fight())
        else {
            return false;
        };
        let Some(sess) = self.session.as_mut() else {
            return false;
        };
        let def = run.stage_def();
        let center = crate::story::fight_center(def, run.fight_index());
        if !crate::story::objects::spawn_bonus(&mut sess.m, def, center) {
            return false;
        }
        run.bonus = true;
        let m = &mut sess.m;
        sess.over = None;
        sess.over_timer = 0.0;
        m.round_state = 2;
        m.round_time = 0;
        m.winner_side = -1;
        m.wins = [0, 0];
        m.match_over = false;
        m.won_by_ko = false;
        m.slow_mo = 0;
        m.announce = None;
        m.explods.clear();
        m.projectiles.clear();
        m.pause = None;
        m.fighters.retain(|f| !f.is_helper && f.side == 0);
        m.team_simul = false;
        m.lanes.show_foe_hud = false;
        m.lanes.target_x = center;
        m.stage.bound_left = center - crate::story::FIGHT_HALF_RANGE;
        m.stage.bound_right = center + crate::story::FIGHT_HALF_RANGE;
        sess.pilots.truncate(1);
        if let Some(Pilot::Cpu(ai)) = sess.pilots.first_mut() {
            ai.reset();
        }
        let solid = m.objects.breakables.iter().any(|b| b.solid);
        let start_x = m
            .objects
            .breakables
            .iter()
            .map(|b| b.x - b.half_width)
            .fold(center, f32::min)
            - 22.0;
        if let Some(hero) = m.fighters.first_mut() {
            hero.pos = V2 { x: start_x, y: 0.0 };
            hero.vel = V2::default();
            hero.facing = 1.0;
            hero.hit_pause = 0;
            hero.hit_shake = 0;
            hero.targets.clear();
            hero.bind = None;
            hero.hitdef_active = false;
            hero.move_type = 'I';
            hero.state_type = 'S';
            hero.combo_count = 0;
            hero.combo_target = None;
            hero.no_ctrl_ticks = 0;
            let hero_def = hero.def.clone();
            m.change_state(0, 0, hero_def);
            m.fighters[0].ctrl = true;
        }
        let ground_k = m.ground_k;
        if m.zoffset0 != 0.0 {
            m.stage.zoffset = m.zoffset0;
        }
        m.set_zoom_for_teams(1, ground_k);
        sess.vfx
            .banner(text::bonus_banner(), text::bonus_goal(solid), BONUS_ACCENT);
        self.screen = Screen::Fight;
        true
    }
}
