use super::types::{HitSpec, ProjectileSpec};

const GLUE_KNOCKBACK: [f32; 2] = [-0.6, 0.0];
const GLUE_MIN_STUN: i32 = 18;

/// Moves with three or more hit frames are strings (rushes, flurries): every hit but the last keeps the target in front of the
/// attacker (tiny push-back, enough hit-stun) so the whole string connects; the finisher keeps its own launch / knock-back.
pub(super) fn glue(hits: &[HitSpec]) -> Vec<HitSpec> {
    let last = hits.len().saturating_sub(1);
    hits.iter()
        .enumerate()
        .map(|(i, h)| {
            let mut h = h.clone();
            if hits.len() >= 3 && i < last && h.launch != Some(true) {
                h.knockback.get_or_insert(GLUE_KNOCKBACK);
                h.hitstun = Some(h.hitstun.unwrap_or(0).max(GLUE_MIN_STUN));
            }
            h
        })
        .collect()
}

/// Piercing beams (4+ hits) must not shove the target out of themselves after the first tick.
pub(super) fn projectile_knockback(p: &ProjectileSpec) -> Option<[f32; 2]> {
    if p.knockback.is_none() && p.hits.unwrap_or(1) >= 4 && p.launch != Some(true) {
        Some(GLUE_KNOCKBACK)
    } else {
        p.knockback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hit(extra: serde_json::Value) -> HitSpec {
        let mut v = json!({"at": 1, "damage": 20});
        v.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn strings_keep_the_target_close_until_the_finisher() {
        let out = glue(&[
            hit(json!({})),
            hit(json!({})),
            hit(json!({"launch": true, "knockback": [-3.0, -8.0]})),
        ]);
        assert_eq!(out[0].knockback, Some(GLUE_KNOCKBACK));
        assert!(out[0].hitstun.unwrap() >= GLUE_MIN_STUN);
        assert_eq!(out[2].knockback, Some([-3.0, -8.0]), "finisher untouched");
    }

    #[test]
    fn explicit_knockback_and_short_moves_are_left_alone() {
        let out = glue(&[
            hit(json!({"knockback": [-2.0, 0.0]})),
            hit(json!({})),
            hit(json!({})),
        ]);
        assert_eq!(out[0].knockback, Some([-2.0, 0.0]));
        let pair = glue(&[hit(json!({})), hit(json!({}))]);
        assert_eq!(pair[0].knockback, None);
    }
}
