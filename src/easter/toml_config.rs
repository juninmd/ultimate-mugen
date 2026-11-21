use super::{ActiveEgg, EggSprite, EggStyle};
use crate::i18n::{get, Lang};
use serde::Deserialize;
use std::sync::OnceLock;

const DEFAULT_TOML: &str = include_str!("../../assets/interactions.toml");
const USER_FILE: &str = "interactions.toml";

#[derive(Default, Deserialize)]
#[serde(default)]
pub(crate) struct InteractionsConfig {
    pub taunts: TauntsSection,
    pub matchup: Vec<MatchupEntry>,
    pub stage_interaction: Vec<StageInteraction>,
    pub fighter_trigger: Vec<FighterTrigger>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub(crate) struct TauntsSection {
    pub opening_pt: Vec<String>,
    pub opening_en: Vec<String>,
    pub opening_es: Vec<String>,
    pub answer_pt: Vec<String>,
    pub answer_en: Vec<String>,
    pub answer_es: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub(crate) struct MatchupEntry {
    pub id: String,
    pub mirror: bool,
    pub fighters: Vec<String>,
    pub speaker_id: String,
    pub speaker: String,
    pub style: String,
    pub sprite: String,
    pub sfx: String,
    pub color: [f32; 3],
    pub title_pt: String,
    pub title_en: String,
    pub title_es: String,
    pub sub_pt: String,
    pub sub_en: String,
    pub sub_es: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub(crate) struct StageInteraction {
    pub id: String,
    pub fighter: String,
    pub stage_contains: Vec<String>,
    pub speaker: String,
    pub style: String,
    pub sprite: String,
    pub sfx: String,
    pub color: [f32; 3],
    pub title_pt: String,
    pub title_en: String,
    pub title_es: String,
    pub sub_pt: String,
    pub sub_en: String,
    pub sub_es: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub(crate) struct FighterTrigger {
    pub id: String,
    pub fighter: String,
    pub fighter_prefix: String,
    pub trigger: String,
    pub threshold: f32,
    pub per_round: bool,
    pub speaker: String,
    pub style: String,
    pub sprite: String,
    pub sfx: String,
    pub color: [f32; 3],
    pub title_pt: String,
    pub title_en: String,
    pub title_es: String,
    pub sub_pt: String,
    pub sub_en: String,
    pub sub_es: String,
}

pub(crate) fn config() -> &'static InteractionsConfig {
    static CONFIG: OnceLock<InteractionsConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut base: InteractionsConfig = toml::from_str(DEFAULT_TOML).unwrap_or_default();
        if let Some(user_text) = crate::storage::read(USER_FILE) {
            if let Ok(extra) = toml::from_str::<InteractionsConfig>(&user_text) {
                merge_config(&mut base, extra);
            }
        }
        base
    })
}

fn merge_config(base: &mut InteractionsConfig, extra: InteractionsConfig) {
    if !extra.taunts.opening_pt.is_empty() { base.taunts.opening_pt = extra.taunts.opening_pt; }
    if !extra.taunts.opening_en.is_empty() { base.taunts.opening_en = extra.taunts.opening_en; }
    if !extra.taunts.opening_es.is_empty() { base.taunts.opening_es = extra.taunts.opening_es; }
    if !extra.taunts.answer_pt.is_empty() { base.taunts.answer_pt = extra.taunts.answer_pt; }
    if !extra.taunts.answer_en.is_empty() { base.taunts.answer_en = extra.taunts.answer_en; }
    if !extra.taunts.answer_es.is_empty() { base.taunts.answer_es = extra.taunts.answer_es; }
    base.matchup.extend(extra.matchup);
    base.stage_interaction.extend(extra.stage_interaction);
    base.fighter_trigger.extend(extra.fighter_trigger);
}

pub(crate) fn generic_taunt(answer: bool, pick: usize) -> &'static str {
    let t = &config().taunts;
    let list = match (answer, get()) {
        (false, Lang::Pt) => &t.opening_pt,
        (false, Lang::En) => &t.opening_en,
        (false, Lang::Es) => &t.opening_es,
        (true, Lang::Pt) => &t.answer_pt,
        (true, Lang::En) => &t.answer_en,
        (true, Lang::Es) => &t.answer_es,
    };
    list.get(pick % list.len().max(1)).map(String::as_str).unwrap_or("!")
}

pub(crate) fn pick_text<'a>(pt: &'a str, en: &'a str, es: &'a str) -> &'a str {
    let chosen = match get() {
        Lang::Pt => pt,
        Lang::En => en,
        Lang::Es => es,
    };
    if !chosen.is_empty() { chosen } else if !en.is_empty() { en } else { pt }
}

pub(crate) fn parse_sprite(name: &str) -> EggSprite {
    match name {
        "toasty_ref" => EggSprite::ToastyRef,
        "tumbleweed" => EggSprite::Tumbleweed,
        "arcade_coin" => EggSprite::ArcadeCoin,
        "crown_perfect" => EggSprite::CrownPerfect,
        "stopwatch" => EggSprite::Stopwatch,
        "shield_fortress" => EggSprite::ShieldFortress,
        "origami_crane" => EggSprite::OrigamiCrane,
        "ufo_neon" => EggSprite::UfoNeon,
        "kitsune_mask" => EggSprite::KitsuneMask,
        "capybara" => EggSprite::Capybara,
        "toucan" => EggSprite::Toucan,
        "scarab_laurel" => EggSprite::ScarabLaurel,
        "pirate_parrot" => EggSprite::PirateParrot,
        "ramen_bowl" => EggSprite::RamenBowl,
        "crossed_blades" => EggSprite::CrossedBlades,
        "eclipse_moon" => EggSprite::EclipseMoon,
        _ => EggSprite::None,
    }
}

pub(crate) fn parse_style(name: &str) -> EggStyle {
    match name {
        "corner_pop" => EggStyle::CornerPop,
        "floor_roll" => EggStyle::FloorRoll,
        "flyby" => EggStyle::Flyby,
        "speech" => EggStyle::SpeechBubble,
        _ => EggStyle::TopRibbon,
    }
}

pub(crate) fn parse_sfx(name: &str) -> &'static str {
    match name {
        "easter_toasty" => "easter_toasty",
        "easter_coin" => "easter_coin",
        "easter_crown" => "easter_crown",
        "easter_clash" => "easter_clash",
        "easter_tumbleweed" => "easter_tumbleweed",
        "easter_ufo" => "easter_ufo",
        "easter_kitsune" => "easter_kitsune",
        "easter_capybara" => "easter_capybara",
        "easter_bird" => "easter_bird",
        "easter_parrot" => "easter_parrot",
        "easter_wonder" => "easter_wonder",
        "easter_banter" => "easter_banter",
        _ => "",
    }
}

pub(crate) fn build_egg(
    style_name: &str,
    sprite_name: &str,
    speaker: &str,
    title: &str,
    sub: &str,
    color: [f32; 3],
    side: usize,
) -> ActiveEgg {
    let style = parse_style(style_name);
    let dur = match style {
        EggStyle::CornerPop => 2.3,
        EggStyle::FloorRoll => 3.4,
        EggStyle::Flyby => 3.8,
        EggStyle::SpeechBubble => 2.9,
        EggStyle::TopRibbon => 2.8,
    };
    let col = if color == [0.0, 0.0, 0.0] { [1.0, 0.82, 0.32] } else { color };
    ActiveEgg {
        style,
        sprite: parse_sprite(sprite_name),
        title: title.to_string(),
        sub: sub.to_string(),
        speaker: speaker.to_string(),
        color: col,
        side,
        t: 0.0,
        dur,
        flip: side == 1,
    }
}
