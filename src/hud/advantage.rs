use ume_engine::matchs::Match;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvantageKind {
    Hit,
    Guard,
    Knockdown,
}

#[derive(Clone, Copy, Debug)]
pub struct AdvantageResult {
    pub frames: i32,
    pub kind: AdvantageKind,
    pub attacker_side: usize,
    pub display_ticks: u32,
    #[allow(dead_code)]
    pub atk_recovery: i32,
    #[allow(dead_code)]
    pub def_recovery: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct ActiveExchange {
    pub attacker_idx: usize,
    pub defender_idx: usize,
    pub contact_tick: i64,
    pub guarded: bool,
    pub knockdown: bool,
    pub attacker_recovered: Option<i64>,
    pub defender_recovered: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct AdvantageTracker {
    pub active: Option<ActiveExchange>,
    pub result: Option<AdvantageResult>,
}

impl AdvantageTracker {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn inject_test(&mut self, frames: i32, kind: AdvantageKind, ticks: u32) {
        self.result = Some(AdvantageResult {
            frames,
            kind,
            attacker_side: 0,
            display_ticks: ticks,
            atk_recovery: 12,
            def_recovery: 12 + frames,
        });
    }

    pub fn update(&mut self, m: &Match) {
        if !m.training {
            self.active = None;
            self.result = None;
            return;
        }
        if let Some(ref mut res) = self.result {
            if res.display_ticks > 0 {
                res.display_ticks -= 1;
            } else {
                self.result = None;
            }
        }
        if m.fighters.len() < 2 {
            return;
        }

        let (f0, f1) = (&m.fighters[0], &m.fighters[1]);
        let p1_contact = f0.move_contact == 1 && f0.move_type == 'A';
        let p2_contact = f1.move_contact == 1 && f1.move_type == 'A';

        if p1_contact || p2_contact {
            let (atk_idx, def_idx) = if p1_contact { (0, 1) } else { (1, 0) };
            let fd = &m.fighters[def_idx];
            self.active = Some(ActiveExchange {
                attacker_idx: atk_idx,
                defender_idx: def_idx,
                contact_tick: m.game_time,
                guarded: fd.ghv.guarded,
                knockdown: fd.ghv.fall,
                attacker_recovered: None,
                defender_recovered: None,
            });
            return;
        }

        if let Some(mut ex) = self.active {
            if ex.attacker_idx >= m.fighters.len() || ex.defender_idx >= m.fighters.len() {
                self.active = None;
                return;
            }
            let (fa, fd) = (&m.fighters[ex.attacker_idx], &m.fighters[ex.defender_idx]);
            if ex.attacker_recovered.is_none()
                && (fa.ctrl || (fa.move_type != 'A' && fa.hit_pause == 0))
            {
                ex.attacker_recovered = Some(m.game_time);
            }
            if ex.defender_recovered.is_none()
                && (fd.ctrl || (fd.move_type != 'H' && fd.hit_shake == 0 && fd.hit_over_time <= 0))
            {
                ex.defender_recovered = Some(m.game_time);
            }
            if let (Some(atk_t), Some(def_t)) = (ex.attacker_recovered, ex.defender_recovered) {
                let atk_dur = (atk_t - ex.contact_tick).max(1) as i32;
                let def_dur = (def_t - ex.contact_tick).max(1) as i32;
                let kind = if ex.knockdown {
                    AdvantageKind::Knockdown
                } else if ex.guarded {
                    AdvantageKind::Guard
                } else {
                    AdvantageKind::Hit
                };
                self.result = Some(AdvantageResult {
                    frames: def_dur - atk_dur,
                    kind,
                    attacker_side: fa.side,
                    display_ticks: 90,
                    atk_recovery: atk_dur,
                    def_recovery: def_dur,
                });
                self.active = None;
            } else if m.game_time - ex.contact_tick > 180 {
                self.active = None;
            } else {
                self.active = Some(ex);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ume_engine::common::{common_def, flat_stage};

    #[test]
    fn tracker_lifecycle_and_advantage_calculation() {
        let mut tracker = AdvantageTracker::new();
        assert!(tracker.active.is_none());
        assert!(tracker.result.is_none());

        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(common_def(), 0, 0, 1);
        m.add_player(common_def(), 1, 0, 2);

        m.training = false;
        tracker.update(&m);
        assert!(tracker.result.is_none());

        m.training = true;
        m.game_time = 100;
        m.fighters[0].move_contact = 1;
        m.fighters[0].move_type = 'A';
        m.fighters[0].ctrl = false;
        m.fighters[1].ghv.guarded = true;
        m.fighters[1].move_type = 'H';
        m.fighters[1].hit_shake = 6;
        m.fighters[1].ctrl = false;
        tracker.update(&m);

        assert!(tracker.active.is_some());
        let ex = tracker.active.unwrap();
        assert_eq!(ex.contact_tick, 100);
        assert!(ex.guarded);

        m.game_time = 114;
        m.fighters[0].move_contact = 2;
        m.fighters[0].move_type = 'I';
        m.fighters[0].ctrl = true;
        tracker.update(&m);

        m.game_time = 118;
        m.fighters[1].move_type = 'I';
        m.fighters[1].hit_shake = 0;
        m.fighters[1].ctrl = true;
        tracker.update(&m);

        assert!(tracker.active.is_none());
        assert!(tracker.result.is_some());
        let res = tracker.result.unwrap();
        assert_eq!(res.frames, 4);
        assert_eq!(res.kind, AdvantageKind::Guard);
        assert_eq!(res.display_ticks, 90);

        tracker.update(&m);
        assert_eq!(tracker.result.unwrap().display_ticks, 89);

        m.game_time = 200;
        m.fighters[0].move_contact = 1;
        m.fighters[0].move_type = 'A';
        m.fighters[0].ctrl = false;
        m.fighters[1].ghv.guarded = true;
        m.fighters[1].move_type = 'H';
        m.fighters[1].ctrl = false;
        tracker.update(&m);
        m.game_time = 210;
        m.fighters[0].move_contact = 2;
        m.fighters[1].move_type = 'I';
        m.fighters[1].ctrl = true;
        tracker.update(&m);
        m.game_time = 216;
        m.fighters[0].move_type = 'I';
        m.fighters[0].ctrl = true;
        tracker.update(&m);
        assert_eq!(tracker.result.unwrap().frames, -6);

        m.game_time = 300;
        m.fighters[0].move_contact = 1;
        m.fighters[0].move_type = 'A';
        m.fighters[0].ctrl = false;
        m.fighters[1].ghv.fall = true;
        m.fighters[1].ghv.guarded = false;
        m.fighters[1].move_type = 'H';
        m.fighters[1].ctrl = false;
        tracker.update(&m);
        m.game_time = 312;
        m.fighters[0].move_contact = 2;
        m.fighters[0].move_type = 'I';
        m.fighters[0].ctrl = true;
        tracker.update(&m);
        m.game_time = 330;
        m.fighters[1].move_type = 'I';
        m.fighters[1].ctrl = true;
        tracker.update(&m);
        assert_eq!(tracker.result.unwrap().kind, AdvantageKind::Knockdown);
    }
}
