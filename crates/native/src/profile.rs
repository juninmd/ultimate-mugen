use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const FALLBACK_LANG: &str = "en";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalText(pub BTreeMap<String, String>);

impl LocalText {
    pub fn pick(&self, lang: &str) -> &str {
        self.0
            .get(lang)
            .or_else(|| self.0.get(FALLBACK_LANG))
            .or_else(|| self.0.values().next())
            .map(String::as_str)
            .unwrap_or("")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CharProfile {
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub nationality: LocalText,
    #[serde(default)]
    pub origin: LocalText,
    #[serde(default)]
    pub bio: LocalText,
    #[serde(default)]
    pub title: LocalText,
    #[serde(default, rename = "fightingStyle")]
    pub fighting_style: LocalText,
    #[serde(default)]
    pub affiliation: LocalText,
    #[serde(default)]
    pub element: Option<String>,
    #[serde(default)]
    pub weakness: Option<String>,
    #[serde(default)]
    pub alignment: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> CharProfile {
        serde_json::from_str(
            r#"{"nationality":{"en":"Japanese","pt":"Japonês"},"origin":{"en":"Planet Vegeta","pt":"Planeta Vegeta"},"bio":{"pt":"Só em português."},"element":"ki","weakness":"dark","alignment":"hero"}"#,
        )
        .unwrap()
    }

    #[test]
    fn requested_language_wins() {
        assert_eq!(profile().nationality.pick("pt"), "Japonês");
        assert_eq!(profile().origin.pick("pt"), "Planeta Vegeta");
    }

    #[test]
    fn missing_language_falls_back_to_english_then_to_any() {
        assert_eq!(profile().nationality.pick("es"), "Japanese");
        assert_eq!(profile().bio.pick("es"), "Só em português.");
    }

    #[test]
    fn element_weakness_and_alignment_deserialize() {
        let p = profile();
        assert_eq!(p.element.as_deref(), Some("ki"));
        assert_eq!(p.weakness.as_deref(), Some("dark"));
        assert_eq!(p.alignment.as_deref(), Some("hero"));
    }

    #[test]
    fn tags_are_optional_and_keep_their_order() {
        assert!(CharProfile::default().tags.is_empty());
        let tagged: CharProfile = serde_json::from_str(r#"{"tags":["rushdown","zoner"]}"#).unwrap();
        assert_eq!(tagged.tags, ["rushdown", "zoner"]);
    }

    #[test]
    fn empty_text_is_empty() {
        assert_eq!(CharProfile::default().bio.pick("en"), "");
    }
}
