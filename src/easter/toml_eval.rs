use super::toml_config::{build_egg, config, parse_sfx, pick_text};
use super::FightTracker;
use crate::audio::Audio;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

fn roots(m: &Match) -> Option<(&Fighter, &Fighter)> {
    let p1 = m.roots().find(|f| f.side == 0)?;
    let p2 = m.roots().find(|f| f.side == 1)?;
    Some((p1, p2))
}

fn id_is(f: &Fighter, target: &str) -> bool {
    !target.is_empty() && f.def.id.eq_ignore_ascii_case(target)
}

fn slot_bit(base: u8, idx: usize) -> u8 {
    base.saturating_add((idx as u8) % 28).min(126)
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
    let cfg = config();

    if m.round_ticks == 45 && !check_matchups(p1, p2, tr, audio, cfg) {
        check_stage_interactions(fighters, &stage, tr, audio, cfg);
    }
    check_fighter_triggers(fighters, tr, audio, cfg);
}

fn check_matchups(
    p1: &Fighter,
    p2: &Fighter,
    tr: &mut FightTracker,
    audio: &mut Audio,
    cfg: &super::toml_config::InteractionsConfig,
) -> bool {
    let is_mirror = p1.def.id.eq_ignore_ascii_case(&p2.def.id) && !p1.def.id.is_empty();
    for (idx, entry) in cfg.matchup.iter().enumerate() {
        let matched = if entry.mirror {
            is_mirror
        } else if entry.fighters.len() == 2 {
            let (a, b) = (&entry.fighters[0], &entry.fighters[1]);
            (id_is(p1, a) && id_is(p2, b)) || (id_is(p1, b) && id_is(p2, a))
        } else {
            false
        };
        if !matched {
            continue;
        }
        let side = if id_is(p2, &entry.speaker_id) { 1 } else { 0 };
        let speaker = if !entry.speaker.is_empty() {
            entry.speaker.as_str()
        } else if entry.mirror {
            p1.def.display_name.as_str()
        } else {
            ""
        };
        let title = pick_text(&entry.title_pt, &entry.title_en, &entry.title_es);
        let sub = pick_text(&entry.sub_pt, &entry.sub_en, &entry.sub_es);
        let egg = build_egg(&entry.style, &entry.sprite, speaker, title, sub, entry.color, side);
        if tr.trigger(slot_bit(40, idx), false, egg, audio, parse_sfx(&entry.sfx)) {
            return true;
        }
    }
    false
}

fn check_stage_interactions(
    fighters: [&Fighter; 2],
    stage: &str,
    tr: &mut FightTracker,
    audio: &mut Audio,
    cfg: &super::toml_config::InteractionsConfig,
) {
    for (idx, entry) in cfg.stage_interaction.iter().enumerate() {
        if !entry.stage_contains.iter().any(|k| stage.contains(&k.to_lowercase())) {
            continue;
        }
        for side in 0..2 {
            if !id_is(fighters[side], &entry.fighter) {
                continue;
            }
            let title = pick_text(&entry.title_pt, &entry.title_en, &entry.title_es);
            let sub = pick_text(&entry.sub_pt, &entry.sub_en, &entry.sub_es);
            let egg = build_egg(&entry.style, &entry.sprite, &entry.speaker, title, sub, entry.color, side);
            if tr.trigger(slot_bit(68, idx), false, egg, audio, parse_sfx(&entry.sfx)) {
                return;
            }
        }
    }
}

fn check_fighter_triggers(
    fighters: [&Fighter; 2],
    tr: &mut FightTracker,
    audio: &mut Audio,
    cfg: &super::toml_config::InteractionsConfig,
) {
    for (idx, entry) in cfg.fighter_trigger.iter().enumerate() {
        for side in 0..2 {
            let f = fighters[side];
            let opp = fighters[1 - side];
            let fighter_match = if !entry.fighter.is_empty() {
                id_is(f, &entry.fighter)
            } else if !entry.fighter_prefix.is_empty() {
                f.def.id.to_lowercase().starts_with(&entry.fighter_prefix.to_lowercase())
            } else {
                true
            };
            if !fighter_match {
                continue;
            }
            let super_fired = tr.prev_power[side] - f.power >= 950.0;
            let hp_ratio = if f.life_max > 0.0 { f.life / f.life_max } else { 1.0 };
            let taunted = f.state_no == 195 && tr.prev_state[side] != 195;
            let fired = match entry.trigger.as_str() {
                "low_hp" => f.alive && hp_ratio <= entry.threshold.max(0.05),
                "combo" => f.combo_count as f32 >= entry.threshold.max(1.0),
                "opp_combo" => opp.combo_count as f32 >= entry.threshold.max(1.0),
                "super" => super_fired,
                "super_or_combo" => super_fired || f.combo_count as f32 >= entry.threshold.max(1.0),
                "taunt" => taunted,
                _ => false,
            };
            if !fired {
                continue;
            }
            let speaker = if !entry.speaker.is_empty() {
                entry.speaker.as_str()
            } else {
                f.def.display_name.as_str()
            };
            let title = pick_text(&entry.title_pt, &entry.title_en, &entry.title_es);
            let sub = pick_text(&entry.sub_pt, &entry.sub_en, &entry.sub_es);
            let egg = build_egg(&entry.style, &entry.sprite, speaker, title, sub, entry.color, side);
            tr.trigger(slot_bit(96, idx), entry.per_round, egg, audio, parse_sfx(&entry.sfx));
        }
    }
}
