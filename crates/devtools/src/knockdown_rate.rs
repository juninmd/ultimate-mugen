use ume_engine::input::InputSource;
use serde_json::{json, Value};
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::{Event, KnockdownRule, KnockdownScope, Match};

const BUCKETS: [f32; 5] = [4.0, 8.0, 12.0, 20.0, f32::MAX];

#[derive(Default)]
struct Tally {
    hits: u32,
    knockdowns: u32,
    pushed: u32,
    by_bucket: [[u32; 2]; BUCKETS.len()],
}

pub fn parse_rule(text: &str, threshold: f32) -> Result<KnockdownRule, String> {
    let scope = match text {
        "off" => KnockdownScope::Off,
        "native" => KnockdownScope::Native,
        "all" => KnockdownScope::All,
        other => return Err(format!("unknown rule {other}")),
    };
    Ok(KnockdownRule { scope, threshold_percent: threshold })
}

fn fight(roster: &[String], a: usize, b: usize, frames: usize, rule: KnockdownRule, seed: u32, tally: &mut Tally) -> Result<(), String> {
    let p1 = crate::debug_fight::load_fighter(&roster[a])?;
    let p2 = crate::debug_fight::load_fighter(&roster[b])?;
    let mut m = Match::new(common_def(), None, flat_stage());
    m.knockdown = rule;
    m.add_player(p1, 0, 8, 1);
    m.add_player(p2, 1, 8, 2);
    m.start_round();
    m.round_state = 2;
    for f in m.fighters.iter_mut() {
        f.ctrl = true;
    }
    let mut ai0 = Ai::new(8, seed);
    let mut ai1 = Ai::new(8, seed + 1);
    for _ in 0..frames {
        let grounded: Vec<bool> = m.fighters.iter().map(|f| f.state_type != 'A' && f.state_type != 'L' && f.air_height() >= -1.0 && !f.ghv.fall).collect();
        let i0 = ai0.poll(&m, 0);
        let i1 = ai1.poll(&m, 1);
        m.tick(&[i0, i1]);
        for ev in m.events.drain(..).collect::<Vec<_>>() {
            let Event::Hit { damage, guarded: false, defender, .. } = ev else { continue };
            if damage <= 0.0 {
                continue;
            }
            if !grounded.get(defender).copied().unwrap_or(false) {
                continue;
            }
            let d = &m.fighters[defender];
            let down = d.ghv.fall || d.state_type == 'A' || d.state_type == 'L' || d.air_height() < -1.0;
            let pct = damage / d.life_max.max(1.0) * 100.0;
            tally.hits += 1;
            tally.knockdowns += down as u32;
            tally.pushed += (!down && d.ghv.xvel.abs() > 0.0) as u32;
            let slot = BUCKETS.iter().position(|&hi| pct < hi).unwrap_or(BUCKETS.len() - 1);
            tally.by_bucket[slot][down as usize] += 1;
        }
        if m.fighters.iter().any(|f| f.life <= 1.0) {
            for f in m.fighters.iter_mut() {
                f.life = f.life_max;
            }
        }
    }
    Ok(())
}

pub fn run(roster: &[String], frames: usize, rule: KnockdownRule) -> Result<Value, String> {
    let mut tally = Tally::default();
    let n = roster.len();
    for i in 0..n {
        for step in [1usize, 5] {
            fight(roster, i, (i + step) % n, frames, rule, 7 + i as u32, &mut tally)?;
        }
    }
    let rate = |k: u32, h: u32| if h == 0 { 0.0 } else { (k as f64 / h as f64 * 1000.0).round() / 10.0 };
    let buckets: Vec<Value> = BUCKETS
        .iter()
        .zip(tally.by_bucket.iter())
        .map(|(hi, [stand, down])| json!({ "below_pct": if *hi == f32::MAX { Value::Null } else { json!(hi) }, "hits": stand + down, "knockdown_pct": rate(*down, stand + down) }))
        .collect();
    Ok(json!({ "scope": format!("{:?}", rule.scope), "threshold": rule.threshold_percent, "hits": tally.hits, "knockdowns": tally.knockdowns, "knockdown_pct": rate(tally.knockdowns, tally.hits), "buckets": buckets }))
}
