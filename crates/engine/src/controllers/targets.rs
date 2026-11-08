use crate::chardef::Controller;
use crate::matchs::Match;
pub(super) fn life_add(m: &mut Match, t: usize, v: f32, kill: bool, absolute: bool) {
    let f = &mut m.fighters[t];
    let dv = if !absolute && v < 0.0 { v / f.defence_mul } else { v };
    let mut nl = f.life + dv.trunc();
    if !kill && nl < 1.0 && f.life > 0.0 { nl = 1.0; }
    f.life = nl.clamp(0.0, f.life_max);
}

pub(super) fn target_indices(m: &Match, i: usize, sc: &Controller) -> [Option<usize>; 8] {
    let id = m.num_opt(i, sc, "id", 0).map(|v| v as i32).unwrap_or(-1);
    let mut out = [None; 8];
    let mut count = 0;
    for t in &m.fighters[i].targets {
        if id < 0 || t.1 == id {
            if let Some(idx) = m.idx(t.0) {
                if count < out.len() {
                    out[count] = Some(idx);
                    count += 1;
                }
            }
        }
    }
    out
}