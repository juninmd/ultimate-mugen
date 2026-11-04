use crate::bundle::Files;
use crate::catalog::{detect, Catalog, FileRef, Item};
use crate::client::{check_request, file_request, interpret_file, interpret_tree, is_transient, redirect_target, tree_request, MarketError, RateLimit, Request, Response, TreeOutcome};
use crate::ops::{InstallJob, Progress, PARALLEL_DOWNLOADS};
use crate::persist::Persist;
use crate::repo::RepoRef;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

const MAX_REDIRECTS: usize = 3;
const ATTEMPTS: usize = 3;

pub trait Transport: Send + Sync {
    fn get(&self, req: &Request) -> Result<Response, MarketError>;
}

pub struct UreqTransport {
    agent: ureq::Agent,
}

impl Default for UreqTransport {
    fn default() -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .max_redirects(0)
            .max_redirects_will_error(false)
            .http_status_as_error(false)
            .user_agent("ultimate-mugen-market")
            .timeout_global(Some(Duration::from_secs(120)))
            .timeout_connect(Some(Duration::from_secs(10)))
            .build()
            .into();
        UreqTransport { agent }
    }
}

fn header(resp: &ureq::http::Response<ureq::Body>, name: &str) -> Option<String> {
    resp.headers().get(name).and_then(|v| v.to_str().ok()).map(str::to_string)
}

impl UreqTransport {
    fn once(&self, url: &str, req: &Request) -> Result<Response, MarketError> {
        let mut call = self.agent.get(url);
        if url.starts_with("https://api.github.com/") {
            call = call.header("Accept", "application/vnd.github+json");
            if let Some(e) = &req.etag {
                call = call.header("If-None-Match", e);
            }
            if let Some(t) = &req.token {
                call = call.header("Authorization", &format!("Bearer {t}"));
            }
        }
        let mut resp = call.call().map_err(|e| MarketError::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        let rate = match (header(&resp, "x-ratelimit-remaining").and_then(|v| v.parse().ok()), header(&resp, "x-ratelimit-reset").and_then(|v| v.parse().ok())) {
            (Some(remaining), Some(reset_epoch)) => Some(RateLimit { remaining, reset_epoch }),
            _ => None,
        };
        let out = Response { status, etag: header(&resp, "etag"), body: Vec::new(), rate, retry_after_secs: header(&resp, "retry-after").and_then(|v| v.parse().ok()), location: header(&resp, "location") };
        if status != 200 {
            return Ok(out);
        }
        let body = resp.body_mut().with_config().limit(req.max_bytes).read_to_vec().map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(_) => MarketError::TooLarge,
            other => MarketError::Network(other.to_string()),
        })?;
        Ok(Response { body, ..out })
    }
}

impl Transport for UreqTransport {
    fn get(&self, req: &Request) -> Result<Response, MarketError> {
        check_request(req)?;
        let mut url = req.url.clone();
        for _ in 0..=MAX_REDIRECTS {
            let resp = self.once(&url, req)?;
            if matches!(resp.status, 301 | 302 | 303 | 307 | 308) {
                let loc = resp.location.clone().ok_or_else(|| MarketError::Invalid("redirect without location".into()))?;
                url = redirect_target(&url, &loc)?;
                continue;
            }
            return Ok(resp);
        }
        Err(MarketError::Network("too many redirects".into()))
    }
}

fn fetch_retry(t: &dyn Transport, req: &Request, cancel: &AtomicBool) -> Result<Response, MarketError> {
    let mut last = MarketError::Cancelled;
    for attempt in 0..ATTEMPTS {
        if cancel.load(Ordering::Relaxed) {
            return Err(MarketError::Cancelled);
        }
        match t.get(req) {
            Ok(r) if (500..600).contains(&r.status) => last = MarketError::Http(r.status),
            Ok(r) => return Ok(r),
            Err(e) if is_transient(&e) => last = e,
            Err(e) => return Err(e),
        }
        std::thread::sleep(Duration::from_millis(300 * (attempt as u64 + 1)));
    }
    Err(last)
}

pub enum ListOutcome {
    NotModified,
    Fresh { catalog: Catalog, etag: Option<String> },
}

pub fn list_repo(t: &dyn Transport, repo: &RepoRef, etag: Option<&str>, token: Option<&str>) -> Result<ListOutcome, MarketError> {
    let resp = fetch_retry(t, &tree_request(repo, etag, token), &AtomicBool::new(false))?;
    match interpret_tree(&resp)? {
        TreeOutcome::NotModified => Ok(ListOutcome::NotModified),
        TreeOutcome::Fresh { tree, etag } => Ok(ListOutcome::Fresh { catalog: detect(&tree, &repo.subdir, &repo.name), etag }),
    }
}

pub fn download_files(t: &dyn Transport, repo: &RepoRef, wanted: &[FileRef], cancel: &AtomicBool, on_file: &(dyn Fn(u64) + Sync)) -> Result<Files, MarketError> {
    let next = AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<Vec<u8>>>> = Mutex::new(vec![None; wanted.len()]);
    let failure: Mutex<Option<MarketError>> = Mutex::new(None);
    let workers = PARALLEL_DOWNLOADS.min(wanted.len().max(1));
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                if cancel.load(Ordering::Relaxed) || failure.lock().map(|f| f.is_some()).unwrap_or(true) {
                    return;
                }
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(f) = wanted.get(i) else { return };
                let result = fetch_retry(t, &file_request(repo, &f.path, f.size.max(1).saturating_mul(2).max(1 << 20)), cancel).and_then(|r| interpret_file(&r, f.size.saturating_mul(2).max(1 << 20)));
                match result {
                    Ok(data) => {
                        on_file(data.len() as u64);
                        if let Ok(mut s) = slots.lock() {
                            s[i] = Some(data);
                        }
                    }
                    Err(e) => {
                        if let Ok(mut fail) = failure.lock() {
                            fail.get_or_insert(e);
                        }
                        return;
                    }
                }
            });
        }
    });
    if let Some(e) = failure.into_inner().ok().flatten() {
        return Err(e);
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(MarketError::Cancelled);
    }
    crate::ops::order_files(wanted, slots.into_inner().map_err(|_| MarketError::Cancelled)?)
}

pub fn install_item(t: &dyn Transport, repo: &RepoRef, item: &Item, catalog: &Catalog, cancel: &AtomicBool, progress: &(dyn Fn(Progress) + Sync)) -> Result<Files, MarketError> {
    let mut job = InstallJob::new(item, catalog)?;
    let done_files = AtomicUsize::new(0);
    let done_bytes = AtomicU64::new(0);
    let mut total_files = 0usize;
    let mut total_bytes = 0u64;
    while !job.is_done() {
        let wanted: Vec<FileRef> = job.pending().to_vec();
        total_files += wanted.len();
        total_bytes += wanted.iter().map(|f| f.size).sum::<u64>();
        let (tf, tb) = (total_files, total_bytes);
        let downloaded = download_files(t, repo, &wanted, cancel, &|n| {
            let f = done_files.fetch_add(1, Ordering::Relaxed) + 1;
            let b = done_bytes.fetch_add(n, Ordering::Relaxed) + n;
            progress(Progress { done_files: f, total_files: tf, done_bytes: b, total_bytes: tb });
        })?;
        job.accept(downloaded)?;
    }
    Ok(job.finish())
}

pub struct FsPersist {
    root: PathBuf,
}

fn escape_segment(seg: &str) -> String {
    let mut out = String::with_capacity(seg.len());
    for (i, b) in seg.bytes().enumerate() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || (b == b'.' && i > 0 && !out.ends_with('.')) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    if out.len() > 120 {
        let hash = seg.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
        out.truncate(80);
        out.push_str(&format!("~{hash:016x}"));
    }
    if out.is_empty() { "%00".into() } else { out }
}

impl FsPersist {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        FsPersist { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path_for(&self, key: &str) -> PathBuf {
        match key.split_once('/') {
            Some((kind, rest)) => self.root.join(escape_segment(kind)).join(escape_segment(rest)),
            None => self.root.join(escape_segment(key)),
        }
    }
}

impl Persist for FsPersist {
    fn read(&self, key: &str) -> Option<Vec<u8>> {
        std::fs::read(self.path_for(key)).ok()
    }

    fn write(&self, key: &str, data: &[u8]) -> Result<(), String> {
        let path = self.path_for(key);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, data).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }

    fn delete(&self, key: &str) {
        let _ = std::fs::remove_file(self.path_for(key));
    }
}

pub fn default_root() -> PathBuf {
    if let Ok(p) = std::env::var("UME_MARKET_HOME") {
        if !p.is_empty() {
            return p.into();
        }
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        home.map(|h| h.join("Library").join("Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).filter(|p| p.is_absolute()).or_else(|| home.map(|h| h.join(".local").join("share")))
    };
    base.unwrap_or_else(|| PathBuf::from(".")).join("ultimate-mugen").join("market")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Kind;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicUsize;

    struct Fake {
        files: HashMap<String, Vec<u8>>,
        tree: String,
        calls: AtomicUsize,
        fail_first: AtomicUsize,
    }

    impl Transport for Fake {
        fn get(&self, req: &Request) -> Result<Response, MarketError> {
            check_request(req)?;
            self.calls.fetch_add(1, Ordering::Relaxed);
            if req.url.contains("/git/trees/") {
                if req.etag.as_deref() == Some("\"v1\"") {
                    return Ok(Response { status: 304, ..Default::default() });
                }
                return Ok(Response { status: 200, etag: Some("\"v1\"".into()), body: self.tree.clone().into_bytes(), ..Default::default() });
            }
            if self.fail_first.load(Ordering::Relaxed) > 0 {
                self.fail_first.fetch_sub(1, Ordering::Relaxed);
                return Err(MarketError::Network("boom".into()));
            }
            let path = req.url.split("/HEAD/").nth(1).unwrap_or("").replace("%20", " ");
            match self.files.get(&path) {
                Some(d) => Ok(Response { status: 200, body: d.clone(), ..Default::default() }),
                None => Ok(Response { status: 404, ..Default::default() }),
            }
        }
    }

    fn fake() -> Fake {
        let mut files = HashMap::new();
        files.insert("chars/kfm/kfm.def".to_string(), b"[Files]\ncmd = kfm.cmd\nsprite = kfm.sff\n".to_vec());
        for (p, n) in [("chars/kfm/kfm.sff", 8), ("chars/kfm/kfm.air", 2), ("chars/kfm/kfm.cmd", 2), ("chars/kfm/kfm.cns", 2), ("music/a b.ogg", 4)] {
            files.insert(p.to_string(), vec![7; n]);
        }
        let entries: Vec<String> = files.iter().map(|(p, d)| format!(r#"{{"path":"{p}","type":"blob","size":{}}}"#, d.len())).collect();
        Fake { files, tree: format!(r#"{{"tree":[{}],"truncated":false}}"#, entries.join(",")), calls: AtomicUsize::new(0), fail_first: AtomicUsize::new(0) }
    }

    #[test]
    fn lists_with_etag_and_installs_in_parallel() {
        let t = fake();
        let repo = RepoRef::parse("a/b").unwrap();
        let (catalog, etag) = match list_repo(&t, &repo, None, None).unwrap() {
            ListOutcome::Fresh { catalog, etag } => (catalog, etag),
            ListOutcome::NotModified => panic!("expected fresh"),
        };
        assert_eq!(etag.as_deref(), Some("\"v1\""));
        assert!(matches!(list_repo(&t, &repo, etag.as_deref(), None).unwrap(), ListOutcome::NotModified));
        let kfm = catalog.items.iter().find(|i| i.kind == Kind::Character).unwrap();
        let seen = AtomicUsize::new(0);
        let files = install_item(&t, &repo, kfm, &catalog, &AtomicBool::new(false), &|p| {
            seen.store(p.done_files, Ordering::Relaxed);
        })
        .unwrap();
        assert_eq!(files.len(), 5);
        assert_eq!(seen.load(Ordering::Relaxed), 5);
        assert_eq!(catalog.items.iter().filter(|i| i.kind == Kind::Music).count(), 1);
    }

    #[test]
    fn transient_failures_are_retried_and_missing_files_fail() {
        let t = fake();
        t.fail_first.store(2, Ordering::Relaxed);
        let repo = RepoRef::parse("a/b").unwrap();
        let want = vec![FileRef { path: "chars/kfm/kfm.sff".into(), size: 8 }];
        assert_eq!(download_files(&t, &repo, &want, &AtomicBool::new(false), &|_| {}).unwrap().len(), 1);
        let missing = vec![FileRef { path: "nope.bin".into(), size: 1 }];
        assert_eq!(download_files(&t, &repo, &missing, &AtomicBool::new(false), &|_| {}).unwrap_err(), MarketError::NotFound);
    }

    #[test]
    fn cancellation_stops_downloads() {
        let t = fake();
        let repo = RepoRef::parse("a/b").unwrap();
        let want = vec![FileRef { path: "chars/kfm/kfm.sff".into(), size: 8 }];
        assert_eq!(download_files(&t, &repo, &want, &AtomicBool::new(true), &|_| {}).unwrap_err(), MarketError::Cancelled);
    }

    #[test]
    fn fs_persist_escapes_keys_and_roundtrips() {
        let dir = std::env::temp_dir().join(format!("ume-market-{}", std::process::id()));
        let p = FsPersist::new(&dir);
        let hostile = "item/../../etc/passwd:char:../x";
        let path = p.path_for(hostile);
        assert!(path.starts_with(&dir));
        assert!(!path.to_string_lossy().contains(".."));
        p.write(hostile, b"hi").unwrap();
        assert_eq!(p.read(hostile).as_deref(), Some(&b"hi"[..]));
        p.write("state.json", b"{}").unwrap();
        p.delete(hostile);
        assert!(p.read(hostile).is_none());
        let long = format!("catalog/{}", "x".repeat(400));
        p.write(&long, b"1").unwrap();
        assert_eq!(p.read(&long).as_deref(), Some(&b"1"[..]));
        std::fs::remove_dir_all(&dir).ok();
    }
}
