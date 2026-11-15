use std::cell::RefCell;
use ume_engine::matchs::Match;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct MatchStats {
    pub best_combo: [i32; 2],
    pub damage_dealt: [f32; 2],
    pub ticks: i64,
    team_life: [f32; 2],
}

thread_local! { static STATS: RefCell<MatchStats> = RefCell::new(MatchStats::default()); }

fn side_life(m: &Match, side: usize) -> f32 {
    m.roots().filter(|f| f.side == side).map(|f| f.life.max(0.0)).sum()
}

pub fn observe(m: &Match) {
    STATS.with(|cell| {
        let mut s = cell.borrow_mut();
        if m.game_time <= 1 {
            *s = MatchStats::default();
            s.team_life = [side_life(m, 0), side_life(m, 1)];
        }
        s.ticks = m.game_time;
        for attacker in m.roots() {
            for defender in m.roots().filter(|d| d.side != attacker.side) {
                let linked = defender.move_type == 'H'
                    && defender
                        .last_hit_by
                        .is_some_and(|id| m.fighters.iter().any(|f| f.id == id && f.root == attacker.id));
                if linked && attacker.side < 2 {
                    s.best_combo[attacker.side] = s.best_combo[attacker.side].max(defender.ghv.hitcount);
                }
            }
        }
        for side in 0..2 {
            let now = side_life(m, side);
            let lost = (s.team_life[side] - now).max(0.0);
            s.damage_dealt[1 - side] += lost;
            s.team_life[side] = now;
        }
    });
}

pub fn snapshot() -> MatchStats {
    STATS.with(|cell| *cell.borrow())
}

pub fn clock_text(ticks: i64) -> String {
    let secs = (ticks.max(0) / 60) as i32;
    format!("{}:{:02}", secs / 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_formats_minutes_and_seconds() {
        assert_eq!(clock_text(0), "0:00");
        assert_eq!(clock_text(60 * 75), "1:15");
        assert_eq!(clock_text(-5), "0:00");
    }
}
