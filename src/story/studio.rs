use super::script::{self, CampaignMeta, LocalizedLines, StoryScript};
use super::script_source::STORE_KEY;
use crate::i18n::Lang;

pub const BEATS: usize = 4;
pub const BEAT_MAX_CHARS: usize = 96;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    Credits,
    WaveRecovery,
    StageRecovery,
    BossLife,
    BossAttack,
}

pub const RULES: [Rule; 5] = [
    Rule::Credits,
    Rule::WaveRecovery,
    Rule::StageRecovery,
    Rule::BossLife,
    Rule::BossAttack,
];

impl Rule {
    pub fn label(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Credits => ("Créditos", "Credits", "Créditos"),
            Self::WaveRecovery => ("Vida recuperada por onda", "Life regained per wave", "Vida recuperada por oleada"),
            Self::StageRecovery => ("Vida recuperada por fase", "Life regained per stage", "Vida recuperada por fase"),
            Self::BossLife => ("Vida do chefe", "Boss life", "Vida del jefe"),
            Self::BossAttack => ("Ataque do chefe", "Boss attack", "Ataque del jefe"),
        }
    }

    pub fn display(self, meta: &CampaignMeta) -> String {
        match self {
            Self::Credits => meta.credits.to_string(),
            Self::WaveRecovery => format!("{}%", percent(meta.wave_recovery)),
            Self::StageRecovery => format!("{}%", percent(meta.stage_recovery)),
            Self::BossLife => format!("{}%", percent(meta.boss_life_factor)),
            Self::BossAttack => format!("{}%", percent(meta.boss_attack_factor)),
        }
    }

    pub fn step(self, meta: &mut CampaignMeta, dir: i32) {
        let d = dir.signum() as f32;
        match self {
            Self::Credits => meta.credits = (meta.credits as i32 + dir.signum()).clamp(1, 9) as u8,
            Self::WaveRecovery => meta.wave_recovery = nudge(meta.wave_recovery, d * 0.05, 0.0, 1.0),
            Self::StageRecovery => meta.stage_recovery = nudge(meta.stage_recovery, d * 0.05, 0.0, 1.0),
            Self::BossLife => meta.boss_life_factor = nudge(meta.boss_life_factor, d * 0.1, 0.5, 3.0),
            Self::BossAttack => meta.boss_attack_factor = nudge(meta.boss_attack_factor, d * 0.1, 0.5, 2.0),
        }
    }
}

fn percent(v: f32) -> i32 {
    (v * 100.0).round() as i32
}

fn nudge(v: f32, delta: f32, lo: f32, hi: f32) -> f32 {
    ((v + delta).clamp(lo, hi) * 100.0).round() / 100.0
}

fn expanded(lines: &LocalizedLines) -> [Vec<String>; 3] {
    let mut out = [Lang::En, Lang::Pt, Lang::Es].map(|lang| lines.pick(lang).to_vec());
    for slot in out.iter_mut() {
        let last = slot.last().cloned().unwrap_or_default();
        slot.resize(BEATS.max(slot.len()), last);
    }
    out
}

pub fn set_beat(script: &mut StoryScript, stage: usize, beat: usize, lang: Lang, text: &str) {
    let Some(st) = script.stages.get_mut(stage) else { return };
    let [mut en, mut pt, mut es] = expanded(&st.beats);
    let target = match lang {
        Lang::En => &mut en,
        Lang::Pt => &mut pt,
        Lang::Es => &mut es,
    };
    if let Some(slot) = target.get_mut(beat) {
        *slot = text.trim().chars().take(BEAT_MAX_CHARS).collect();
    }
    st.beats = LocalizedLines::Map { en, pt, es };
}

pub fn to_toml(script: &StoryScript) -> Result<String, String> {
    toml::to_string(script).map_err(|e| format!("story toml: {e}"))
}

pub fn save(script: &StoryScript) -> Result<(), String> {
    let text = to_toml(script)?;
    let mut checked = StoryScript::parse(&text)?;
    checked.custom_source = true;
    crate::storage::write(STORE_KEY, &text);
    script::install(checked);
    Ok(())
}

pub fn reset() -> StoryScript {
    crate::storage::write(STORE_KEY, "");
    let script = StoryScript::default_campaign();
    script::install(script.clone());
    script
}

#[cfg(test)]
#[path = "studio_tests.rs"]
mod tests;
