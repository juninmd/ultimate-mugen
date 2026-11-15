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
    pub nationality: LocalText,
    #[serde(default)]
    pub bio: LocalText,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> CharProfile {
        serde_json::from_str(
            r#"{"nationality":{"en":"Japanese","pt":"Japonês"},"bio":{"pt":"Só em português."}}"#,
        )
        .unwrap()
    }

    #[test]
    fn requested_language_wins() {
        assert_eq!(profile().nationality.pick("pt"), "Japonês");
    }

    #[test]
    fn missing_language_falls_back_to_english_then_to_any() {
        assert_eq!(profile().nationality.pick("es"), "Japanese");
        assert_eq!(profile().bio.pick("es"), "Só em português.");
    }

    #[test]
    fn empty_text_is_empty() {
        assert_eq!(CharProfile::default().bio.pick("en"), "");
    }
}
