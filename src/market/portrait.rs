use super::*;
use ume_market::catalog::{FileRef, TreeEntry};
use ume_market::portrait_cache::{Lookup, PortraitCache};
use ume_market::view::portrait_index;

const MAX_INFLIGHT: usize = 3;
const MAX_READY: usize = 48;

pub struct Art {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<Vec<u8>>,
}

pub struct Loaded {
    key: String,
    result: Result<Art, String>,
}

pub type Portraits = PortraitCache<Arc<Art>>;

pub fn new_cache() -> Portraits {
    PortraitCache::new(MAX_INFLIGHT, MAX_READY)
}

pub fn finish(cache: &mut Portraits, loaded: Loaded) {
    cache.finish(&loaded.key, loaded.result.map(Arc::new));
}

pub(super) fn shrink(rgba: &[u8], w: usize, h: usize) -> (usize, usize, Vec<u8>) {
    let (nw, nh) = (w / 2, h / 2);
    let mut out = vec![0u8; nw * nh * 4];
    for y in 0..nh {
        for x in 0..nw {
            for c in 0..4 {
                let at = |dx: usize, dy: usize| rgba[((y * 2 + dy) * w + x * 2 + dx) * 4 + c] as u32;
                out[(y * nw + x) * 4 + c] = ((at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1)) / 4) as u8;
            }
        }
    }
    (nw, nh, out)
}

fn decode_art(bytes: &[u8]) -> Result<Art, String> {
    let png = ume_formats::png::decode_png(bytes)?;
    let rgba = match png.rgba.clone() {
        Some(rgba) => rgba,
        None => ume_formats::png::png_indexed_to_rgba(&png),
    };
    let (w, h) = (png.width as usize, png.height as usize);
    if w == 0 || h == 0 || rgba.len() != w * h * 4 {
        return Err("invalid portrait".into());
    }
    let (w, h, rgba) = if w >= 400 { shrink(&rgba, w, h) } else { (w, h, rgba) };
    Ok(Art { width: w as u32, height: h as u32, rgba: Arc::new(rgba) })
}

fn file_ref(entry: &TreeEntry) -> FileRef {
    FileRef { path: entry.path.clone(), size: entry.size, sha: entry.sha.clone() }
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_portrait(events: Queue, persist: Arc<dyn Persist>, repo: RepoRef, key: String, entry: TreeEntry) {
    std::thread::spawn(move || {
        let cache = format!("portrait/{}/{}", repo.key(), entry.sha);
        let cached = if entry.sha.is_empty() { None } else { persist.read(&cache) };
        let bytes = match cached {
            Some(b) => Ok(b),
            None => {
                let t = ume_market::native::UreqTransport::default();
                let got = ume_market::native::download_files(&t, &repo, &[file_ref(&entry)], &AtomicBool::new(false), &|_| {});
                got.map_err(|e| describe(&e)).and_then(|f| f.into_iter().next().map(|(_, d)| d).ok_or_else(|| "empty".to_string()))
            }
        };
        if let (Ok(b), false) = (&bytes, entry.sha.is_empty()) {
            let _ = persist.write(&cache, b);
        }
        let result = bytes.and_then(|b| decode_art(&b));
        push(&events, Event::Portrait(Loaded { key, result }));
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_portrait(events: Queue, _persist: Arc<dyn Persist>, repo: RepoRef, key: String, entry: TreeEntry) {
    ume_market::web::spawn(async move {
        let got = ume_market::web::download_files(&repo, &[file_ref(&entry)], &AtomicBool::new(false), &|_| {}).await;
        let bytes = got.map_err(|e| describe(&e)).and_then(|f| f.into_iter().next().map(|(_, d)| d).ok_or_else(|| "empty".to_string()));
        let result = bytes.and_then(|b| decode_art(&b));
        push(&events, Event::Portrait(Loaded { key, result }));
    });
}

impl Market {
    fn portrait_entry(&mut self, repo_key: &str, item_id: &str) -> Option<TreeEntry> {
        if self.portraits.index(repo_key).is_none() {
            let built = portrait_index(self.catalog(repo_key)?);
            self.portraits.set_index(repo_key, built);
        }
        self.portraits.index(repo_key)?.get(item_id).cloned()
    }

    pub fn portrait(&mut self, repo_key: &str, item_id: &str) -> Option<Arc<Art>> {
        let entry = self.portrait_entry(repo_key, item_id)?;
        let key = format!("{repo_key}\u{0}{}\u{0}{}", entry.path, entry.sha);
        match self.portraits.lookup(&key) {
            Lookup::Ready(art) => Some(art),
            Lookup::Pending => None,
            Lookup::Start => {
                let repo = self.state.repo(repo_key)?.repo.clone();
                spawn_portrait(self.events.clone(), self.persist.clone(), repo, key, entry);
                None
            }
        }
    }
}
