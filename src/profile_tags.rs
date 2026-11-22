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
        "power" | "powerhouse" => Some(("FORÇA BRUTA", "POWERHOUSE", "FUERZA BRUTA")),
        "super-speed" => Some(("SUPERVELOCIDADE", "SUPER SPEED", "SUPERVELOCIDAD")),
        "teleport" => Some(("TELEPORTE", "TELEPORT", "TELETRANSPORTE")),
        "sword" => Some(("ESPADA", "SWORD", "ESPADA")),
        "armor" => Some(("ARMADURA", "ARMOR", "ARMADURA")),
        "beam" => Some(("RAIO", "BEAM", "RAYO")),
        "heavy" => Some(("PESADO", "HEAVY", "PESADO")),
        "brawler" => Some(("BRIGÃO", "BRAWLER", "PELEADOR")),
        "dark" => Some(("SOMBRIO", "DARK", "OSCURO")),
        "demon" => Some(("DEMÔNIO", "DEMON", "DEMONIO")),
        "android" => Some(("ANDROIDE", "ANDROID", "ANDROIDE")),
        "kicker" => Some(("CHUTADOR", "KICKER", "PATEADOR")),
        "combo" => Some(("COMBOS", "COMBO", "COMBOS")),
        "boss" => Some(("CHEFE", "BOSS", "JEFE")),
        "illusion" => Some(("ILUSÃO", "ILLUSION", "ILUSIÓN")),
        "prince" => Some(("PRÍNCIPE", "PRINCE", "PRÍNCIPE")),
        "mixup" => Some(("MIX-UP", "MIX-UP", "MIX-UP")),
        "mountain" => Some(("MONTANHA", "MOUNTAIN", "MONTAÑA")),
        "clones" => Some(("CLONES", "CLONES", "CLONES")),
        "tactician" => Some(("ESTRATEGISTA", "TACTICIAN", "ESTRATEGA")),
        "jade" => Some(("JADE", "JADE", "JADE")),
        other => element_words(other),
    };
    match known {
        Some((pt, en, es)) => crate::i18n::tr3(pt, en, es).to_string(),
        None => tag.replace(['-', '_'], " ").to_uppercase(),
    }
}

fn element_words(raw: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match raw {
        "fire" => Some(("FOGO", "FIRE", "FUEGO")),
        "ice" => Some(("GELO", "ICE", "HIELO")),
        "wind" => Some(("VENTO", "WIND", "VIENTO")),
        "earth" => Some(("TERRA", "EARTH", "TIERRA")),
        "water" => Some(("ÁGUA", "WATER", "AGUA")),
        "lightning" => Some(("RAIO", "LIGHTNING", "RAYO")),
        "shadow" => Some(("SOMBRA", "SHADOW", "SOMBRA")),
        "light" => Some(("LUZ", "LIGHT", "LUZ")),
        "ki" => Some(("KI", "KI", "KI")),
        _ => None,
    }
}

pub(crate) fn element_chip(element: &str, weakness: Option<&str>) -> String {
    let word = |raw: &str| {
        let raw = raw.trim().to_ascii_lowercase();
        element_words(&raw).map_or_else(|| raw.to_uppercase(), |(pt, en, es)| crate::i18n::tr3(pt, en, es).to_string())
    };
    match weakness {
        Some(weak) => format!("{} · {} {}", word(element), crate::i18n::tr3("FRACO A", "WEAK TO", "DÉBIL A"), word(weak)),
        None => word(element),
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
            "super-speed", "teleport", "sword", "dark", "wind",
        ] {
            assert!(!label(tag).is_empty(), "{tag}");
        }
    }

    #[test]
    fn element_chip_spells_out_the_weakness() {
        let chip = element_chip("wind", Some("earth"));
        assert!(chip.contains(" · "), "{chip}");
        assert!(!chip.contains("/ -"), "{chip}");
    }
}
