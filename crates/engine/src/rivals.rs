use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ATTACK_MIN: f32 = 0.9;
pub const ATTACK_MAX: f32 = 1.15;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RivalLines {
    #[serde(default)]
    pub intro: Option<String>,
    #[serde(default)]
    pub taunt: Option<String>,
    #[serde(default)]
    pub attack: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rivals {
    pub id: String,
    lines: BTreeMap<String, RivalLines>,
}

impl Rivals {
    pub fn new(id: &str, lines: &BTreeMap<String, RivalLines>) -> Self {
        Rivals {
            id: id.to_string(),
            lines: lines.clone(),
        }
    }

    pub fn against(&self, opponent_id: &str) -> Option<&RivalLines> {
        self.lines.get(opponent_id)
    }

    pub fn attack_against(&self, opponent_id: &str) -> f32 {
        self.against(opponent_id)
            .and_then(|r| r.attack)
            .map_or(1.0, |a| a.clamp(ATTACK_MIN, ATTACK_MAX))
    }

    pub fn line_against(&self, opponent_id: &str, answer: bool) -> Option<&str> {
        let rival = self.against(opponent_id)?;
        let text = if answer { &rival.taunt } else { &rival.intro };
        text.as_deref().filter(|t| !t.is_empty())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
