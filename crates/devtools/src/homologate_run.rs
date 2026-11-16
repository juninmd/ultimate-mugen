use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::Vfs;

const FIGHT_WAIT_TICKS: usize = 800;
const SCRIPTED_TICKS: usize = 1200;
const INPUT_HOLD_TICKS: usize = 6;
const INPUT_MASK: u64 = 0x0CFF;

fn next(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed >> 33
}

fn duel(vfs: &Vfs, def: &str) -> Result<Vec<String>, String> {
    let char_def = Arc::new(load_mugen_char(vfs, def, 1, false)?);
    let mut found = Vec::new();
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(char_def.clone(), 0, 0, 1);
    m.add_player(char_def, 1, 0, 2);
    m.start_round();
    let mut reached_fight = false;
    for _ in 0..FIGHT_WAIT_TICKS {
        m.tick(&[0, 0]);
        if m.round_state == 2 {
            reached_fight = true;
            break;
        }
    }
    if !reached_fight {
        found.push(format!(
            "round never reached the fight phase in {FIGHT_WAIT_TICKS} ticks"
        ));
        return Ok(found);
    }
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut held = [0u32; 2];
    let mut states = std::collections::BTreeSet::new();
    for t in 0..SCRIPTED_TICKS {
        if t % INPUT_HOLD_TICKS == 0 {
            held = [(next(&mut seed) & INPUT_MASK) as u32, 0];
        }
        m.tick(&held);
        states.insert(m.fighters[0].state_no);
        if !m.fighters[0].life.is_finite() || !m.fighters[0].pos.x.is_finite() {
            found.push(format!("non-finite life or position at tick {t}"));
            break;
        }
    }
    if states.len() < 2 {
        found.push("fighter never left its first state under scripted inputs".to_string());
    }
    Ok(found)
}

pub fn runtime_problems(vfs: &Vfs, def: &str) -> Vec<String> {
    match catch_unwind(AssertUnwindSafe(|| duel(vfs, def))) {
        Ok(Ok(found)) => found,
        Ok(Err(error)) => vec![format!("engine load failed: {error}")],
        Err(_) => vec!["engine panicked while loading or ticking".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_engine_load_failure_for_missing_definition() {
        let found = runtime_problems(&Vfs::new(), "c/missing.def");
        assert_eq!(found.len(), 1);
        assert!(found[0].starts_with("engine load failed"));
    }
}
