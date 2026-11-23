use ume_engine::matchs::Match;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesyncPoint {
    pub frame: i32,
    pub local: u128,
    pub remote: u128,
}

pub fn first_mismatch(local: &[(i32, u128)], remote: &[(i32, u128)]) -> Option<DesyncPoint> {
    let mut ri = 0usize;
    for &(lf, lc) in local {
        while ri < remote.len() && remote[ri].0 < lf {
            ri += 1;
        }
        if ri >= remote.len() {
            break;
        }
        if remote[ri].0 == lf && remote[ri].1 != lc {
            return Some(DesyncPoint {
                frame: lf,
                local: lc,
                remote: remote[ri].1,
            });
        }
    }
    None
}

pub fn compare_matches(a: &Match, b: &Match) -> Vec<String> {
    let mut diffs = Vec::new();
    if a.game_time != b.game_time {
        diffs.push(format!("game_time: {} vs {}", a.game_time, b.game_time));
    }
    if a.round_state != b.round_state {
        diffs.push(format!("round_state: {} vs {}", a.round_state, b.round_state));
    }
    if a.fighters.len() != b.fighters.len() {
        diffs.push(format!("fighters.len: {} vs {}", a.fighters.len(), b.fighters.len()));
        return diffs;
    }
    for (i, (fa, fb)) in a.fighters.iter().zip(b.fighters.iter()).enumerate() {
        if fa.pos.x.to_bits() != fb.pos.x.to_bits() || fa.pos.y.to_bits() != fb.pos.y.to_bits() {
            diffs.push(format!(
                "f[{i}].pos: ({:.3},{:.3}) vs ({:.3},{:.3})",
                fa.pos.x, fa.pos.y, fb.pos.x, fb.pos.y
            ));
        }
        if fa.vel.x.to_bits() != fb.vel.x.to_bits() || fa.vel.y.to_bits() != fb.vel.y.to_bits() {
            diffs.push(format!(
                "f[{i}].vel: ({:.3},{:.3}) vs ({:.3},{:.3})",
                fa.vel.x, fa.vel.y, fb.vel.x, fb.vel.y
            ));
        }
        if fa.life.to_bits() != fb.life.to_bits() {
            diffs.push(format!("f[{i}].life: {:.1} vs {:.1}", fa.life, fb.life));
        }
        if fa.power.to_bits() != fb.power.to_bits() {
            diffs.push(format!("f[{i}].power: {:.1} vs {:.1}", fa.power, fb.power));
        }
        if fa.state_no != fb.state_no || fa.state_time != fb.state_time {
            diffs.push(format!(
                "f[{i}].state: ({},{}) vs ({},{})",
                fa.state_no, fa.state_time, fb.state_no, fb.state_time
            ));
        }
        if fa.facing.to_bits() != fb.facing.to_bits() {
            diffs.push(format!("f[{i}].facing: {} vs {}", fa.facing, fb.facing));
        }
    }
    diffs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_mismatch_finds_earliest_overlapping_divergent_frame() {
        let local = [(1, 10), (2, 20), (3, 30), (4, 40)];
        let remote = [(0, 99), (2, 20), (3, 31), (4, 41)];
        let pt = first_mismatch(&local, &remote).expect("must find mismatch at frame 3");
        assert_eq!(pt, DesyncPoint { frame: 3, local: 30, remote: 31 });
    }

    #[test]
    fn first_mismatch_returns_none_when_in_sync() {
        let a = [(1, 111), (2, 222), (5, 555)];
        let b = [(1, 111), (2, 222), (5, 555)];
        assert_eq!(first_mismatch(&a, &b), None);
    }
}
