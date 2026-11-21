use super::{lore_catalog as lc, FightTracker};
use crate::audio::Audio;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

fn roots(m: &Match) -> Option<(&Fighter, &Fighter)> {
    let p1 = m.roots().find(|f| f.side == 0)?;
    let p2 = m.roots().find(|f| f.side == 1)?;
    Some((p1, p2))
}

fn id_is(f: &Fighter, target: &str) -> bool {
    f.def.id.eq_ignore_ascii_case(target)
}

fn pair_is(a: &Fighter, b: &Fighter, x: &str, y: &str) -> bool {
    (id_is(a, x) && id_is(b, y)) || (id_is(a, y) && id_is(b, x))
}

pub(crate) fn check(m: &Match, stage_name: &str, tr: &mut FightTracker, audio: &mut Audio) {
    if m.round_state != 2 {
        return;
    }
    let Some((p1, p2)) = roots(m) else {
        return;
    };
    let fighters = [p1, p2];
    let stage = stage_name.to_lowercase();
    let t = m.round_ticks;

    if t == 45 {
        check_opening_matchups(p1, p2, &stage, tr, audio);
    }

    for side in 0..2 {
        let f = fighters[side];
        let opp = fighters[1 - side];
        let super_fired = tr.prev_power[side] - f.power >= 950.0;
        let hp_ratio = if f.life_max > 0.0 { f.life / f.life_max } else { 1.0 };

        if id_is(f, "akio") && f.alive && hp_ratio <= 0.30 {
            tr.trigger(46, false, lc::akio_fired_up(side), audio, "easter_crown");
        }
        if id_is(f, "haruki") && f.combo_count >= 5 {
            tr.trigger(47, true, lc::haruki_apology(side), audio, "easter_banter");
        }
        if id_is(f, "marlow") && (super_fired || f.combo_count >= 6) {
            tr.trigger(58, true, lc::marlow_parrot(side), audio, "easter_parrot");
        }
        if f.def.id.starts_with("npc_") && opp.combo_count >= 4 {
            tr.trigger(59, true, lc::thug_complaint(&f.def.display_name, side), audio, "easter_banter");
        }
    }
}

fn check_opening_matchups(p1: &Fighter, p2: &Fighter, stage: &str, tr: &mut FightTracker, audio: &mut Audio) {
    if p1.def.id.eq_ignore_ascii_case(&p2.def.id) && !p1.def.id.is_empty() {
        tr.trigger(40, false, lc::mirror_match(&p1.def.display_name), audio, "easter_banter");
        return;
    }
    if pair_is(p1, p2, "suzume", "oboro") {
        tr.trigger(41, false, lc::suzume_oboro(), audio, "easter_clash");
    } else if pair_is(p1, p2, "sylene", "garan") {
        tr.trigger(42, false, lc::sylene_garan(), audio, "easter_clash");
    } else if pair_is(p1, p2, "kaito", "oboro") {
        tr.trigger(43, false, lc::kaito_oboro(), audio, "easter_clash");
    } else if pair_is(p1, p2, "marlow", "valdric") {
        let side = if id_is(p1, "marlow") { 0 } else { 1 };
        tr.trigger(48, false, lc::marlow_valdric(side), audio, "easter_banter");
    } else if pair_is(p1, p2, "kaito", "jin") {
        tr.trigger(49, false, lc::kaito_jin(), audio, "easter_bird");
    } else if pair_is(p1, p2, "shion", "kaname") {
        tr.trigger(50, false, lc::shion_kaname(), audio, "easter_clash");
    } else if pair_is(p1, p2, "bruno", "daichi") {
        tr.trigger(52, false, lc::bruno_daichi(), audio, "easter_clash");
    } else if pair_is(p1, p2, "valdric", "oboro") {
        tr.trigger(54, false, lc::valdric_oboro(), audio, "easter_clash");
    } else if pair_is(p1, p2, "suzume", "jin") {
        tr.trigger(55, false, lc::suzume_jin(), audio, "easter_clash");
    } else if pair_is(p1, p2, "volta", "haruki") {
        let side = if id_is(p1, "volta") { 0 } else { 1 };
        tr.trigger(56, false, lc::volta_haruki(side), audio, "easter_banter");
    } else if pair_is(p1, p2, "akio", "kaito") {
        let side = if id_is(p1, "akio") { 0 } else { 1 };
        tr.trigger(57, false, lc::akio_kaito(side), audio, "easter_banter");
    } else {
        check_home_turf(p1, p2, stage, tr, audio);
    }
}

fn check_home_turf(p1: &Fighter, p2: &Fighter, stage: &str, tr: &mut FightTracker, audio: &mut Audio) {
    let fighters = [p1, p2];
    for side in 0..2 {
        let f = fighters[side];
        if id_is(f, "bruno") && (stage.contains("brasil") || stage.contains("brazil") || stage.contains("cristo") || stage.contains("corcovado")) {
            tr.trigger(44, false, lc::bruno_home_crowd(side), audio, "easter_capybara");
            return;
        }
        if id_is(f, "volta") && (stage.contains("neo-metr") || stage.contains("neon")) {
            tr.trigger(45, false, lc::volta_neon(side), audio, "easter_ufo");
            return;
        }
        if id_is(f, "lian") && (stage.contains("muralha") || stage.contains("great wall")) {
            tr.trigger(51, false, lc::lian_great_wall(side), audio, "easter_kitsune");
            return;
        }
        if id_is(f, "garan") && (stage.contains("vulc") || stage.contains("volcano")) {
            tr.trigger(53, false, lc::garan_volcano(side), audio, "easter_wonder");
            return;
        }
    }
}
