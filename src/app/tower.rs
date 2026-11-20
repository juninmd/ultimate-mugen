use crate::app::{App, Mode, Screen};
use crate::i18n::tr3;
use crate::session::Session;
use crate::tower::{save_daily_record, utc_date_key, TowerPhase, TowerRun};

impl App {
    pub(crate) fn tower_begin(&mut self, me: usize) {
        let playable = self.playable();
        let boss = self.boss_idx();
        let date = utc_date_key();
        let run = TowerRun::generate(
            date,
            &playable,
            self.reg.stages.len(),
            boss,
            me,
            self.ai_level,
        );
        self.sel.0 = me;
        self.story = None;
        self.survival = None;
        self.arcade = None;
        self.team = None;
        self.tower = Some(run);
        self.session = None;
        self.screen = Screen::Tower { t: 0.0 };
    }

    pub(crate) fn tower_prepare_fight(&mut self) {
        let Some(run) = self.tower.as_ref() else { return };
        let Some(floor) = run.current_floor() else { return };
        self.sel = (self.sel.0, floor.foe_idx, floor.stage_idx);
        self.team = None;
        self.session = None;
        self.screen = Screen::Vs { t: 0.0, step: 0 };
    }

    pub(crate) fn tower_level(&self) -> Option<i32> {
        if self.mode != Mode::Tower {
            return None;
        }
        self.tower.as_ref()?.current_floor().map(|f| f.ai_level)
    }

    pub(crate) fn tower_tune(&self, sess: &mut Session) {
        if self.mode != Mode::Tower || self.replaying.is_some() {
            return;
        }
        let Some(run) = self.tower.as_ref() else { return };
        let Some(floor) = run.current_floor() else { return };
        floor.modifier.apply(sess);
        let head = format!(
            "{} {}/{}",
            tr3("ANDAR", "FLOOR", "PISO"),
            floor.floor + 1,
            run.floors.len()
        );
        sess.vfx.banner(&head, floor.modifier.label(), floor.modifier.color());
    }

    fn tower_fight_score(&self) -> i64 {
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

    pub(crate) fn tower_win(&mut self) {
        let gained = self.tower_fight_score();
        if let Some(run) = self.tower.as_mut() {
            let finished = run.won_floor(gained);
            save_daily_record(&run.date_key, finished, run.score, run.wins);
        }
        self.audio.stop_music();
        self.session = None;
        self.screen = Screen::Tower { t: 0.0 };
    }

    pub(crate) fn tower_loss(&mut self) {
        let gained = self.tower_fight_score();
        if let Some(run) = self.tower.as_mut() {
            run.lost_floor(gained);
            save_daily_record(&run.date_key, false, run.score, run.wins);
        }
        self.audio.stop_music();
        self.session = None;
        self.screen = Screen::Tower { t: 0.0 };
    }

    pub(crate) fn tower_quit(&mut self) {
        self.tower = None;
        self.session = None;
        self.audio.stop_music();
        self.screen = Screen::Menu { cur: 11 };
    }

    pub(crate) fn tower_confirm(&mut self) {
        let Some(phase) = self.tower.as_ref().map(|r| r.phase) else { return };
        match phase {
            TowerPhase::Ladder => self.tower_prepare_fight(),
            TowerPhase::Defeated => {
                if let Some(run) = self.tower.as_mut() {
                    run.retry_floor();
                }
                self.tower_prepare_fight();
            }
            TowerPhase::Victory => self.tower_quit(),
        }
    }
}
