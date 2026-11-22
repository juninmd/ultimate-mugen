use crate::model::{Catalog, Item, Kind, TreeEntry};
use crate::safety::lower_ext;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Format {
    Mugen,
    Umf,
}

impl Format {
    pub const ALL: [Format; 2] = [Format::Mugen, Format::Umf];

    pub fn label(self) -> &'static str {
        match self {
            Format::Mugen => "MUGEN",
            Format::Umf => "UMF",
        }
    }
}

const PREVIEW_DIRS: [&str; 6] = ["previews", "preview", "portraits", "portrait", "thumbs", "thumbnails"];

pub fn id_path(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, path)| path)
}

fn umf_word(segment: &str) -> bool {
    let s = segment.strip_suffix(".zip").unwrap_or(segment);
    s == "umf" || s.ends_with("_umf") || s.ends_with("-umf") || s.ends_with(" umf") || s.starts_with("umf_")
}

pub fn format_of(kind: Kind, id: &str, name: &str) -> Option<Format> {
    if kind == Kind::Music {
        return None;
    }
    let path = id_path(id).to_lowercase();
    let umf = path.split('/').any(umf_word) || umf_word(&name.to_lowercase());
    Some(if umf { Format::Umf } else { Format::Mugen })
}

pub fn group_key(name: &str) -> String {
    let lower = name.to_lowercase();
    let base = lower.strip_suffix(".zip").unwrap_or(&lower);
    let base = ["_umf", "-umf", " umf"].iter().find_map(|s| base.strip_suffix(s)).or_else(|| base.strip_prefix("umf_")).unwrap_or(base);
    base.chars().filter(|c| c.is_alphanumeric()).collect::<String>().replace("ssj", "ss")
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    pub kind: Option<Kind>,
    pub format: Option<Format>,
    pub outdated_only: bool,
}

impl Filter {
    pub fn accepts(&self, kind: Kind, format: Option<Format>, outdated: bool) -> bool {
        self.kind.is_none_or(|k| k == kind) && self.format.is_none_or(|f| format == Some(f)) && (!self.outdated_only || outdated)
    }

    pub fn is_default(&self) -> bool {
        *self == Filter::default()
    }
}

pub fn next_format(current: Option<Format>) -> Option<Format> {
    match current {
        None => Some(Format::Mugen),
        Some(Format::Mugen) => Some(Format::Umf),
        Some(Format::Umf) => None,
    }
}

pub fn display_key(kind: Kind, name: &str, format: Option<Format>) -> (Kind, String, Option<Format>, String) {
    (kind, group_key(name), format, name.to_lowercase())
}

fn preview_stem(entry: &TreeEntry) -> Option<String> {
    let lower = entry.path.to_lowercase();
    let mut parts: Vec<&str> = lower.split('/').collect();
    let file = parts.pop()?;
    if lower_ext(&entry.path) != "png" || !parts.iter().any(|p| PREVIEW_DIRS.contains(p)) {
        return None;
    }
    Some(file.strip_suffix(".png").unwrap_or(file).to_string())
}

fn matches_group(item: &Item, entry: &TreeEntry) -> bool {
    let first = id_path(&item.id).split('/').next().unwrap_or("").to_lowercase();
    !first.is_empty() && entry.path.to_lowercase().split('/').any(|p| p == first)
}

pub fn portrait_index(catalog: &Catalog) -> HashMap<String, TreeEntry> {
    let mut by_stem: HashMap<String, Vec<&TreeEntry>> = HashMap::new();
    for e in &catalog.support {
        if let Some(stem) = preview_stem(e) {
            by_stem.entry(stem).or_default().push(e);
        }
    }
    if by_stem.is_empty() {
        return HashMap::new();
    }
    let mut out = HashMap::new();
    for item in catalog.items.iter().filter(|i| i.kind != Kind::Music) {
        let Some(candidates) = by_stem.get(&item.name.to_lowercase()) else { continue };
        let pick = candidates.iter().filter(|e| matches_group(item, e)).min_by_key(|e| e.path.len()).or_else(|| candidates.iter().min_by_key(|e| e.path.len()));
        if let Some(entry) = pick {
            out.insert(item.id.clone(), (*entry).clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{detect, Tree};

    fn tree(files: &[&str]) -> Tree {
        Tree { entries: files.iter().map(|p| TreeEntry { path: (*p).into(), size: 9, sha: format!("sha-{p}") }).collect(), truncated: false }
    }

    #[test]
    fn umf_is_recognised_by_folder_or_name_and_music_has_no_format() {
        assert_eq!(format_of(Kind::Pack, "pack:umf/Goku_SSJ4_UMF.zip", "Goku_SSJ4_UMF"), Some(Format::Umf));
        assert_eq!(format_of(Kind::Pack, "pack:packs/Goku_UMF.zip", "Goku_UMF"), Some(Format::Umf));
        assert_eq!(format_of(Kind::Pack, "pack:chars/Goku_SSJ4.zip", "Goku_SSJ4"), Some(Format::Mugen));
        assert_eq!(format_of(Kind::Character, "char:chars/kfm", "kfm"), Some(Format::Mugen));
        assert_eq!(format_of(Kind::Pack, "pack:chars/Dumfries.zip", "Dumfries"), Some(Format::Mugen));
        assert_eq!(format_of(Kind::Music, "music:umf/a.ogg", "a"), None);
    }

    #[test]
    fn variants_of_one_fighter_share_a_group_but_stay_distinct_items() {
        assert_eq!(group_key("Goku_SSJ4_UMF"), group_key("Goku_SSJ4"));
        assert_eq!(group_key("Black Widow"), group_key("black_widow_umf"));
        assert_eq!(group_key("Goku_SS4"), group_key("Goku_SSJ4_UMF"));
        assert_ne!(group_key("Goku"), group_key("Gogeta"));
        let a = display_key(Kind::Pack, "Goku_SSJ4", Some(Format::Mugen));
        let b = display_key(Kind::Pack, "Goku_SSJ4_UMF", Some(Format::Umf));
        assert!(a < b);
    }

    #[test]
    fn filter_combines_kind_format_and_outdated() {
        let f = Filter { format: Some(Format::Umf), outdated_only: true, ..Default::default() };
        assert!(f.accepts(Kind::Pack, Some(Format::Umf), true));
        assert!(!f.accepts(Kind::Pack, Some(Format::Mugen), true));
        assert!(!f.accepts(Kind::Pack, Some(Format::Umf), false));
        assert!(!f.accepts(Kind::Music, None, true));
        assert!(Filter::default().accepts(Kind::Music, None, false));
        assert_eq!(next_format(next_format(next_format(None))), None);
    }

    #[test]
    fn portraits_come_from_preview_folders_matching_name_and_group() {
        let t = tree(&["chars/Aquaman.zip", "umf/Goku_SSJ4_UMF.zip", "chars/Goku_SSJ4_UMF.zip", "previews/chars/Aquaman.png", "previews/umf/Goku_SSJ4_UMF.png", "previews/chars/Goku_SSJ4_UMF.png", "docs/Aquaman.png", "music/theme.ogg"]);
        let c = detect(&t, "", "r");
        let idx = portrait_index(&c);
        assert_eq!(idx["pack:chars/Aquaman.zip"].path, "previews/chars/Aquaman.png");
        assert_eq!(idx["pack:umf/Goku_SSJ4_UMF.zip"].path, "previews/umf/Goku_SSJ4_UMF.png");
        assert_eq!(idx["pack:chars/Goku_SSJ4_UMF.zip"].path, "previews/chars/Goku_SSJ4_UMF.png");
        assert!(!idx.contains_key("music:music/theme.ogg"));
        assert!(portrait_index(&detect(&tree(&["chars/A.zip"]), "", "r")).is_empty());
    }
}
