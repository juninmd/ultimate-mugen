use crate::registry::{CharEntry, Src};
use std::collections::BTreeMap;
use std::sync::OnceLock;
use ume_native::CharProfile;

#[cfg(test)]
mod tests;
mod wrap;

pub(crate) use wrap::wrap;

const BUNDLED_TEXT: &str = include_str!("../assets/char_profiles.json");

fn bundled_table() -> &'static BTreeMap<String, CharProfile> {
    static TABLE: OnceLock<BTreeMap<String, CharProfile>> = OnceLock::new();
    TABLE.get_or_init(|| serde_json::from_str(BUNDLED_TEXT).unwrap_or_default())
}

pub(crate) fn bundled(id: &str) -> Option<&'static CharProfile> {
    bundled_table().get(id.strip_prefix("umf:").unwrap_or(id))
}

#[cfg(test)]
pub(crate) fn bundled_all() -> impl Iterator<Item = (&'static String, &'static CharProfile)> {
    bundled_table().iter()
}

pub(crate) fn of(entry: &CharEntry) -> Option<&CharProfile> {
    let own = match &entry.src {
        Src::Native(native) => native.profile.as_ref(),
        Src::Mugen { .. } => None,
    };
    own.or_else(|| bundled(&entry.id))
}

pub(crate) fn lang() -> &'static str {
    crate::i18n::get().code()
}

pub(crate) fn nationality_label() -> &'static str {
    crate::i18n::tr3("Nacionalidade", "Nationality", "Nacionalidad")
}

pub(crate) fn alignment_label(raw: &str) -> Option<(&'static str, [f32; 3])> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "hero" => Some((crate::i18n::tr3("HERÓI", "HERO", "HÉROE"), [0.24, 0.82, 0.54])),
        "villain" => Some((crate::i18n::tr3("VILÃO", "VILLAIN", "VILLANO"), [0.95, 0.24, 0.36])),
        "anti-hero" => Some((crate::i18n::tr3("ANTI-HERÓI", "ANTI-HERO", "ANTIHÉROE"), [0.78, 0.42, 0.96])),
        "neutral" => Some((crate::i18n::tr3("NEUTRO", "NEUTRAL", "NEUTRO"), [0.68, 0.74, 0.84])),
        _ => None,
    }
}

pub(crate) fn matches_query(profile: &CharProfile, q: &str) -> bool {
    let hit = |s: &str| s.to_lowercase().contains(q);
    if hit(&profile.country)
        || profile.element.as_deref().is_some_and(hit)
        || profile.weakness.as_deref().is_some_and(hit)
        || profile.alignment.as_deref().is_some_and(|a| {
            hit(a) || alignment_label(a).is_some_and(|(lbl, _)| hit(lbl))
        })
        || profile.tags.iter().any(|t| hit(t))
    {
        return true;
    }
    [&profile.nationality, &profile.origin, &profile.fighting_style, &profile.affiliation]
        .iter()
        .any(|lt| lt.0.values().any(|v| hit(v)))
}
