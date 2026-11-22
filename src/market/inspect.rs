use super::*;
use ume_market::catalog::FileRef;
use ume_market::packinfo::PackInfo;
use ume_market::portrait_cache::{Lookup, PortraitCache};
use ume_market::safety::file_stem;

const MAX_INFLIGHT: usize = 2;
const MAX_KNOWN: usize = 4096;

pub struct Probed {
    key: String,
    result: Result<PackInfo, String>,
}

pub type Inspected = PortraitCache<Arc<PackInfo>>;

pub fn new_cache() -> Inspected {
    PortraitCache::new(MAX_INFLIGHT, MAX_KNOWN)
}

pub fn finish(cache: &mut Inspected, probed: Probed) {
    cache.finish(&probed.key, probed.result.map(Arc::new));
}

fn persist_key(repo_key: &str, version: &str) -> String {
    format!("zipinfo/{repo_key}/{version}")
}

fn cached(persist: &Arc<dyn Persist>, key: &str) -> Option<PackInfo> {
    serde_json::from_slice(&persist.read(key)?).ok()
}

fn conclude(persist: &Arc<dyn Persist>, key: &str, file: &FileRef, entries: Vec<ume_market::TreeEntry>) -> PackInfo {
    let info = PackInfo::from_entries(entries, file_stem(&file.path));
    let _ = persist.write(key, &serde_json::to_vec(&info).unwrap_or_default());
    info
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_probe(events: Queue, persist: Arc<dyn Persist>, repo: RepoRef, key: String, store: String, file: FileRef) {
    std::thread::spawn(move || {
        let result = match cached(&persist, &store) {
            Some(info) => Ok(info),
            None => {
                let t = ume_market::native::UreqTransport::default();
                ume_market::probe::probe_zip(&t, &repo, &file).map(|e| conclude(&persist, &store, &file, e)).map_err(|e| describe(&e))
            }
        };
        push(&events, Event::PackInfo(Probed { key, result }));
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_probe(events: Queue, persist: Arc<dyn Persist>, repo: RepoRef, key: String, store: String, file: FileRef) {
    ume_market::web::spawn(async move {
        let result = match cached(&persist, &store) {
            Some(info) => Ok(info),
            None => ume_market::web::probe_zip(&repo, &file).await.map(|e| conclude(&persist, &store, &file, e)).map_err(|e| describe(&e)),
        };
        push(&events, Event::PackInfo(Probed { key, result }));
    });
}

impl Market {
    pub fn pack_info(&mut self, repo_key: &str, item_id: &str) -> Option<Arc<PackInfo>> {
        let item = self.catalog(repo_key)?.find(item_id)?;
        let file = match (item.kind, item.files.as_slice()) {
            (Kind::Pack, [only]) if lower_ext(&only.path) == "zip" => only.clone(),
            _ => return None,
        };
        let version = item.version.clone();
        let key = format!("{repo_key}\u{0}{item_id}\u{0}{version}");
        match self.inspected.lookup(&key) {
            Lookup::Ready(info) => Some(info),
            Lookup::Pending => None,
            Lookup::Start => {
                let repo = self.state.repo(repo_key)?.repo.clone();
                spawn_probe(self.events.clone(), self.persist.clone(), repo, key, persist_key(repo_key, &version), file);
                None
            }
        }
    }
}
