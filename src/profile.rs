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
