use crate::safety::{file_name, file_stem, lower_ext, parent_dir, safe_rel_path, MAX_FILE_BYTES, MAX_ITEM_BYTES, MAX_ITEM_FILES, MAX_TREE_ENTRIES};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    Character,
    Stage,
    Music,
    Pack,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Character, Kind::Stage, Kind::Music, Kind::Pack];

    pub fn tag(self) -> &'static str {
        match self {
            Kind::Character => "char",
            Kind::Stage => "stage",
            Kind::Music => "music",
            Kind::Pack => "pack",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    pub size: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRef {
    pub path: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub kind: Kind,
    pub name: String,
    pub files: Vec<FileRef>,
    pub bytes: u64,
    pub installable: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    pub items: Vec<Item>,
    pub truncated: bool,
    #[serde(default)]
    pub support: Vec<TreeEntry>,
}

impl Catalog {
    pub fn count(&self, kind: Kind) -> usize {
        self.items.iter().filter(|i| i.kind == kind).count()
    }

    pub fn find(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn reference_entries(&self) -> Vec<TreeEntry> {
        let mut out: Vec<TreeEntry> = self.items.iter().flat_map(|i| i.files.iter().map(|f| TreeEntry { path: f.path.clone(), size: f.size })).collect();
        out.extend(self.support.iter().cloned());
        out
    }
}

pub fn parse_tree(body: &str) -> Result<Tree, String> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|e| format!("invalid tree response: {e}"))?;
    let list = v.get("tree").and_then(|t| t.as_array()).ok_or("tree response without entries")?;
    if list.len() > MAX_TREE_ENTRIES {
        return Err("repository has too many files".into());
    }
    let mut entries = Vec::with_capacity(list.len());
    for e in list {
        if e.get("type").and_then(|t| t.as_str()) != Some("blob") {
            continue;
        }
        let Some(path) = e.get("path").and_then(|p| p.as_str()).and_then(safe_rel_path) else { continue };
        entries.push(TreeEntry { path, size: e.get("size").and_then(|s| s.as_u64()).unwrap_or(0) });
    }
    Ok(Tree { entries, truncated: v.get("truncated").and_then(|t| t.as_bool()).unwrap_or(false) })
}

const IGNORED_DEF_STEMS: [&str; 12] = ["system", "select", "fight", "fight_ex", "motif", "storyboard", "intro", "ending", "title", "credits", "demo", "screenpack"];
const STAGE_DIRS: [&str; 2] = ["stages", "stage"];
const AUDIO_EXTS: [&str; 4] = ["ogg", "mp3", "flac", "wav"];
const MIN_WAV_BYTES: u64 = 32 << 10;
const SUPPORT_EXTS: [&str; 14] = ["def", "cns", "cmd", "air", "sff", "snd", "act", "st", "zss", "ogg", "mp3", "flac", "wav", "png"];
const MAX_SUPPORT_ENTRIES: usize = 6000;

#[derive(Default, Clone, Copy)]
struct Counts {
    sff: u32,
    air: u32,
    cns: u32,
    cmd: u32,
}

fn hidden(path: &str) -> bool {
    path.split('/').any(|s| s.starts_with('.') || s == "__MACOSX" || s == "node_modules")
}

fn ancestors(path: &str) -> impl Iterator<Item = &str> {
    let mut cur = Some(parent_dir(path));
    std::iter::from_fn(move || {
        let c = cur?;
        cur = if c.is_empty() { None } else { Some(parent_dir(c)) };
        Some(c)
    })
}

fn ignored_def(path: &str) -> bool {
    let stem = file_stem(path).to_ascii_lowercase();
    IGNORED_DEF_STEMS.contains(&stem.as_str())
}

fn make_item(kind: Kind, root: &str, name: String, files: Vec<FileRef>) -> Item {
    let bytes: u64 = files.iter().map(|f| f.size).sum();
    let installable = !files.is_empty() && bytes <= MAX_ITEM_BYTES && files.len() <= MAX_ITEM_FILES && files.iter().all(|f| f.size <= MAX_FILE_BYTES);
    Item { id: format!("{}:{}", kind.tag(), if root.is_empty() { "." } else { root }), kind, name, files, bytes, installable }
}

pub fn detect(tree: &Tree, subdir: &str, fallback_name: &str) -> Catalog {
    let prefix = if subdir.is_empty() { String::new() } else { format!("{}/", subdir.trim_end_matches('/')) };
    let scope: Vec<&TreeEntry> = tree.entries.iter().filter(|e| e.path.starts_with(&prefix) && !hidden(&e.path[prefix.len()..])).collect();
    let root_dir = prefix.trim_end_matches('/').to_string();

    let mut subtree: HashMap<&str, Counts> = HashMap::new();
    let mut direct_defs: HashMap<&str, u32> = HashMap::new();
    for e in &scope {
        let ext = lower_ext(&e.path);
        if ext == "def" && !ignored_def(&e.path) {
            *direct_defs.entry(parent_dir(&e.path)).or_default() += 1;
        }
        for dir in ancestors(&e.path) {
            if !dir.starts_with(&root_dir) {
                break;
            }
            let c = subtree.entry(dir).or_default();
            match ext.as_str() {
                "sff" => c.sff += 1,
                "air" => c.air += 1,
                "cns" => c.cns += 1,
                "cmd" => c.cmd += 1,
                _ => {}
            }
        }
    }

    let char_dirs: HashSet<&str> = direct_defs
        .keys()
        .copied()
        .filter(|d| subtree.get(d).is_some_and(|c| c.sff > 0 && c.air > 0 && (c.cns > 0 || c.cmd > 0)))
        .collect();

    let mut claimed: HashMap<&str, &str> = HashMap::new();
    let mut char_files: HashMap<&str, Vec<FileRef>> = HashMap::new();
    for e in &scope {
        if let Some(dir) = ancestors(&e.path).find(|d| char_dirs.contains(d)) {
            claimed.insert(e.path.as_str(), dir);
            char_files.entry(dir).or_default().push(FileRef { path: e.path.clone(), size: e.size });
        }
    }

    let mut items = Vec::new();
    for (dir, files) in char_files {
        let name = if dir.is_empty() || dir == root_dir { fallback_name.to_string() } else { file_name(dir).to_string() };
        items.push(make_item(Kind::Character, dir, name, files));
    }

    let unclaimed: Vec<&TreeEntry> = scope.iter().copied().filter(|e| !claimed.contains_key(e.path.as_str())).collect();
    let lower_index: HashSet<String> = unclaimed.iter().map(|e| e.path.to_ascii_lowercase()).collect();
    let mut stage_claimed: HashSet<&str> = HashSet::new();
    let mut stage_items = Vec::new();
    let defs_per_dir: HashMap<&str, usize> = unclaimed.iter().filter(|e| lower_ext(&e.path) == "def" && !ignored_def(&e.path)).fold(HashMap::new(), |mut m, e| {
        *m.entry(parent_dir(&e.path)).or_default() += 1;
        m
    });
    for e in unclaimed.iter().filter(|e| lower_ext(&e.path) == "def" && !ignored_def(&e.path)) {
        let dir = parent_dir(&e.path);
        let stem = file_stem(&e.path);
        let sibling_sff = lower_index.contains(&format!("{}{}.sff", if dir.is_empty() { String::new() } else { format!("{dir}/") }, stem).to_ascii_lowercase());
        let in_stage_dir = e.path.split('/').any(|s| STAGE_DIRS.contains(&s.to_ascii_lowercase().as_str()));
        if !sibling_sff && !in_stage_dir {
            continue;
        }
        let dir_is_stage_folder = STAGE_DIRS.contains(&file_name(dir).to_ascii_lowercase().as_str());
        let own_folder = !dir_is_stage_folder && dir != root_dir && !dir.is_empty() && defs_per_dir.get(dir) == Some(&1);
        let files: Vec<FileRef> = if own_folder {
            let dp = format!("{dir}/");
            unclaimed.iter().filter(|x| x.path.starts_with(&dp) && !stage_claimed.contains(x.path.as_str())).map(|x| FileRef { path: x.path.clone(), size: x.size }).collect()
        } else {
            unclaimed
                .iter()
                .filter(|x| parent_dir(&x.path) == dir && file_stem(&x.path).eq_ignore_ascii_case(stem) && !stage_claimed.contains(x.path.as_str()))
                .map(|x| FileRef { path: x.path.clone(), size: x.size })
                .collect()
        };
        if files.is_empty() {
            continue;
        }
        let root = if own_folder { dir.to_string() } else { e.path.clone() };
        let name = if own_folder { file_name(dir).to_string() } else { stem.to_string() };
        for f in &files {
            if let Some(orig) = unclaimed.iter().find(|x| x.path == f.path) {
                stage_claimed.insert(orig.path.as_str());
            }
        }
        stage_items.push(make_item(Kind::Stage, &root, name, files));
    }
    items.extend(stage_items);

    for e in unclaimed.iter().filter(|e| !stage_claimed.contains(e.path.as_str())) {
        let ext = lower_ext(&e.path);
        if ext == "zip" {
            items.push(make_item(Kind::Pack, &e.path, file_stem(&e.path).to_string(), vec![FileRef { path: e.path.clone(), size: e.size }]));
        } else if AUDIO_EXTS.contains(&ext.as_str()) && (ext != "wav" || e.size >= MIN_WAV_BYTES) {
            items.push(make_item(Kind::Music, &e.path, file_stem(&e.path).to_string(), vec![FileRef { path: e.path.clone(), size: e.size }]));
        }
    }

    for it in &mut items {
        it.files.sort_by(|a, b| a.path.cmp(&b.path));
    }
    items.sort_by(|a, b| (a.kind, a.name.to_lowercase(), &a.id).cmp(&(b.kind, b.name.to_lowercase(), &b.id)));
    items.dedup_by(|a, b| a.id == b.id);
    let used: HashSet<&str> = items.iter().flat_map(|i| i.files.iter().map(|f| f.path.as_str())).collect();
    let support: Vec<TreeEntry> = scope.iter().filter(|e| !used.contains(e.path.as_str()) && SUPPORT_EXTS.contains(&lower_ext(&e.path).as_str())).take(MAX_SUPPORT_ENTRIES).map(|e| (*e).clone()).collect();
    Catalog { items, truncated: tree.truncated, support }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(files: &[(&str, u64)]) -> Tree {
        Tree { entries: files.iter().map(|(p, s)| TreeEntry { path: p.to_string(), size: *s }).collect(), truncated: false }
    }

    fn ids(c: &Catalog) -> Vec<String> {
        c.items.iter().map(|i| i.id.clone()).collect()
    }

    #[test]
    fn parses_github_tree_response_keeping_only_safe_blobs() {
        let body = r#"{"sha":"x","truncated":true,"tree":[
            {"path":"chars","type":"tree"},
            {"path":"chars/kfm/kfm.def","type":"blob","size":120},
            {"path":"../evil","type":"blob","size":1},
            {"path":"a\\b","type":"blob","size":1},
            {"path":"sub","type":"commit"}]}"#;
        let t = parse_tree(body).unwrap();
        assert!(t.truncated);
        assert_eq!(t.entries, vec![TreeEntry { path: "chars/kfm/kfm.def".into(), size: 120 }]);
        assert!(parse_tree("{").is_err());
        assert!(parse_tree("{}").is_err());
    }

    #[test]
    fn detects_characters_stages_music_and_packs() {
        let t = tree(&[
            ("chars/kfm/kfm.def", 100), ("chars/kfm/kfm.sff", 5000), ("chars/kfm/kfm.air", 300), ("chars/kfm/kfm.cmd", 200), ("chars/kfm/kfm.cns", 900), ("chars/kfm/sound/kfm.snd", 40),
            ("stages/arena.def", 60), ("stages/arena.sff", 7000), ("stages/other.def", 60), ("stages/other.sff", 8000),
            ("music/theme.ogg", 4_000_000), ("music/click.wav", 100), ("music/loop.WAV", 90_000),
            ("packs/goku.zip", 9_000_000), ("README.md", 10), (".github/workflows/x.yml", 10),
        ]);
        let c = detect(&t, "", "repo");
        assert_eq!(ids(&c), vec!["char:chars/kfm", "stage:stages/arena.def", "stage:stages/other.def", "music:music/loop.WAV", "music:music/theme.ogg", "pack:packs/goku.zip"]);
        let kfm = c.find("char:chars/kfm").unwrap();
        assert_eq!((kfm.name.as_str(), kfm.files.len(), kfm.bytes), ("kfm", 6, 6540));
        let arena = c.find("stage:stages/arena.def").unwrap();
        assert_eq!(arena.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(), vec!["stages/arena.def", "stages/arena.sff"]);
        assert!(c.items.iter().all(|i| i.installable));
    }

    #[test]
    fn character_files_are_not_listed_again_as_music_or_stages() {
        let t = tree(&[("kfm/kfm.def", 1), ("kfm/kfm.sff", 1), ("kfm/kfm.air", 1), ("kfm/kfm.cns", 1), ("kfm/bgm.ogg", 500_000), ("kfm/extra/stage.def", 1), ("kfm/extra/stage.sff", 1)]);
        let c = detect(&t, "", "repo");
        assert_eq!(ids(&c), vec!["char:kfm"]);
        assert_eq!(c.items[0].files.len(), 7);
    }

    #[test]
    fn nested_characters_split_and_screenpack_defs_are_ignored() {
        let t = tree(&[
            ("select.def", 1), ("system.def", 1),
            ("chars/a/a.def", 1), ("chars/a/a.sff", 1), ("chars/a/a.air", 1), ("chars/a/a.cmd", 1),
            ("chars/b/b.def", 1), ("chars/b/b.sff", 1), ("chars/b/b.air", 1), ("chars/b/b.cns", 1),
        ]);
        let c = detect(&t, "", "repo");
        assert_eq!(ids(&c), vec!["char:chars/a", "char:chars/b"]);
    }

    #[test]
    fn repository_that_is_a_single_character_uses_the_repo_name() {
        let t = tree(&[("goku.def", 1), ("goku.sff", 1), ("goku.air", 1), ("goku.cns", 1), ("readme.md", 1)]);
        let c = detect(&t, "", "Goku-Pack");
        assert_eq!(c.items.len(), 1);
        assert_eq!((c.items[0].id.as_str(), c.items[0].name.as_str(), c.items[0].files.len()), ("char:.", "Goku-Pack", 5));
    }

    #[test]
    fn subdir_limits_the_scan_and_hidden_folders_are_skipped() {
        let t = tree(&[("a/stages/x.def", 1), ("a/stages/x.sff", 1), ("b/stages/y.def", 1), ("b/stages/y.sff", 1), ("a/.git/z.ogg", 99999999)]);
        assert_eq!(ids(&detect(&t, "a", "r")), vec!["stage:a/stages/x.def"]);
        assert_eq!(ids(&detect(&t, "", "r")).len(), 2);
    }

    #[test]
    fn stage_in_its_own_folder_takes_the_whole_folder() {
        let t = tree(&[("cenarios/dojo/dojo.def", 1), ("cenarios/dojo/dojo.sff", 1), ("cenarios/dojo/bgm.mp3", 100_000), ("cenarios/dojo/fx/a.png", 1)]);
        let c = detect(&t, "", "r");
        assert_eq!(ids(&c), vec!["stage:cenarios/dojo"]);
        assert_eq!(c.items[0].files.len(), 4);
        assert_eq!(c.items[0].name, "dojo");
    }

    #[test]
    fn oversized_items_are_flagged_not_installable() {
        let t = tree(&[("big.zip", MAX_FILE_BYTES + 1), ("ok.zip", 10)]);
        let c = detect(&t, "", "r");
        assert!(!c.find("pack:big.zip").unwrap().installable);
        assert!(c.find("pack:ok.zip").unwrap().installable);
    }
}
