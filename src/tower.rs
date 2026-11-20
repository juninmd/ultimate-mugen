pub mod daily;
pub mod modifiers;

pub use daily::{daily_seed, load_daily_record, save_daily_record, tower_theme, utc_date_key};
pub use modifiers::Modifier;

pub const TOWER_FLOORS: usize = 6;
const FLOOR_BONUS: i64 = 1200;
const BOSS_CLEAR_BONUS: i64 = 5000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TowerPhase {
    Ladder,
    Defeated,
    Victory,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloorDef {
    pub floor: usize,
    pub foe_idx: usize,
    pub stage_idx: usize,
    pub modifier: Modifier,
    pub ai_level: i32,
    pub is_boss: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TowerRun {
    pub date_key: String,
    pub seed: u64,
    pub floors: Vec<FloorDef>,
    pub current: usize,
    pub score: i64,
    pub wins: usize,
    pub phase: TowerPhase,
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0.max(1);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn pick(&mut self, len: usize) -> usize {
        if len <= 1 { 0 } else { (self.next() as usize) % len }
    }
}

impl TowerRun {
    pub fn generate(
        date_key: String,
        playable: &[usize],
        stage_count: usize,
        boss_idx: Option<usize>,
        player_idx: usize,
        base_ai: i32,
    ) -> Self {
        let seed = daily_seed(&date_key);
        let mut rng = Rng(seed ^ 0x9e37_79b9_7f4a_7c15);
        let mut pool: Vec<usize> = playable
            .iter()
            .copied()
            .filter(|&i| i != player_idx && Some(i) != boss_idx)
            .collect();
        if pool.is_empty() {
            pool.push(player_idx);
        }
        for i in (1..pool.len()).rev() {
            let j = rng.pick(i + 1);
            pool.swap(i, j);
        }
        let stages = stage_count.max(1);
        let mut floors = Vec::with_capacity(TOWER_FLOORS);
        for floor in 0..TOWER_FLOORS {
            let is_boss = floor + 1 == TOWER_FLOORS;
            let foe_idx = if is_boss {
                boss_idx.filter(|&b| b != player_idx).unwrap_or(pool[floor % pool.len()])
            } else {
                pool[floor % pool.len()]
            };
            let stage_idx = rng.pick(stages);
            let modifier = if is_boss {
                Modifier::BossOverdrive
            } else {
                Modifier::for_floor(rng.next())
            };
            let ai_level = (base_ai - 1 + floor as i32 + if is_boss { 2 } else { 0 }).clamp(1, 8);
            floors.push(FloorDef { floor, foe_idx, stage_idx, modifier, ai_level, is_boss });
        }
        Self { date_key, seed, floors, current: 0, score: 0, wins: 0, phase: TowerPhase::Ladder }
    }

    pub fn current_floor(&self) -> Option<&FloorDef> {
        self.floors.get(self.current)
    }

    pub fn won_floor(&mut self, fight_score: i64) -> bool {
        let bonus = FLOOR_BONUS * (self.current as i64 + 1);
        self.score += fight_score.max(0) + bonus;
        self.wins += 1;
        if self.current + 1 >= self.floors.len() {
            self.score += BOSS_CLEAR_BONUS;
            self.phase = TowerPhase::Victory;
            true
        } else {
            self.current += 1;
            self.phase = TowerPhase::Ladder;
            false
        }
    }

    pub fn lost_floor(&mut self, fight_score: i64) {
        self.score += fight_score.max(0);
        self.phase = TowerPhase::Defeated;
    }

    pub fn retry_floor(&mut self) {
        self.phase = TowerPhase::Ladder;
    }
}

#[cfg(test)]
#[path = "tower/tests.rs"]
mod tests;
