use super::Match;

pub(super) fn keep(m: &Match, pending: &[(usize, usize, (f32, f32))], count: usize) -> [bool; 8] {
    let mut keep = [true; 8];
    let mut dodge = [false; 8];
    for i in 0..count {
        if let Some(hd) = m.fighters[pending[i].0].hitdef.as_ref() {
            if hd.priority_type != 0 {
                keep[i] = false;
                dodge[i] = hd.priority_type == 2;
            }
        }
    }
    for i in 0..count {
        for j in i + 1..count {
            let (ai, di, _) = pending[i];
            let (aj, dj, _) = pending[j];
            if ai != dj || aj != di {
                continue;
            }
            if dodge[i] {
                keep[j] = false;
            }
            if dodge[j] {
                keep[i] = false;
            }
            if !keep[i] || !keep[j] {
                continue;
            }
            let pi = m.fighters[ai].hitdef.as_ref().map_or(0, |h| h.priority);
            let pj = m.fighters[aj].hitdef.as_ref().map_or(0, |h| h.priority);
            match pi.cmp(&pj) {
                std::cmp::Ordering::Greater => keep[j] = false,
                std::cmp::Ordering::Less => keep[i] = false,
                std::cmp::Ordering::Equal => {}
            }
        }
    }
    keep
}
