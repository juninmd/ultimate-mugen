use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    pub size: u64,
    #[serde(default)]
    pub sha: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRef {
    pub path: String,
    pub size: u64,
    #[serde(default)]
    pub sha: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub kind: Kind,
    pub name: String,
    pub files: Vec<FileRef>,
    pub bytes: u64,
    pub installable: bool,
    #[serde(default)]
    pub version: String,
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
        let mut out: Vec<TreeEntry> = self
            .items
            .iter()
            .flat_map(|i| {
                i.files.iter().map(|f| TreeEntry {
                    path: f.path.clone(),
                    size: f.size,
                    sha: f.sha.clone(),
                })
            })
            .collect();
        out.extend(self.support.iter().cloned());
        out
    }
}
