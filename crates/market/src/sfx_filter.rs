use crate::model::Kind;
use crate::safety::{file_stem, parent_dir};
use std::collections::HashSet;

pub(crate) fn is_fighter_sfx(path: &str, sprite_dirs: &HashSet<&str>) -> bool {
    let stem = file_stem(path).to_ascii_lowercase();
    let numbered = stem.strip_prefix("snd_").is_some_and(|r| !r.is_empty() && r.split('_').all(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())));
    numbered || sprite_dirs.contains(parent_dir(path))
}

/// Installed music entries saved before sfx were excluded from the catalog.
pub fn is_stale_fighter_sfx_music(kind: Kind, item_id: &str) -> bool {
    kind == Kind::Music && is_fighter_sfx(item_id.strip_prefix("music:").unwrap_or(item_id), &HashSet::new())
}
