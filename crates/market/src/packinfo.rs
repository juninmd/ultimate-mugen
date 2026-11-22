use crate::catalog::detect;
use crate::model::{Kind, Tree, TreeEntry};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackInfo {
    pub fighters: Vec<String>,
    pub stages: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shape {
    Single(String),
    Multi(usize),
    Stages(usize),
    Empty,
}

impl PackInfo {
    pub fn from_entries(entries: Vec<TreeEntry>, fallback: &str) -> PackInfo {
        let catalog = detect(&Tree { entries, truncated: false }, "", fallback);
        let names = |kind: Kind| catalog.items.iter().filter(|i| i.kind == kind).map(|i| i.name.clone()).collect::<Vec<_>>();
        PackInfo { fighters: names(Kind::Character), stages: names(Kind::Stage) }
    }

    pub fn shape(&self) -> Shape {
        match (self.fighters.len(), self.stages.len()) {
            (1, _) => Shape::Single(self.fighters[0].clone()),
            (0, 0) => Shape::Empty,
            (0, n) => Shape::Stages(n),
            (n, _) => Shape::Multi(n),
        }
    }

    pub fn summary(&self, limit: usize) -> String {
        let mut names: Vec<&str> = self.fighters.iter().map(String::as_str).collect();
        names.sort_unstable_by_key(|n| n.to_lowercase());
        let shown = names.iter().take(limit).copied().collect::<Vec<_>>().join(", ");
        match names.len().saturating_sub(limit) {
            0 => shown,
            more => format!("{shown} +{more}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(paths: &[&str]) -> Vec<TreeEntry> {
        paths.iter().map(|p| TreeEntry { path: (*p).into(), size: 1, sha: String::new() }).collect()
    }

    fn fighter(name: &str) -> Vec<String> {
        ["def", "sff", "air", "cns"].iter().map(|e| format!("{name}/{name}.{e}")).collect()
    }

    #[test]
    fn content_not_name_decides_between_one_fighter_and_a_pack() {
        let one: Vec<String> = fighter("Goku");
        let info = PackInfo::from_entries(entries(&one.iter().map(String::as_str).collect::<Vec<_>>()), "Goku_Pack");
        assert_eq!(info.shape(), Shape::Single("Goku".into()));
        let many: Vec<String> = ["Ryu", "Ken", "Chun"].iter().flat_map(|n| fighter(n)).collect();
        let info = PackInfo::from_entries(entries(&many.iter().map(String::as_str).collect::<Vec<_>>()), "pack");
        assert_eq!(info.shape(), Shape::Multi(3));
        assert_eq!(info.summary(2), "Chun, Ken +1");
        assert_eq!(PackInfo::from_entries(entries(&["readme.txt"]), "x").shape(), Shape::Empty);
        let stage = PackInfo::from_entries(entries(&["stages/a.def", "stages/a.sff"]), "x");
        assert_eq!(stage.shape(), Shape::Stages(1));
    }

    #[test]
    fn a_zip_with_the_fighter_at_its_root_uses_the_archive_name() {
        let info = PackInfo::from_entries(entries(&["k.def", "k.sff", "k.air", "k.cns"]), "Kyo_Kusanagi");
        assert_eq!(info.shape(), Shape::Single("Kyo_Kusanagi".into()));
    }
}
