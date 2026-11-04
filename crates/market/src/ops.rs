use crate::bundle::Files;
use crate::catalog::{Catalog, FileRef, Item, Kind, TreeEntry};
use crate::client::MarketError;
use crate::refs::{followups, TreeIndex};
use crate::safety::{lower_ext, MAX_FILE_BYTES, MAX_ITEM_BYTES, MAX_ITEM_FILES};
use std::collections::HashSet;
use ume_formats::vfs::text;

pub const PARALLEL_DOWNLOADS: usize = 6;
pub const MAX_ROUNDS: usize = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub done_files: usize,
    pub total_files: usize,
    pub done_bytes: u64,
    pub total_bytes: u64,
}

pub struct InstallJob {
    kind: Kind,
    reference: Vec<TreeEntry>,
    have: HashSet<String>,
    got: Files,
    round: usize,
    bytes: u64,
    pending: Vec<FileRef>,
}

impl InstallJob {
    pub fn new(item: &Item, catalog: &Catalog) -> Result<InstallJob, MarketError> {
        if !item.installable {
            return Err(MarketError::TooLarge);
        }
        Ok(InstallJob {
            kind: item.kind,
            reference: catalog.reference_entries(),
            have: item.files.iter().map(|f| f.path.clone()).collect(),
            got: Vec::new(),
            round: 0,
            bytes: 0,
            pending: item.files.clone(),
        })
    }

    pub fn pending(&self) -> &[FileRef] {
        &self.pending
    }

    pub fn accept(&mut self, downloaded: Files) -> Result<(), MarketError> {
        self.round += 1;
        for (path, data) in &downloaded {
            self.bytes += data.len() as u64;
            if data.len() as u64 > MAX_FILE_BYTES || self.bytes > MAX_ITEM_BYTES {
                return Err(MarketError::TooLarge);
            }
            self.have.insert(path.clone());
        }
        let mut next: Vec<FileRef> = Vec::new();
        if matches!(self.kind, Kind::Character | Kind::Stage) && self.round < MAX_ROUNDS {
            let index = TreeIndex::new(&self.reference);
            for (path, data) in downloaded.iter().filter(|(p, _)| lower_ext(p) == "def") {
                for f in followups(path, &text(data), &index, &self.have) {
                    if f.size <= MAX_FILE_BYTES && !next.iter().any(|n| n.path == f.path) {
                        next.push(f);
                    }
                }
            }
        }
        self.got.extend(downloaded);
        if self.got.len() + next.len() > MAX_ITEM_FILES {
            next.clear();
        }
        for f in &next {
            self.have.insert(f.path.clone());
        }
        self.pending = next;
        Ok(())
    }

    pub fn is_done(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn finish(self) -> Files {
        self.got
    }
}

pub fn order_files(wanted: &[FileRef], mut results: Vec<Option<Vec<u8>>>) -> Result<Files, MarketError> {
    let mut out = Vec::with_capacity(wanted.len());
    for (f, r) in wanted.iter().zip(results.drain(..)) {
        out.push((f.path.clone(), r.ok_or_else(|| MarketError::Network(format!("missing {}", f.path)))?));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{detect, Tree};

    fn catalog(files: &[(&str, u64)]) -> Catalog {
        detect(&Tree { entries: files.iter().map(|(p, s)| TreeEntry { path: p.to_string(), size: *s }).collect(), truncated: false }, "", "repo")
    }

    #[test]
    fn stage_install_pulls_referenced_sprites_and_music_in_a_second_round() {
        let c = catalog(&[("stages/arena.def", 50), ("stages/arena.sff", 10), ("shared/bg.sff", 10), ("sound/theme.mp3", 100_000)]);
        let item = c.find("stage:stages/arena.def").unwrap().clone();
        let mut job = InstallJob::new(&item, &c).unwrap();
        assert_eq!(job.pending().len(), 2);
        let def = b"[BGdef]\nspr = shared/bg.sff\n[Music]\nbgmusic = sound/theme.mp3\n".to_vec();
        job.accept(vec![("stages/arena.def".into(), def), ("stages/arena.sff".into(), vec![1])]).unwrap();
        assert!(!job.is_done());
        assert_eq!(job.pending().iter().map(|f| f.path.as_str()).collect::<Vec<_>>(), vec!["shared/bg.sff", "sound/theme.mp3"]);
        job.accept(vec![("shared/bg.sff".into(), vec![2]), ("sound/theme.mp3".into(), vec![3])]).unwrap();
        assert!(job.is_done());
        assert_eq!(job.finish().len(), 4);
    }

    #[test]
    fn music_and_packs_finish_in_one_round() {
        let c = catalog(&[("m/a.ogg", 100_000)]);
        let mut job = InstallJob::new(c.find("music:m/a.ogg").unwrap(), &c).unwrap();
        job.accept(vec![("m/a.ogg".into(), vec![0; 4])]).unwrap();
        assert!(job.is_done());
    }

    #[test]
    fn oversized_items_cannot_start() {
        let c = catalog(&[("big.zip", MAX_FILE_BYTES + 1)]);
        assert_eq!(InstallJob::new(c.find("pack:big.zip").unwrap(), &c).err(), Some(MarketError::TooLarge));
    }

    #[test]
    fn downloaded_bytes_beyond_the_item_budget_abort() {
        let c = catalog(&[("m/a.ogg", 100_000)]);
        let mut job = InstallJob::new(c.find("music:m/a.ogg").unwrap(), &c).unwrap();
        let huge = vec![0u8; (MAX_FILE_BYTES + 1) as usize];
        assert_eq!(job.accept(vec![("m/a.ogg".into(), huge)]), Err(MarketError::TooLarge));
    }

    #[test]
    fn order_files_reports_missing_results() {
        let wanted = vec![FileRef { path: "a".into(), size: 1 }, FileRef { path: "b".into(), size: 1 }];
        assert_eq!(order_files(&wanted, vec![Some(vec![1]), Some(vec![2])]).unwrap(), vec![("a".to_string(), vec![1]), ("b".to_string(), vec![2])]);
        assert!(order_files(&wanted, vec![Some(vec![1]), None]).is_err());
    }
}
