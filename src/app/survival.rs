//! Survival mode: endless opponents, escalating AI, partial life recovery and a score.
use crate::app::{App, Mode, Screen, Team};
use crate::controls::nav;
use crate::gfx::*;
use crate::settings::SURVIVAL_RECOVERY;
use crate::ui::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Survival {
    pub score: i64,
    /// 1-based round number (the opponent you are about to face)
    pub round: usize,
    /// life ratio carried into the next fight
    pub life: f32,
}

impl Default for Survival { fn default() -> Self { Survival { score: 0, round: 1, life: 1.0 } } }

impl Survival {
    /// Difficulty climbs every two fights, starting from the player's setting.
    pub fn ai_level(&self, base: i32) -> i32 { (1 + (self.round as i32 - 1) / 2 + base / 4).clamp(1, 8) }

    /// Points for a won fight: the engine's own ScoreTotal plus a clear bonus per round.
    pub fn payout(&self, fight_score: i64) -> i64 { fight_score.max(0) + 1_000 * self.round as i64 }

    pub fn recover(&mut self, percent: u32) {
        self.life = (self.life + percent as f32 / 100.0).clamp(0.0, 1.0);
    }

    /// Advance to the next opponent after a win.
    pub fn advance(&mut self, fight_score: i64, recovery: u32) {
        self.score += self.payout(fight_score);
        self.round += 1;
        self.recover(recovery);
    }
}

impl App {
    pub(crate) fn survival_recovery(&self) -> u32 { SURVIVAL_RECOVERY[self.settings.survival_recovery.min(SURVIVAL_RECOVERY.len() - 1)] }

    /// Pick an opponent other than `me`: playable, loaded and not the hidden boss.
    pub(crate) fn survival_opponent(&self, me: usize, avoid: Option<usize>) -> usize {
        let pool: Vec<usize> = (0..self.reg.chars.len()).filter(|&i| i != me && !self.reg.chars[i].broken.get() && !self.reg.chars[i].boss && Some(i) != avoid).collect();
        if pool.is_empty() { return if me == 0 && self.reg.chars.len() > 1 { 1 } else { me }; }
        let seed = (get_time() * 977.0) as usize + self.survival.map(|s| s.round).unwrap_or(1) * 31;
        pool[seed % pool.len()]
    }

    /// Called when the player confirms a character in the Survival select screen.
    pub(crate) fn survival_begin(&mut self, me: usize, mate: Option<usize>) {
        self.survival = Some(Survival::default());
        self.sel.0 = me;
        self.sel.1 = self.survival_opponent(me, mate);
        self.arcade = None;
        self.team = mate.map(|m| Team::new(vec![me, m], vec![self.sel.1], true));
        self.team_pal = if mate.is_some() { vec![self.side_pal(0), self.side_pal(1)] } else { vec![] };
        self.team_pick.clear();
        self.ending_shown = false;
        self.screen = Screen::Stages { cur: 1 };
    }

    /// Score earned by side 0 in the fight that just ended (deterministic: read from the simulation).
    fn survival_fight_score(&self) -> i64 {
        self.session.as_ref().map(|s| s.m.fighters.iter().filter(|f| !f.is_helper && f.side == 0).map(|f| f.score_total as i64).sum::<i64>()).unwrap_or(0)
    }

    /// Won a fight: bank the score, heal a little and queue the next opponent.
    pub(crate) fn survival_win(&mut self) {
        let gained = self.survival_fight_score();
        let recovery = self.survival_recovery();
        let Some(s) = self.survival.as_mut() else { return };
        s.advance(gained, recovery);
        let round = s.round;
        let me = self.sel.0;
        self.sel.1 = self.survival_opponent(me, Some(self.sel.1));
        if let Some(t) = self.team.as_mut() { t.members[1] = vec![self.sel.1]; }
        self.sel.2 = (self.sel.2 + 3 + round) % self.reg.stages.len().max(1);
        self.session = None;
        self.audio.stop_music();
        self.screen = Screen::Vs { t: 0.0, step: 0 };
    }

    /// Lost a fight: bank the score, persist a new best and show the game-over screen.
    pub(crate) fn survival_loss(&mut self) {
        let gained = self.survival_fight_score();
        let Some(s) = self.survival.as_mut() else { return };
        s.score += gained.max(0);
        let (score, rounds) = (s.score, s.round);
        if score as u64 > self.settings.survival_best { self.settings.survival_best = score as u64; self.settings.save(); }
        self.session = None;
        self.screen = Screen::SurvivalOver { score, rounds };
    }

    pub(crate) fn survival_over_screen(&mut self) {
        let Screen::SurvivalOver { score, rounds } = self.screen else { return };
        self.ui.background();
        self.ui.motes(50, 1.4);
        self.ui.text_bold(crate::i18n::t("survival.over"), VW / 2.0, 210.0, 62.0, hex("#ff4d6d"), BLACK, Align::Center);
        let best = self.settings.survival_best.max(0) as i64;
        let record = score >= best && score > 0;
        self.ui.text(&crate::i18n::tf("survival.points", &[("{score}", &score.to_string())]), VW / 2.0, 300.0, 40.0, hex("#ffcc33"), Align::Center, true);
        self.ui.text(&crate::i18n::tf("survival.wins", &[("{count}", &rounds.to_string())]), VW / 2.0, 346.0, 22.0, WHITE, Align::Center, false);
        self.ui.text(&if record { crate::i18n::t("survival.new_record").to_string() } else { crate::i18n::tf("survival.record", &[("{best}", &best.to_string())]) }, VW / 2.0, 386.0, 20.0, if record { hex("#6dff9e") } else { hex("#a49cc0") }, Align::Center, true);
        let can_replay = self.last_fight.is_some();
        let mut items: Vec<&str> = vec![crate::i18n::t("survival.retry")];
        if can_replay { items.push(crate::i18n::t("survival.watch")); }
        items.extend([crate::i18n::t("results.characters"), crate::i18n::t("results.menu")]);
        let mut cur = 0usize;
        let mut ok = self.c.pressed(nav::OK | nav::START);
        for (i, it) in items.iter().enumerate() {
            let (bx, by) = (VW / 2.0 - 310.0 + i as f32 * 160.0, 460.0);
            if self.clicked_rect(bx, by, 150.0, 50.0) { cur = i; ok = true; }
            self.ui.rect(bx, by, 150.0, 50.0, if i == 0 { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.08) });
            self.ui.text(it, bx + 75.0, by + 33.0, 19.0, WHITE, Align::Center, true);
        }
        if self.c.pressed(nav::RIGHT) { cur = (cur + 1) % items.len(); }
        if self.c.pressed(nav::LEFT) { cur = (cur + items.len() - 1) % items.len(); }
        if ok {
            self.audio.stop_music();
            match items[cur] {
                x if x == crate::i18n::t("survival.retry") => {
                    self.replaying = None;
                    self.survival = Some(Survival::default());
                    self.sel.1 = self.survival_opponent(self.sel.0, None);
                    if let Some(t) = self.team.as_mut() { t.members[1] = vec![self.sel.1]; }
                    self.screen = Screen::Vs { t: 0.0, step: 0 };
                }
                x if x == crate::i18n::t("survival.watch") => { self.screen = Screen::Results { winner: 1, cur: 0 }; }
                x if x == crate::i18n::t("results.characters") => { self.replaying = None; self.session = None; self.survival = None; self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }; }
                _ => { self.replaying = None; self.session = None; self.survival = None; self.screen = Screen::Menu { cur: 0 }; }
            }
            return;
        }
        if self.c.pressed(nav::BACK) || is_key_pressed(KeyCode::Escape) { self.survival = None; self.screen = Screen::Menu { cur: 0 }; return; }
        self.screen = Screen::SurvivalOver { score, rounds };
    }

    /// Top banner during a Survival fight (round, score and best).
    pub(crate) fn survival_hud(&self) {
        let Some(s) = self.survival else { return };
        let best = self.settings.survival_best;
        self.ui.rect(VW / 2.0 - 210.0, 6.0, 420.0, 44.0, Color::new(0.02, 0.01, 0.06, 0.6));
        self.ui.text(&crate::i18n::tf("survival.hud", &[("{round}", &s.round.to_string()), ("{score}", &s.score.to_string())]), VW / 2.0, 28.0, 17.0, hex("#ffcc33"), Align::Center, true);
        self.ui.text(&crate::i18n::tf("survival.record", &[("{best}", &best.to_string())]), VW / 2.0, 43.0, 12.0, hex("#a49cc0"), Align::Center, false);
    }

    /// Life ratios the recorded fight started with (Survival carries life between fights).
    pub(crate) fn survival_life(&self) -> [f32; 2] { self.survival.map(|s| [s.life, 1.0]).unwrap_or([1.0, 1.0]) }

    pub(crate) fn survival_is_playing(&self) -> bool { self.mode == Mode::Survival && self.replaying.is_none() }
}

#[cfg(test)]
mod tests {
    use super::Survival;

    #[test]
    fn difficulty_climbs_every_two_rounds() {
        let mut s = Survival::default();
        let start = s.ai_level(4);
        s.round = 3;
        assert!(s.ai_level(4) > start);
        s.round = 40;
        assert_eq!(s.ai_level(8), 8, "the AI level is capped at 8");
    }

    #[test]
    fn score_banks_the_fight_and_the_round_bonus() {
        let mut s = Survival::default();
        s.advance(250, 35);
        assert_eq!(s.score, 1_250, "fight score plus the round-1 clear bonus");
        assert_eq!(s.round, 2);
        s.advance(1_000, 35);
        assert_eq!(s.score, 1_250 + 2_000 + 1_000);
    }

    #[test]
    fn recovery_is_partial_and_clamped() {
        let mut s = Survival::default();
        s.life = 0.2;
        s.recover(35);
        assert!((s.life - 0.55).abs() < 1e-6);
        s.recover(100);
        assert_eq!(s.life, 1.0, "life never goes above the bar");
        s.life = 0.0;
        s.recover(0);
        assert_eq!(s.life, 0.0, "0% recovery keeps the bar where it was");
    }

    #[test]
    fn a_negative_fight_score_never_subtracts() {
        let s = Survival::default();
        assert!(s.payout(-500) >= 0);
    }
}
