use crate::chardef::Controller;
use crate::matchs::Match;

const MAX_ROUND_TIME: i32 = 100000;

fn score_owner(m: &Match, i: usize) -> usize {
    m.idx(m.fighters[i].root).unwrap_or(i)
}

pub fn target_add(m: &mut Match, i: usize, sc: &Controller) {
    if !sc.has("id", 0) {
        return;
    }
    let player_id = m.num(i, sc, "id", 0.0, 0) as u32;
    let Some(t) = m.idx(player_id) else { return };
    let me = &m.fighters[i];
    let target = &m.fighters[t];
    if t == i || target.destroyed || target.root == me.root || me.targets.iter().any(|x| x.0 == player_id) {
        return;
    }
    let default_tag = me.hitdef.as_ref().map_or(0, |h| h.id);
    let tag = if sc.has("targetid", 0) { m.num(i, sc, "targetid", 0.0, 0) as i32 } else { default_tag };
    m.fighters[i].targets.push((player_id, tag));
}

pub fn target_score_add(m: &mut Match, i: usize, sc: &Controller) {
    let value = m.num(i, sc, "value", 0.0, 0) as f32;
    for t in crate::controllers::target_indices(m, i, sc) {
        let owner = score_owner(m, t);
        m.fighters[owner].ikemen.score += value;
    }
}

pub fn score_add(m: &mut Match, i: usize, sc: &Controller) {
    let value = m.num(i, sc, "value", 0.0, 0) as f32;
    let owner = score_owner(m, i);
    m.fighters[owner].ikemen.score += value;
}

pub fn target_red_life_add(m: &mut Match, i: usize, sc: &Controller) {
    let value = m.num(i, sc, "value", 0.0, 0) as f32;
    let absolute = m.num(i, sc, "absolute", 0.0, 0) != 0.0;
    for t in crate::controllers::target_indices(m, i, sc) {
        let f = &mut m.fighters[t];
        let delta = if value < 0.0 && !absolute { value / f.defence_mul } else { value };
        f.ikemen.red_life = (f.ikemen.red_life + delta).clamp(f.life.max(0.0), f.life_max);
    }
}

pub fn target_dizzy_points_add(m: &mut Match, i: usize, sc: &Controller) {
    let value = m.num(i, sc, "value", 0.0, 0) as f32;
    for t in crate::controllers::target_indices(m, i, sc) {
        let s = &mut m.fighters[t].ikemen;
        s.dizzy_points = (s.dizzy_points + value).clamp(0.0, s.dizzy_max);
    }
}

pub fn target_guard_points_add(m: &mut Match, i: usize, sc: &Controller) {
    let value = m.num(i, sc, "value", 0.0, 0) as f32;
    for t in crate::controllers::target_indices(m, i, sc) {
        let s = &mut m.fighters[t].ikemen;
        s.guard_points = (s.guard_points + value).clamp(0.0, s.guard_max);
    }
}

pub fn round_time(m: &mut Match, i: usize, sc: &Controller, add: bool) {
    if !sc.has("value", 0) {
        return;
    }
    let value = m.num(i, sc, "value", 0.0, 0) as i32;
    let base = if add { m.timer } else { 0 };
    m.timer = base.saturating_add(value).clamp(0, MAX_ROUND_TIME);
}
