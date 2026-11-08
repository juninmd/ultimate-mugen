use crate::chardef::Controller;
use crate::fighter::HitOverride;
use crate::matchs::Match;

pub(super) fn apply(m: &mut Match, i: usize, sc: &Controller) {
    let n = |m: &Match, k: &str, d: f64| m.num(i, sc, k, d, 0);
    let attr = sc.raw("attr").unwrap_or("SCA,AA,AT,AP").to_string();
    let slot = (n(m, "slot", 0.0) as usize).min(7);
    let (state_no, time, force_air) = (n(m, "stateno", -1.0) as i32, n(m, "time", 1.0) as i32, n(m, "forceair", 0.0) != 0.0);
    let (keep, hits, threshold, chip) = (n(m, "keepstate", 0.0) != 0.0, n(m, "armorhits", 1.0) as i32, n(m, "armorthreshold", 0.0) as f32, n(m, "armorchip", 35.0) as f32);
    m.fighters[i].hit_overrides[slot] = if state_no < 0 && !keep { None } else { Some(HitOverride { attr, state_no, time, force_air, keep, hits, threshold, chip }) };
}
