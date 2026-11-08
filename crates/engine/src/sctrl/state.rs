use crate::chardef::Controller;
use crate::matchs::Match;

const MAX_RUN_STATE_DEPTH: u8 = 8;

pub fn run_state(m: &mut Match, i: usize, sc: &Controller) {
    if !sc.has("value", 0) || m.fighters[i].run_state_depth >= MAX_RUN_STATE_DEPTH {
        return;
    }
    let no = m.num(i, sc, "value", 0.0, 0) as i32;
    let (id, owner, in_pause) = {
        let f = &mut m.fighters[i];
        f.run_state_depth += 1;
        (f.id, f.st_def.clone(), f.hit_pause > 0 || f.hit_shake > 0)
    };
    m.run_state(i, &owner, no, in_pause);
    if let Some(k) = m.idx(id) {
        m.fighters[k].run_state_depth -= 1;
    }
}

pub fn height(m: &mut Match, i: usize, sc: &Controller) {
    let l = m.fighters[i].lscl;
    let top = m.num(i, sc, "value", m.fighters[i].c("size.height", 60.0) as f64, 0) as f32 * l;
    let bottom = if sc.has("value", 1) { m.num(i, sc, "value", 0.0, 1) as f32 * l } else { 0.0 };
    m.fighters[i].height_override = Some((top, bottom));
}
