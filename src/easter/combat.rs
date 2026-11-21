use super::{catalog, FightTracker};
use crate::audio::Audio;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

fn roots(m: &Match) -> Option<(&Fighter, &Fighter)> {
    let p1 = m.roots().find(|f| f.side == 0)?;
    let p2 = m.roots().find(|f| f.side == 1)?;
    Some((p1, p2))
}

pub(crate) fn check(m: &Match, _stage_name: &str, tr: &mut FightTracker, audio: &mut Audio) {
    let Some((p1, p2)) = roots(m) else {
        return;
    };
    let fighters = [p1, p2];
    let now = m.game_time;

    check_round_outcomes(m, fighters, tr, audio);
    if m.round_state != 2 {
        return;
    }

    for side in 0..2 {
        let f = fighters[side];
        let opp = fighters[1 - side];
        let took_dmg = (tr.prev_life[side] - f.life).max(0.0);

        if took_dmg > 0.5 {
            tr.last_hit_tick[side] = now;
            if took_dmg >= 115.0 && (f.state_type == 'A' || f.vel.y < -4.0) {
                tr.trigger(0, true, catalog::toasty(), audio, "combo6");
            }
            if now - tr.taunt_tick[side] <= 90 && tr.taunt_tick[side] > 0 {
                tr.trigger(5, true, catalog::taunt_bodied(), audio, "counter");
            }
            if tr.corner_ticks[1 - side] >= 90 && took_dmg >= 70.0 {
                tr.trigger(17, true, catalog::corner_escape(1 - side), audio, "confirm");
            }
        }

        if f.state_no == 195 && tr.prev_state[side] != 195 {
            tr.taunt_tick[side] = now;
            if tr.taunt_tick[1 - side] > 0 && now - tr.taunt_tick[1 - side] <= 120 {
                tr.trigger(6, true, catalog::honorable_taunt(), audio, "unlock");
            }
        }

        if f.state_type == 'A' && tr.prev_type[side] != 'A' && f.move_type != 'H' {
            tr.jump_tick[side] = now;
            if tr.jump_tick[1 - side] > 0 && (now - tr.jump_tick[1 - side]).abs() <= 4 {
                tr.sync_jumps = tr.sync_jumps.saturating_add(1);
                if tr.sync_jumps >= 2 {
                    tr.trigger(7, true, catalog::sync_jump(), audio, "confirm");
                }
            }
        }

        if f.state_type == 'C' && tr.prev_type[side] != 'C' && (p1.pos.x - p2.pos.x).abs() > 110.0 {
            if now - tr.crouch_window[side] > 70 {
                tr.crouch_count[side] = 0;
            }
            tr.crouch_window[side] = now;
            tr.crouch_count[side] = tr.crouch_count[side].saturating_add(1);
            if tr.crouch_count[side] >= 4 {
                tr.crouch_count[side] = 0;
                tr.trigger(8, true, catalog::crouch_party(side), audio, "toggle");
            }
        }

        if (150..=155).contains(&f.state_no) && !(150..=155).contains(&tr.prev_state[side]) {
            tr.guard_streak[side] = tr.guard_streak[side].saturating_add(1);
            if tr.guard_streak[side] >= 6 {
                tr.trigger(12, true, catalog::guard_fortress(side), audio, "guard");
            }
        } else if f.move_type == 'H' && !(120..=155).contains(&f.state_no) {
            tr.guard_streak[side] = 0;
        }

        if f.state_no == 5300 && tr.prev_state[side] != 5300 {
            tr.trigger(13, true, catalog::guard_shatter(side), audio, "guardbreak");
        }

        if f.combo_count >= 5 && opp.state_type == 'A' {
            tr.trigger(11, true, catalog::air_juggle(side), audio, "combo4");
        }
        if f.combo_count >= 15 {
            tr.trigger(16, true, catalog::overkill_combo(), audio, "combo6");
        }

        if f.power >= f.power_max.max(1.0) && tr.prev_power[side] < f.power_max.max(1.0) {
            tr.trigger(19, true, catalog::max_meter_coin(side), audio, "meter");
        }

        if tr.prev_power[side] - f.power >= 950.0 {
            tr.super_tick[side] = now;
            if tr.super_tick[1 - side] > 0 && now - tr.super_tick[1 - side] <= 90 {
                tr.trigger(14, true, catalog::double_super_clash(), audio, "super");
            }
            if f.life_max > 0.0 && f.life / f.life_max <= 0.06 && f.alive {
                tr.trigger(2, true, catalog::magic_pixel(side), audio, "danger");
            }
        }

        if f.form > tr.prev_form[side] {
            tr.awakened[side] = true;
            if tr.awakened[0] && tr.awakened[1] {
                tr.trigger(15, false, catalog::awakening_duel(), audio, "transform");
            }
        }

        let near_edge = (f.pos.x - m.stage.bound_left).abs() < 36.0 || (m.stage.bound_right - f.pos.x).abs() < 36.0;
        if near_edge && f.alive {
            tr.corner_ticks[side] += 1;
        } else {
            tr.corner_ticks[side] = 0;
        }
    }

    if tr.last_hit_tick[0] > 0 && (tr.last_hit_tick[0] - tr.last_hit_tick[1]).abs() <= 3 && now - tr.last_hit_tick[0] <= 2 {
        tr.trigger(4, true, catalog::mutual_clash(), audio, "counter");
    }

    let both_still = p1.vel.x.abs() < 0.05
        && p2.vel.x.abs() < 0.05
        && p1.state_type == 'S'
        && p2.state_type == 'S'
        && p1.move_type == 'I'
        && p2.move_type == 'I'
        && (p1.pos.x - p2.pos.x).abs() > 75.0;
    if both_still {
        tr.still_ticks += 1;
        if tr.still_ticks == 180 {
            tr.trigger(18, true, catalog::staredown(), audio, "tick");
        }
    } else {
        tr.still_ticks = 0;
    }
}

fn check_round_outcomes(m: &Match, fighters: [&Fighter; 2], tr: &mut FightTracker, audio: &mut Audio) {
    if m.round_state < 3 {
        return;
    }
    let [p1, p2] = fighters;
    if !p1.alive && !p2.alive {
        tr.trigger(1, true, catalog::double_ko(), audio, "crowd");
        return;
    }
    for side in 0..2 {
        let f = fighters[side];
        let opp = fighters[1 - side];
        if f.alive && !opp.alive {
            if f.life >= f.life_max - 0.5 && f.life_max > 0.0 {
                tr.trigger(3, true, catalog::flawless_perfect(side), audio, "perfect");
            } else if f.life_max > 0.0 && f.life / f.life_max <= 0.05 {
                tr.trigger(2, true, catalog::magic_pixel(side), audio, "crowd");
            }
            if m.round_ticks > 0 && m.round_ticks <= 600 {
                tr.trigger(9, true, catalog::speedrun_ko(side), audio, "unlock");
            }
        }
    }
    if m.timer > 0 && m.timer <= 3 && m.round_state >= 3 {
        tr.trigger(10, true, catalog::buzzer_beater(), audio, "crowd");
    }
}
