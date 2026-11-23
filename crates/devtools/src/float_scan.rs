use std::collections::BTreeMap;
use serde_json::{json, Value};
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::Match;

pub const HOVER_TICKS: i32 = 30;
pub const AIRBORNE_TICKS: i32 = 300;
pub const FROZEN_TICKS: i32 = 90;

fn scan_pair(roster: &[String], a: usize, b: usize, frames: usize, seed: u32, out: &mut BTreeMap<String, Value>) -> Result<(), String> {
    let p1 = crate::debug_fight::load_fighter(&roster[a])?;
    let p2 = crate::debug_fight::load_fighter(&roster[b])?;
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(p1, 0, 8, 1);
    m.add_player(p2, 1, 8, 2);
    m.start_round();
    m.round_state = 2;
    for f in m.fighters.iter_mut() {
        f.ctrl = true;
    }
    let lvl = 4 + (seed % 5) as i32;
    let (mut ai0, mut ai1) = (Ai::new(lvl, seed), Ai::new(lvl, seed + 1));
    let mut hover = [0i32; 2];
    let mut air = [0i32; 2];
    let mut freeze = [0i32; 2];
    for _ in 0..frames {
        let (i0, i1) = (ai0.poll(&m, 0), ai1.poll(&m, 1));
        m.tick(&[i0, i1]);
        for k in 0..2 {
            let f = &m.fighters[k];
            let frozen = f.hit_shake > 0 || f.hit_pause > 0 || m.pause.is_some() || f.bind.is_some();
            let airborne_hit = f.move_type == 'H' && f.pos.y < -1.0;
            freeze[k] = if airborne_hit && frozen { freeze[k] + 1 } else { 0 };
            let hit_air = f.move_type == 'H' && f.pos.y < -1.0 && !frozen;
            hover[k] = if hit_air && f.vel.y.abs() < 0.01 { hover[k] + 1 } else { 0 };
            air[k] = if hit_air { air[k] + 1 } else { 0 };
            if hover[k] == HOVER_TICKS || air[k] == AIRBORNE_TICKS || freeze[k] == FROZEN_TICKS {
                let kind = if hover[k] == HOVER_TICKS { "hover" } else if freeze[k] == FROZEN_TICKS { "frozen" } else { "airborne" };
                let key = format!("{}|{}|{kind}|st{}", roster[k], roster[1 - k], f.state_no);
                out.entry(key).or_insert_with(|| json!({
                    "tick": m.game_time, "state": f.state_no, "anim": f.anim_no, "y": f.pos.y,
                    "vy": f.vel.y, "yaccel": f.ghv.yaccel, "fall": f.ghv.fall, "shake": f.hit_shake, "pause": f.hit_pause, "bound": f.bind.is_some(), "kind": kind,
                }));
            }
        }
        if m.fighters.iter().any(|f| f.life <= 0.0) {
            break;
        }
    }
    Ok(())
}

pub fn run(roster: &[String], frames: usize) -> Result<Value, String> {
    let mut found = BTreeMap::new();
    let n = roster.len();
    for i in 0..n {
        for j in (0..n).filter(|&j| j != i) {
            scan_pair(roster, i, j, frames, 11 + (i * n + j) as u32, &mut found)?;
        }
    }
    Ok(json!({ "fights": n * (n - 1), "findings": found }))
}
