use crate::chardef::Controller;
use crate::matchs::Match;

fn target_index(m: &Match, i: usize, sc: &Controller) -> Option<usize> {
    if !sc.has("id", 0) { return Some(i); }
    let id = m.num(i, sc, "id", -1.0, 0);
    if id < 0.0 { return Some(i); }
    m.idx(id as u32)
}

pub fn modify_player(m: &mut Match, i: usize, sc: &Controller) {
    let Some(t) = target_index(m, i, sc) else { return };
    let n = |m: &Match, k: &str| m.num(i, sc, k, 0.0, 0);
    if sc.has("lifemax", 0) {
        let v = n(m, "lifemax") as f32;
        let f = &mut m.fighters[t];
        f.life_max = v.max(1.0);
        f.life = f.life.min(f.life_max);
    }
    if sc.has("life", 0) {
        let v = n(m, "life") as f32;
        let f = &mut m.fighters[t];
        f.life = v.clamp(0.0, f.life_max);
    }
    let root = m.idx(m.fighters[t].root).unwrap_or(t);
    if sc.has("powermax", 0) {
        let v = n(m, "powermax") as f32;
        let f = &mut m.fighters[root];
        f.power_max = v.max(0.0);
        f.power = f.power.min(f.power_max);
    }
    if sc.has("power", 0) {
        let v = n(m, "power") as f32;
        let f = &mut m.fighters[root];
        f.power = v.clamp(0.0, f.power_max);
    }
    if sc.has("ctrl", 0) {
        let v = n(m, "ctrl") != 0.0;
        m.fighters[t].ctrl = v;
    }
    if sc.has("ailevel", 0) {
        let v = n(m, "ailevel").clamp(0.0, 8.0) as i32;
        m.fighters[t].ai_level = v;
    }
    if sc.has("redlife", 0) {
        let v = n(m, "redlife") as f32;
        let f = &mut m.fighters[t];
        f.ikemen.red_life = v.clamp(f.life.max(0.0), f.life_max);
    }
    if sc.has("dizzypoints", 0) {
        let v = n(m, "dizzypoints") as f32;
        let s = &mut m.fighters[t].ikemen;
        s.dizzy_points = v.clamp(0.0, s.dizzy_max);
    }
    if sc.has("guardpoints", 0) {
        let v = n(m, "guardpoints") as f32;
        let s = &mut m.fighters[t].ikemen;
        s.guard_points = v.clamp(0.0, s.guard_max);
    }
}
