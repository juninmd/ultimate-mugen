pub(crate) const MAX_SHOWN: usize = 3;

pub(crate) fn label(tag: &str) -> String {
    let known = match tag {
        "rushdown" => Some(("RUSHDOWN", "RUSHDOWN", "RUSHDOWN")),
        "grappler" => Some(("AGARRADOR", "GRAPPLER", "LUCHADOR")),
        "zoner" => Some(("ZONER", "ZONER", "ZONER")),
        "balanced" => Some(("EQUILIBRADO", "ALL-ROUNDER", "EQUILIBRADO")),
        "aerial" => Some(("AÉREO", "AERIAL", "AÉREO")),
        "trapper" => Some(("ARMADILHEIRO", "TRAPPER", "TRAMPERO")),
        "counter" => Some(("CONTRA-ATAQUE", "COUNTER", "CONTRAATAQUE")),
        "power" => Some(("FORÇA BRUTA", "POWERHOUSE", "FUERZA BRUTA")),
        _ => None,
    };
    match known {
        Some((pt, en, es)) => crate::i18n::tr3(pt, en, es).to_string(),
        None => tag.replace(['-', '_'], " ").to_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_tags_are_shown_uppercased_with_spaces() {
        assert_eq!(label("storm-caller"), "STORM CALLER");
        assert_eq!(label("lotus_bomber"), "LOTUS BOMBER");
    }

    #[test]
    fn known_tags_have_a_translation() {
        for tag in [
            "rushdown", "grappler", "zoner", "balanced", "aerial", "trapper", "counter", "power",
        ] {
            assert!(!label(tag).is_empty(), "{tag}");
        }
    }
}
