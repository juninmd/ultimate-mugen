use crate::catalog::Kind;
use crate::repo::RepoRef;
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoRecord {
    pub key: String,
    pub repo: RepoRef,
    pub etag: Option<String>,
    pub listed_at: u64,
    pub item_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installed {
    pub repo_key: String,
    pub item_id: String,
    pub kind: Kind,
    pub name: String,
    pub bytes: u64,
    pub files: usize,
    pub active: bool,
    pub installed_at: u64,
}

impl Installed {
    pub fn tag(&self) -> String {
        format!("market:{}:{}", self.repo_key, self.item_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketState {
    pub version: u32,
    pub repos: Vec<RepoRecord>,
    pub installed: Vec<Installed>,
    pub token: Option<String>,
}

impl Default for MarketState {
    fn default() -> Self {
        MarketState { version: VERSION, repos: vec![], installed: vec![], token: None }
    }
}

impl MarketState {
    pub fn from_json(bytes: &[u8]) -> MarketState {
        serde_json::from_slice::<MarketState>(bytes).ok().filter(|s| s.version == VERSION).unwrap_or_default()
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap_or_default()
    }

    pub fn add_repo(&mut self, repo: RepoRef) -> bool {
        let key = repo.key();
        if self.repos.iter().any(|r| r.key == key) {
            return false;
        }
        self.repos.push(RepoRecord { key, repo, etag: None, listed_at: 0, item_count: 0 });
        true
    }

    pub fn remove_repo(&mut self, key: &str) -> bool {
        let before = self.repos.len();
        self.repos.retain(|r| r.key != key);
        self.repos.len() != before
    }

    pub fn repo(&self, key: &str) -> Option<&RepoRecord> {
        self.repos.iter().find(|r| r.key == key)
    }

    pub fn repo_mut(&mut self, key: &str) -> Option<&mut RepoRecord> {
        self.repos.iter_mut().find(|r| r.key == key)
    }

    pub fn installed_item(&self, repo_key: &str, item_id: &str) -> Option<&Installed> {
        self.installed.iter().find(|i| i.repo_key == repo_key && i.item_id == item_id)
    }

    pub fn upsert_installed(&mut self, entry: Installed) {
        match self.installed.iter_mut().find(|i| i.repo_key == entry.repo_key && i.item_id == entry.item_id) {
            Some(slot) => {
                let keep_active = slot.active;
                *slot = Installed { active: keep_active || entry.active, ..entry };
            }
            None => self.installed.push(entry),
        }
    }

    pub fn remove_installed(&mut self, repo_key: &str, item_id: &str) -> Option<Installed> {
        let i = self.installed.iter().position(|e| e.repo_key == repo_key && e.item_id == item_id)?;
        Some(self.installed.remove(i))
    }

    pub fn set_active(&mut self, repo_key: &str, item_id: &str, active: bool) -> bool {
        match self.installed.iter_mut().find(|i| i.repo_key == repo_key && i.item_id == item_id) {
            Some(i) if i.active != active => {
                i.active = active;
                true
            }
            _ => false,
        }
    }

    pub fn set_active_many<'a>(&mut self, targets: impl IntoIterator<Item = (&'a str, &'a str)>, active: bool) -> usize {
        targets.into_iter().filter(|(r, i)| self.set_active(r, i, active)).count()
    }

    pub fn active(&self) -> impl Iterator<Item = &Installed> {
        self.installed.iter().filter(|i| i.active)
    }

    pub fn total_bytes(&self) -> u64 {
        self.installed.iter().map(|i| i.bytes).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(repo: &str, id: &str, active: bool) -> Installed {
        Installed { repo_key: repo.into(), item_id: id.into(), kind: Kind::Character, name: id.into(), bytes: 10, files: 2, active, installed_at: 1 }
    }

    #[test]
    fn repos_are_unique_by_key() {
        let mut s = MarketState::default();
        assert!(s.add_repo(RepoRef::parse("A/B").unwrap()));
        assert!(!s.add_repo(RepoRef::parse("a/b").unwrap()));
        assert!(s.add_repo(RepoRef::parse("a/b/tree/dev").unwrap()));
        assert_eq!(s.repos.len(), 2);
        assert!(s.remove_repo("a/b"));
        assert!(!s.remove_repo("a/b"));
    }

    #[test]
    fn json_roundtrip_and_tolerance() {
        let mut s = MarketState::default();
        s.add_repo(RepoRef::parse("a/b").unwrap());
        s.upsert_installed(entry("a/b", "char:x", true));
        s.token = Some("t".into());
        assert_eq!(MarketState::from_json(&s.to_json()), s);
        assert_eq!(MarketState::from_json(b"garbage"), MarketState::default());
        assert_eq!(MarketState::from_json(br#"{"version":99,"repos":[],"installed":[],"token":null}"#), MarketState::default());
    }

    #[test]
    fn bulk_activation_counts_only_changes() {
        let mut s = MarketState::default();
        for id in ["char:a", "char:b", "stage:c"] {
            s.upsert_installed(entry("r", id, false));
        }
        let all: Vec<(&str, &str)> = vec![("r", "char:a"), ("r", "char:b"), ("r", "stage:c"), ("r", "missing")];
        assert_eq!(s.set_active_many(all.iter().copied(), true), 3);
        assert_eq!(s.set_active_many(all.iter().copied(), true), 0);
        assert_eq!(s.active().count(), 3);
        assert_eq!(s.set_active_many([("r", "char:a")], false), 1);
        assert_eq!(s.active().count(), 2);
    }

    #[test]
    fn reinstalling_keeps_activation() {
        let mut s = MarketState::default();
        s.upsert_installed(entry("r", "char:a", true));
        s.upsert_installed(Installed { bytes: 99, ..entry("r", "char:a", false) });
        assert_eq!(s.installed.len(), 1);
        assert!(s.installed[0].active);
        assert_eq!(s.total_bytes(), 99);
        assert_eq!(s.installed[0].tag(), "market:r:char:a");
        assert!(s.remove_installed("r", "char:a").is_some());
        assert!(s.remove_installed("r", "char:a").is_none());
    }
}
