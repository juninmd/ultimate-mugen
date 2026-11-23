use crate::i18n::Lang;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

pub const DEFAULT_TOML: &str = include_str!("../../assets/story/default.toml");

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LocalizedText {
    Plain(String),
    Map {
        #[serde(default)]
        en: String,
        #[serde(default)]
        pt: String,
        #[serde(default)]
        es: String,
    },
}

impl Default for LocalizedText {
    fn default() -> Self {
        Self::Plain(String::new())
    }
}

impl LocalizedText {
    pub fn pick(&self, lang: Lang) -> &str {
        match self {
            Self::Plain(s) => s,
            Self::Map { en, pt, es } => match lang {
                Lang::Pt if !pt.is_empty() => pt,
                Lang::Es if !es.is_empty() => es,
                _ if !en.is_empty() => en,
                _ if !pt.is_empty() => pt,
                _ => es,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LocalizedLines {
    Plain(Vec<String>),
    Map {
        #[serde(default)]
        en: Vec<String>,
        #[serde(default)]
        pt: Vec<String>,
        #[serde(default)]
        es: Vec<String>,
    },
}

impl Default for LocalizedLines {
    fn default() -> Self {
        Self::Plain(Vec::new())
    }
}

impl LocalizedLines {
    pub fn pick(&self, lang: Lang) -> &[String] {
        match self {
            Self::Plain(v) => v,
            Self::Map { en, pt, es } => match lang {
                Lang::Pt if !pt.is_empty() => pt,
                Lang::Es if !es.is_empty() => es,
                _ if !en.is_empty() => en,
                _ if !pt.is_empty() => pt,
                _ => es,
            },
        }
    }
}

fn default_true() -> bool { true }
fn default_credits() -> u8 { 3 }
fn default_wave_recovery() -> f32 { 0.2 }
fn default_stage_recovery() -> f32 { 0.6 }
fn default_boss_life() -> f32 { 1.5 }
fn default_boss_attack() -> f32 { 1.1 }
fn default_accent() -> [f32; 3] { [1.0, 0.72, 0.2] }
fn default_lane_far() -> f32 { -20.0 }
fn default_lane_near() -> f32 { 8.0 }

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CampaignMeta {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: LocalizedText,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_credits")]
    pub credits: u8,
    #[serde(default = "default_wave_recovery")]
    pub wave_recovery: f32,
    #[serde(default = "default_stage_recovery")]
    pub stage_recovery: f32,
    #[serde(default = "default_boss_life")]
    pub boss_life_factor: f32,
    #[serde(default = "default_boss_attack")]
    pub boss_attack_factor: f32,
}

impl Default for CampaignMeta {
    fn default() -> Self {
        Self {
            id: "default".into(),
            title: LocalizedText::default(),
            enabled: true,
            credits: default_credits(),
            wave_recovery: default_wave_recovery(),
            stage_recovery: default_stage_recovery(),
            boss_life_factor: default_boss_life(),
            boss_attack_factor: default_boss_attack(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ScriptStage {
    pub arena: String,
    #[serde(default)]
    pub title: LocalizedText,
    #[serde(default = "default_accent")]
    pub accent: [f32; 3],
    #[serde(default = "default_lane_far")]
    pub lane_far: f32,
    #[serde(default = "default_lane_near")]
    pub lane_near: f32,
    pub waves: Vec<Vec<String>>,
    pub boss: String,
    #[serde(default)]
    pub alt_boss: String,
    #[serde(default)]
    pub beats: LocalizedLines,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StoryScript {
    #[serde(default)]
    pub campaign: CampaignMeta,
    #[serde(default)]
    pub stages: Vec<ScriptStage>,
    #[serde(skip)]
    pub custom_source: bool,
}

impl StoryScript {
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut parsed: Self = toml::from_str(source).map_err(|e| format!("story toml: {e}"))?;
        if parsed.campaign.enabled && parsed.stages.is_empty() {
            return Err("story toml: enabled campaign must declare at least one [[stages]]".into());
        }
        for (i, st) in parsed.stages.iter_mut().enumerate() {
            if st.arena.trim().is_empty() {
                return Err(format!("story toml: stage {i} missing arena"));
            }
            if st.boss.trim().is_empty() {
                return Err(format!("story toml: stage {i} missing boss"));
            }
            if st.alt_boss.trim().is_empty() {
                st.alt_boss = st.boss.clone();
            }
            st.waves.retain(|w| !w.is_empty());
        }
        Ok(parsed)
    }

    pub fn default_campaign() -> Self {
        Self::parse(DEFAULT_TOML).expect("embedded default story.toml must be valid")
    }

    pub fn stage(&self, index: usize) -> Option<&ScriptStage> {
        if self.stages.is_empty() {
            return None;
        }
        Some(&self.stages[index.min(self.stages.len() - 1)])
    }

    pub fn title(&self, lang: Lang) -> &str {
        self.campaign.title.pick(lang)
    }

    pub fn stage_title(&self, stage: usize, lang: Lang) -> &str {
        self.stage(stage).map(|s| s.title.pick(lang)).unwrap_or("")
    }

    pub fn stage_beat(&self, stage: usize, beat: usize, lang: Lang) -> &str {
        let Some(st) = self.stage(stage) else {
            return "";
        };
        let lines = st.beats.pick(lang);
        if lines.is_empty() {
            return "";
        }
        &lines[beat.min(lines.len() - 1)]
    }
}

static ACTIVE: RwLock<Option<&'static StoryScript>> = RwLock::new(None);

fn load_initial() -> StoryScript {
    if let Some(raw) = super::script_source::read_external_toml() {
        if let Ok(mut custom) = StoryScript::parse(&raw) {
            custom.custom_source = true;
            return custom;
        }
    }
    StoryScript::default_campaign()
}

pub fn active() -> &'static StoryScript {
    if let Some(script) = *ACTIVE.read().unwrap() {
        return script;
    }
    let mut slot = ACTIVE.write().unwrap();
    *slot.get_or_insert_with(|| Box::leak(Box::new(load_initial())))
}

pub fn install(script: StoryScript) {
    *ACTIVE.write().unwrap() = Some(Box::leak(Box::new(script)));
}

#[cfg(test)]
#[path = "script_tests.rs"]
mod tests;
