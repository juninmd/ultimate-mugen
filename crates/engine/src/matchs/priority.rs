use super::Match;

pub(super) fn keep(m: &Match, pending: &[(usize, usize, (f32, f32))], count: usize) -> [bool; 8] {
    let mut keep = [true; 8];
    let mut dodge = [false; 8];
    for i in 0..count {
        if let Some(hd) = m.fighters[pending[i].0].hitdef.as_ref() {
            dodge[i] = hd.priority_type == 2;
        }
    }
    for i in 0..count {
        for j in i + 1..count {
            let (ai, di, _) = pending[i];
            let (aj, dj, _) = pending[j];
            if ai != dj || aj != di {
                continue;
            }
            let ci = m.fighters[ai].hitdef.as_ref().map_or(0, |h| h.priority_type);
            let cj = m.fighters[aj].hitdef.as_ref().map_or(0, |h| h.priority_type);
            if dodge[i] || dodge[j] || (ci == 1 && cj == 1) {
                keep[i] = false;
                keep[j] = false;
            } else if ci == 1 {
                keep[i] = false;
            } else if cj == 1 {
                keep[j] = false;
            } else {
                let pi = m.fighters[ai].hitdef.as_ref().map_or(0, |h| h.priority);
                let pj = m.fighters[aj].hitdef.as_ref().map_or(0, |h| h.priority);
                match pi.cmp(&pj) {
                    std::cmp::Ordering::Greater => keep[j] = false,
                    std::cmp::Ordering::Less => keep[i] = false,
                    std::cmp::Ordering::Equal => {}
                }
            }
        }
    }
    keep
}
