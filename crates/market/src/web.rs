use crate::bundle::Files;
use crate::catalog::{detect, Catalog, FileRef, Item};
use crate::client::{check_request, file_request, interpret_file, interpret_tree, is_transient, tree_request, MarketError, RateLimit, Request, Response, TreeOutcome};
use crate::ops::{InstallJob, Progress, PARALLEL_DOWNLOADS};
use crate::repo::RepoRef;
use crate::safety::url_allowed;
use futures_util::future::join_all;
use js_sys::{Function, Promise, Uint8Array};
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, RequestInit, RequestMode, Response as WebResponse, WorkerGlobalScope};

const ATTEMPTS: usize = 3;

fn js_err(e: JsValue) -> MarketError {
    MarketError::Network(e.as_string().or_else(|| js_sys::Reflect::get(&e, &"message".into()).ok().and_then(|m| m.as_string())).unwrap_or_else(|| "request failed".into()))
}

async fn sleep_ms(ms: i32) {
    let promise = Promise::new(&mut |resolve: Function, _| {
        let global = js_sys::global();
        if let Some(w) = global.dyn_ref::<web_sys::Window>() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        } else if let Some(w) = global.dyn_ref::<WorkerGlobalScope>() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        } else {
            let _ = resolve.call0(&JsValue::NULL);
        }
    });
    let _ = JsFuture::from(promise).await;
}

fn fetch_promise(req: &web_sys::Request) -> Promise {
    let global = js_sys::global();
    if let Some(w) = global.dyn_ref::<web_sys::Window>() {
        w.fetch_with_request(req)
    } else if let Some(w) = global.dyn_ref::<WorkerGlobalScope>() {
        w.fetch_with_request(req)
    } else {
        Promise::reject(&JsValue::from_str("fetch is not available"))
    }
}

pub async fn get(req: &Request) -> Result<Response, MarketError> {
    check_request(req)?;
    let headers = Headers::new().map_err(js_err)?;
    if req.url.starts_with("https://api.github.com/") {
        headers.set("Accept", "application/vnd.github+json").map_err(js_err)?;
        if let Some(e) = &req.etag {
            headers.set("If-None-Match", e).map_err(js_err)?;
        }
        if let Some(t) = &req.token {
            headers.set("Authorization", &format!("Bearer {t}")).map_err(js_err)?;
        }
    }
    let init = RequestInit::new();
    init.set_method("GET");
    init.set_mode(RequestMode::Cors);
    init.set_headers(&headers);
    let request = web_sys::Request::new_with_str_and_init(&req.url, &init).map_err(js_err)?;
    let resp: WebResponse = JsFuture::from(fetch_promise(&request)).await.map_err(js_err)?.dyn_into().map_err(|_| MarketError::Network("unexpected response".into()))?;
    if !url_allowed(&resp.url()) && !resp.url().is_empty() {
        return Err(MarketError::HostNotAllowed);
    }
    let status = resp.status();
    let h = resp.headers();
    let header = |n: &str| h.get(n).ok().flatten();
    let rate = match (header("x-ratelimit-remaining").and_then(|v| v.parse().ok()), header("x-ratelimit-reset").and_then(|v| v.parse().ok())) {
        (Some(remaining), Some(reset_epoch)) => Some(RateLimit { remaining, reset_epoch }),
        _ => None,
    };
    let mut out = Response { status, etag: header("etag"), body: Vec::new(), rate, retry_after_secs: header("retry-after").and_then(|v| v.parse().ok()), location: None };
    if status != 200 {
        return Ok(out);
    }
    if header("content-length").and_then(|v| v.parse::<u64>().ok()).is_some_and(|n| n > req.max_bytes) {
        return Err(MarketError::TooLarge);
    }
    let buf = JsFuture::from(resp.array_buffer().map_err(js_err)?).await.map_err(js_err)?;
    let bytes = Uint8Array::new(&buf);
    if bytes.length() as u64 > req.max_bytes {
        return Err(MarketError::TooLarge);
    }
    out.body = bytes.to_vec();
    Ok(out)
}

async fn get_retry(req: &Request, cancel: &AtomicBool) -> Result<Response, MarketError> {
    let mut last = MarketError::Cancelled;
    for attempt in 0..ATTEMPTS {
        if cancel.load(Ordering::Relaxed) {
            return Err(MarketError::Cancelled);
        }
        match get(req).await {
            Ok(r) if (500..600).contains(&r.status) => last = MarketError::Http(r.status),
            Ok(r) => return Ok(r),
            Err(e) if is_transient(&e) => last = e,
            Err(e) => return Err(e),
        }
        sleep_ms(300 * (attempt as i32 + 1)).await;
    }
    Err(last)
}

pub enum ListOutcome {
    NotModified,
    Fresh { catalog: Catalog, etag: Option<String> },
}

pub async fn list_repo(repo: &RepoRef, etag: Option<&str>, token: Option<&str>) -> Result<ListOutcome, MarketError> {
    let resp = get_retry(&tree_request(repo, etag, token), &AtomicBool::new(false)).await?;
    match interpret_tree(&resp)? {
        TreeOutcome::NotModified => Ok(ListOutcome::NotModified),
        TreeOutcome::Fresh { tree, etag } => Ok(ListOutcome::Fresh { catalog: detect(&tree, &repo.subdir, &repo.name), etag }),
    }
}

async fn download_one(repo: &RepoRef, f: &FileRef, cancel: &AtomicBool) -> Result<Vec<u8>, MarketError> {
    let limit = f.size.max(1).saturating_mul(2).max(1 << 20);
    let resp = get_retry(&file_request(repo, &f.path, limit), cancel).await?;
    interpret_file(&resp, limit)
}

pub async fn download_files(repo: &RepoRef, wanted: &[FileRef], cancel: &AtomicBool, on_file: &dyn Fn(u64)) -> Result<Files, MarketError> {
    let mut out: Files = Vec::with_capacity(wanted.len());
    for chunk in wanted.chunks(PARALLEL_DOWNLOADS) {
        let results = join_all(chunk.iter().map(|f| download_one(repo, f, cancel))).await;
        for (f, r) in chunk.iter().zip(results) {
            let data = r?;
            on_file(data.len() as u64);
            out.push((f.path.clone(), data));
        }
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(MarketError::Cancelled);
    }
    Ok(out)
}

pub async fn install_item(repo: &RepoRef, item: &Item, catalog: &Catalog, cancel: &AtomicBool, progress: &dyn Fn(Progress)) -> Result<Files, MarketError> {
    let mut job = InstallJob::new(item, catalog)?;
    let (mut done_files, mut done_bytes, mut total_files, mut total_bytes) = (0usize, 0u64, 0usize, 0u64);
    while !job.is_done() {
        let wanted: Vec<FileRef> = job.pending().to_vec();
        total_files += wanted.len();
        total_bytes += wanted.iter().map(|f| f.size).sum::<u64>();
        let counters = std::cell::Cell::new((done_files, done_bytes));
        let (tf, tb) = (total_files, total_bytes);
        let downloaded = download_files(repo, &wanted, cancel, &|n| {
            let (f, b) = counters.get();
            counters.set((f + 1, b + n));
            progress(Progress { done_files: f + 1, total_files: tf, done_bytes: b + n, total_bytes: tb });
        })
        .await?;
        (done_files, done_bytes) = counters.get();
        job.accept(downloaded)?;
    }
    Ok(job.finish())
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn ume_market_put(key: &str, data: &[u8]);
    fn ume_market_del(key: &str);
    fn ume_market_get(key: &str);
}

#[derive(Default)]
pub struct WebPersist {
    memory: crate::persist::MemPersist,
}

fn cached_in_memory(key: &str) -> bool {
    key == crate::persist::STATE_KEY || key.starts_with("catalog/")
}

impl WebPersist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hydrate(&self, key: &str, data: &[u8]) {
        use crate::persist::Persist;
        if cached_in_memory(key) {
            let _ = self.memory.write(key, data);
        }
    }

    pub fn request_item(&self, key: &str) {
        ume_market_get(key);
    }
}

impl crate::persist::Persist for WebPersist {
    fn read(&self, key: &str) -> Option<Vec<u8>> {
        self.memory.read(key)
    }

    fn write(&self, key: &str, data: &[u8]) -> Result<(), String> {
        if cached_in_memory(key) {
            self.memory.write(key, data)?;
        }
        ume_market_put(key, data);
        Ok(())
    }

    fn delete(&self, key: &str) {
        self.memory.delete(key);
        ume_market_del(key);
    }
}

pub fn spawn(future: impl std::future::Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(future);
}
