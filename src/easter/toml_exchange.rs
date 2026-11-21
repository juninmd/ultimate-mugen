use super::toml_config::{build_egg, parse_sfx, pick_text};
use super::FightTracker;
use crate::audio::Audio;
use serde::Deserialize;
use ume_engine::fighter::Fighter;

#[derive(Default, Deserialize)]
#[serde(default)]
pub(crate) struct ExchangeEntry {
    pub id: String,
    pub fighters: Vec<String>,
    pub first_id: String,
    pub first_speaker: String,
    pub second_speaker: String,
    pub delay_ticks: i64,
    pub sfx: String,
    pub color_a: [f32; 3],
    pub color_b: [f32; 3],
    pub line_a_pt: String,
    pub line_a_en: String,
    pub line_a_es: String,
    pub line_b_pt: String,
    pub line_b_en: String,
    pub line_b_es: String,
}

fn id_is(f: &Fighter, target: &str) -> bool {
    !target.is_empty() && f.def.id.eq_ignore_ascii_case(target)
}

pub(crate) fn check(
    p1: &Fighter,
    p2: &Fighter,
    tr: &mut FightTracker,
    audio: &mut Audio,
    exchanges: &[ExchangeEntry],
) -> bool {
    for (idx, entry) in exchanges.iter().enumerate() {
        if entry.fighters.len() != 2 {
            continue;
        }
        let (a, b) = (&entry.fighters[0], &entry.fighters[1]);
        let matched = (id_is(p1, a) && id_is(p2, b)) || (id_is(p1, b) && id_is(p2, a));
        if !matched {
            continue;
        }
        let first_side = if id_is(p2, &entry.first_id) { 1 } else { 0 };
        let second_side = 1 - first_side;
        let first_f = if first_side == 0 { p1 } else { p2 };
        let second_f = if second_side == 0 { p1 } else { p2 };
        let spk_a = if !entry.first_speaker.is_empty() {
            entry.first_speaker.as_str()
        } else {
            first_f.def.display_name.as_str()
        };
        let spk_b = if !entry.second_speaker.is_empty() {
            entry.second_speaker.as_str()
        } else {
            second_f.def.display_name.as_str()
        };
        let line_a = pick_text(&entry.line_a_pt, &entry.line_a_en, &entry.line_a_es);
        let line_b = pick_text(&entry.line_b_pt, &entry.line_b_en, &entry.line_b_es);
        let sfx = parse_sfx(&entry.sfx);
        let egg_a = build_egg("speech", "", spk_a, line_a, "", entry.color_a, first_side);
        let bit = (112u8.saturating_add((idx as u8) % 14)).min(126);
        if tr.trigger(bit, false, egg_a, audio, sfx) {
            if !line_b.is_empty() {
                let delay = if entry.delay_ticks > 0 { entry.delay_ticks } else { 130 };
                let egg_b = build_egg("speech", "", spk_b, line_b, "", entry.color_b, second_side);
                tr.queue_delayed(delay, egg_b, sfx);
            }
            return true;
        }
    }
    false
}
