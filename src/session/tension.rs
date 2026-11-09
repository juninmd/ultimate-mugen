use crate::audio::Audio;
use ume_engine::matchs::Match;

pub(super) fn update_with<F: FnMut(&str)>(m: &Match, mut play: F, tick: &mut u32) {
    if m.round_state != 2 {
        *tick = 0;
        return;
    }
    let mut min_ratio = 1.0f32;
    for f in m.roots() {
        if f.life > 0.0 {
            let r = f.life / f.life_max.max(1.0);
            if r < min_ratio {
                min_ratio = r;
            }
        }
    }
    if min_ratio <= 0.25 {
        let period = if min_ratio <= 0.12 { 46 } else { 68 };
        if *tick % period == 0 {
            play("danger");
        }
        *tick = tick.wrapping_add(1);
    } else {
        *tick = 0;
    }
}

pub(super) fn update(m: &Match, audio: &mut Audio, tick: &mut u32) {
    update_with(m, |k| audio.sfx(k), tick);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ume_engine::common::{common_def, flat_stage};

    #[test]
    fn tension_heartbeat_triggers_only_on_critical_life() {
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(common_def(), 0, 0, 1);
        m.add_player(common_def(), 1, 0, 2);
        let mut played = Vec::<&'static str>::new();
        let mut tick = 0u32;

        // Inactive round: tension stays reset
        m.round_state = 1;
        update_with(
            &m,
            |k| played.push(if k == "danger" { "danger" } else { "other" }),
            &mut tick,
        );
        assert_eq!(tick, 0);
        assert!(played.is_empty());

        // Active round, full health: no heartbeat tension
        m.round_state = 2;
        m.fighters[0].life = 1000.0;
        m.fighters[0].life_max = 1000.0;
        m.fighters[1].life = 1000.0;
        m.fighters[1].life_max = 1000.0;
        update_with(
            &m,
            |k| played.push(if k == "danger" { "danger" } else { "other" }),
            &mut tick,
        );
        assert_eq!(tick, 0);
        assert!(played.is_empty());

        // Low health (< 25%): tension tick advances and triggers heartbeat
        m.fighters[0].life = 200.0;
        update_with(
            &m,
            |k| played.push(if k == "danger" { "danger" } else { "other" }),
            &mut tick,
        );
        assert_eq!(tick, 1);
        assert_eq!(played, vec!["danger"]);

        // Mortal peril (< 12%): faster pulse period
        m.fighters[0].life = 100.0;
        tick = 45;
        update_with(
            &m,
            |k| played.push(if k == "danger" { "danger" } else { "other" }),
            &mut tick,
        );
        assert_eq!(tick, 46);
    }
}
