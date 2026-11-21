use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Fire,
    Ice,
    Lightning,
    Water,
    Earth,
    Wind,
    Neutral,
}

impl Element {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "fire" | "flame" => Some(Self::Fire),
            "ice" | "frost" => Some(Self::Ice),
            "lightning" | "electric" | "thunder" => Some(Self::Lightning),
            "water" | "tidal" => Some(Self::Water),
            "earth" | "stone" => Some(Self::Earth),
            "wind" | "storm" => Some(Self::Wind),
            "neutral" | "ki" | "shadow" | "dark" | "light" => Some(Self::Neutral),
            _ => None,
        }
    }

    pub fn advantage_multiplier(attacker: Option<Self>, defender_weakness: Option<Self>) -> f32 {
        match (attacker, defender_weakness) {
            (Some(a), Some(w)) if a != Self::Neutral && a == w => 1.10,
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_elements_and_aliases() {
        assert_eq!(Element::parse("fire"), Some(Element::Fire));
        assert_eq!(Element::parse("FROST"), Some(Element::Ice));
        assert_eq!(Element::parse("thunder"), Some(Element::Lightning));
        assert_eq!(Element::parse("ki"), Some(Element::Neutral));
        assert_eq!(Element::parse("unknown"), None);
    }

    #[test]
    fn advantage_multiplier_rewards_hitting_weakness_only() {
        assert_eq!(
            Element::advantage_multiplier(Some(Element::Fire), Some(Element::Fire)),
            1.10
        );
        assert_eq!(
            Element::advantage_multiplier(Some(Element::Fire), Some(Element::Ice)),
            1.0
        );
        assert_eq!(
            Element::advantage_multiplier(Some(Element::Neutral), Some(Element::Neutral)),
            1.0
        );
        assert_eq!(
            Element::advantage_multiplier(None, Some(Element::Fire)),
            1.0
        );
    }
}
