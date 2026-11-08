use crate::catalog::{FileRef, TreeEntry};
use crate::safety::parent_dir;
use std::collections::{HashMap, HashSet};
use ume_formats::ini::{find_section, parse_ini, unquote};

pub struct TreeIndex<'a> {
    by_lower: HashMap<String, &'a TreeEntry>,
}

impl<'a> TreeIndex<'a> {
    pub fn new(entries: &'a [TreeEntry]) -> Self {
        let mut by_lower = HashMap::with_capacity(entries.len());
        for e in entries {
            by_lower.entry(e.path.to_ascii_lowercase()).or_insert(e);
        }
        TreeIndex { by_lower }
    }

    fn get(&self, path: &str) -> Option<&'a TreeEntry> {
        self.by_lower.get(&path.to_ascii_lowercase()).copied()
    }
}

pub fn declared_files(def_text: &str) -> Vec<String> {
    let secs = parse_ini(def_text);
    let mut out = Vec::new();
    if let Some(files) = find_section(&secs, "files") {
        out.extend(files.entries.iter().map(|(_, v)| unquote(v)));
    }
    if let Some(bg) = find_section(&secs, "bgdef") {
        out.extend(bg.get("spr").map(unquote));
    }
    if let Some(m) = find_section(&secs, "music") {
        out.extend(m.get("bgmusic").map(unquote));
    }
    out.retain(|n| !n.is_empty());
    out
}

fn clean_relative(name: &str) -> Option<String> {
    let n = name.replace('\\', "/");
    let n = n.trim().trim_start_matches("./");
    if n.is_empty()
        || n.starts_with('/')
        || n.split('/').any(|s| s.is_empty() || s == ".." || s == ".")
    {
        return None;
    }
    Some(n.to_string())
}

pub fn resolve_reference<'a>(
    index: &TreeIndex<'a>,
    def_path: &str,
    name: &str,
) -> Option<&'a TreeEntry> {
    let n = clean_relative(name)?;
    let dir = parent_dir(def_path);
    let relative = if dir.is_empty() {
        n.clone()
    } else {
        format!("{dir}/{n}")
    };
    [relative, n.clone(), format!("data/{n}")]
        .iter()
        .find_map(|c| index.get(c))
}

pub fn followups(
    def_path: &str,
    def_text: &str,
    index: &TreeIndex,
    have: &HashSet<String>,
) -> Vec<FileRef> {
    let mut out: Vec<FileRef> = Vec::new();
    for name in declared_files(def_text) {
        let Some(e) = resolve_reference(index, def_path, &name) else {
            continue;
        };
        if have.contains(&e.path) || out.iter().any(|f| f.path == e.path) {
            continue;
        }
        out.push(FileRef {
            path: e.path.clone(),
            size: e.size,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(paths: &[&str]) -> Vec<TreeEntry> {
        paths
            .iter()
            .map(|p| TreeEntry {
                path: p.to_string(),
                size: 10,
            })
            .collect()
    }

    #[test]
    fn reads_declared_files_from_character_and_stage_defs() {
        let c = declared_files(
            "[Files]\ncmd = kfm.cmd\nstcommon = \"common1.cns\"\n[Info]\nname = x\n",
        );
        assert_eq!(c, vec!["kfm.cmd", "common1.cns"]);
        let s =
            declared_files("[BGdef]\nspr = stages/arena.sff\n[Music]\nbgmusic = sound/arena.mp3\n");
        assert_eq!(s, vec!["stages/arena.sff", "sound/arena.mp3"]);
    }

    #[test]
    fn stage_followups_find_sprites_and_music_outside_the_stage_files() {
        let e = entries(&[
            "stages/arena.def",
            "stages/Arena_BG.sff",
            "sound/Arena.MP3",
            "data/common1.cns",
        ]);
        let idx = TreeIndex::new(&e);
        let have: HashSet<String> = ["stages/arena.def".to_string()].into();
        let f = followups(
            "stages/arena.def",
            "[BGdef]\nspr = arena_bg.sff\n[Music]\nbgmusic = sound/arena.mp3\n",
            &idx,
            &have,
        );
        assert_eq!(
            f.iter().map(|x| x.path.as_str()).collect::<Vec<_>>(),
            vec!["stages/Arena_BG.sff", "sound/Arena.MP3"]
        );
    }

    #[test]
    fn character_followups_pull_shared_common_files_once() {
        let e = entries(&["chars/a/a.def", "chars/a/a.cmd", "data/common1.cns"]);
        let idx = TreeIndex::new(&e);
        let have: HashSet<String> = ["chars/a/a.def".into(), "chars/a/a.cmd".into()].into();
        let f = followups(
            "chars/a/a.def",
            "[Files]\ncmd = a.cmd\nstcommon = common1.cns\nst = common1.cns\n",
            &idx,
            &have,
        );
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].path, "data/common1.cns");
    }

    #[test]
    fn references_never_escape_the_repository() {
        let e = entries(&["secret.txt", "a/b.def"]);
        let idx = TreeIndex::new(&e);
        assert!(resolve_reference(&idx, "a/b.def", "../secret.txt").is_none());
        assert!(resolve_reference(&idx, "a/b.def", "/secret.txt").is_none());
        assert!(resolve_reference(&idx, "a/b.def", "secret.txt").is_some());
    }
}
