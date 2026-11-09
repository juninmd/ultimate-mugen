use crate::audio::MusicLoop;
use crate::registry::Registry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use ume_formats::vfs::{add_zip, norm_path, Vfs};
use ume_market::bundle::{self, Files};
use ume_market::ops::Progress;
use ume_market::persist::{catalog_key, item_key, STATE_KEY};
use ume_market::safety::lower_ext;
use ume_market::{Catalog, Installed, Item, Kind, MarketError, MarketState, Persist, RepoRef};

pub const SUGGESTED: [(&str, &str); 2] = [
    ("juninmd/ultimate-mugen", "Official project · Elecbyte sample pack"),
    ("Carlmundo/NooksVsCrans", "NooksVsCrans · chars, stages, music and packs"),
];

const STALE_AFTER_SECS: u64 = 600;

pub enum Event {
    Listed {
        repo_key: String,
        result: Result<Option<(Catalog, Option<String>)>, String>,
    },
    Progress(Progress),
    Installed {
        repo_key: String,
        item: Item,
        files: Files,
    },
    InstallFailed {
        name: String,
        error: String,
    },
    ItemFiles {
        repo_key: String,
        item_id: String,
        result: Result<Files, String>,
    },
}

type Queue = Arc<Mutex<VecDeque<Event>>>;

fn push(queue: &Queue, e: Event) {
    if let Ok(mut q) = queue.lock() {
        q.push_back(e);
    }
}

pub fn describe(e: &MarketError) -> String {
    match e {
        MarketError::Network(m) => format!("{}: {m}", crate::i18n::tr3("Falha de rede", "Network error", "Error de red")),
        MarketError::NotFound => {
            crate::i18n::tr3("Repositório ou arquivo não encontrado (repositórios privados não são suportados).", "Repository or file not found (private repositories are not supported).", "Repositorio o archivo no encontrado (los repositorios privados no son compatibles).").into()
        }
        MarketError::RateLimited {
            reset_epoch: Some(t),
        } => {
            let wait = t.saturating_sub(ume_market::now_secs()).div_ceil(60);
            crate::i18n::fill(crate::i18n::tr3("Limite de requisições do GitHub atingido; tente em ~{wait} min ou informe um token.", "GitHub rate limit reached; try again in ~{wait} min or provide a token.", "Límite de peticiones de GitHub alcanzado; prueba en ~{wait} min o indica un token."), &[("wait", &wait.to_string())])
        }
        MarketError::RateLimited { reset_epoch: None } => {
            crate::i18n::tr3("Limite de requisições do GitHub atingido; tente mais tarde ou informe um token.", "GitHub rate limit reached; try again later or provide a token.", "Límite de peticiones de GitHub alcanzado; prueba más tarde o indica un token.").into()
        }
        MarketError::Forbidden => crate::i18n::tr3("Acesso negado pelo GitHub.", "Access denied by GitHub.", "Acceso denegado por GitHub.").into(),
        MarketError::Http(c) => format!("{} (HTTP {c}).", crate::i18n::tr3("Resposta inesperada do GitHub", "Unexpected GitHub response", "Respuesta inesperada de GitHub")),
        MarketError::TooLarge => crate::i18n::tr3("Download maior que o limite permitido.", "Download exceeds the allowed limit.", "La descarga supera el límite permitido.").into(),
        MarketError::Invalid(m) => m.clone(),
        MarketError::HostNotAllowed => crate::i18n::tr3("Bloqueado: só é permitido baixar de github.com.", "Blocked: downloads are only allowed from github.com.", "Bloqueado: solo se permite descargar de github.com.").into(),
        MarketError::Cancelled => "Cancelado.".into(),
    }
}

pub fn vfs_of(kind: Kind, files: Files) -> Vfs {
    let mut vfs = Vfs::new();
    for (path, data) in files {
        if kind == Kind::Pack && lower_ext(&path) == "zip" {
            let _ = add_zip(&mut vfs, &data, "");
        } else {
            vfs.insert(norm_path(&path), data);
        }
    }
    vfs
}

fn first_audio(files: &Files) -> Option<Vec<u8>> {
    files
        .iter()
        .find(|(p, _)| matches!(lower_ext(p).as_str(), "ogg" | "mp3" | "flac" | "wav"))
        .map(|(_, d)| d.clone())
}

pub struct Market {
    pub state: MarketState,
    pub ready: bool,
    pub listing: HashSet<String>,
    pub queue: VecDeque<(String, String)>,
    pub current: Option<String>,
    pub progress: Progress,
    pub notice: String,
    pub notice_error: bool,
    catalogs: HashMap<String, Catalog>,
    persist: Arc<dyn Persist>,
    events: Queue,
    cancel: Arc<AtomicBool>,
    registered: HashSet<String>,
    requested: HashSet<String>,
    tracks: HashMap<String, Arc<(Vec<u8>, MusicLoop)>>,
    track_cursor: usize,
    to_register: Vec<(String, Vfs)>,
    to_remove: Vec<String>,
    #[cfg(target_arch = "wasm32")]
    web: Arc<ume_market::web::WebPersist>,
}

impl Market {
    pub fn open() -> Market {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let persist: Arc<dyn Persist> =
                Arc::new(ume_market::native::FsPersist::new(storage_root()));
            let mut m = Market::with_persist(persist);
            m.finish_hydration();
            m
        }
        #[cfg(target_arch = "wasm32")]
        {
            let web = Arc::new(ume_market::web::WebPersist::new());
            let persist: Arc<dyn Persist> = web.clone();
            let mut m = Market::with_persist(persist);
            m.web = web;
            m
        }
    }

    pub fn with_persist(persist: Arc<dyn Persist>) -> Market {
        Market {
            state: MarketState::default(),
            ready: false,
            listing: HashSet::new(),
            queue: VecDeque::new(),
            current: None,
            progress: Progress::default(),
            notice: String::new(),
            notice_error: false,
            catalogs: HashMap::new(),
            persist,
            events: Arc::new(Mutex::new(VecDeque::new())),
            cancel: Arc::new(AtomicBool::new(false)),
            registered: HashSet::new(),
            requested: HashSet::new(),
            tracks: HashMap::new(),
            track_cursor: 0,
            to_register: Vec::new(),
            to_remove: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            web: Arc::new(ume_market::web::WebPersist::new()),
        }
    }

    pub fn finish_hydration(&mut self) {
        self.state = self
            .persist
            .read(STATE_KEY)
            .map(|b| MarketState::from_json(&b))
            .unwrap_or_default();
        self.ready = true;
        self.sync_active();
    }

    pub fn token(&self) -> Option<String> {
        let saved = self.state.token.clone().filter(|t| !t.is_empty());
        #[cfg(not(target_arch = "wasm32"))]
        let saved = saved.or_else(|| {
            ["UME_GITHUB_TOKEN", "GITHUB_TOKEN"]
                .iter()
                .find_map(|k| std::env::var(k).ok())
                .filter(|t| !t.is_empty())
        });
        saved
    }

    pub fn set_token(&mut self, token: &str) {
        let t = token.trim();
        self.state.token = if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        };
        self.save_state();
        self.notice = if t.is_empty() {
            "Token removido.".into()
        } else {
            "Token salvo neste aparelho.".into()
        };
    }

    pub fn save_state(&self) {
        let _ = self.persist.write(STATE_KEY, &self.state.to_json());
    }

    pub fn catalog(&mut self, repo_key: &str) -> Option<&Catalog> {
        if !self.catalogs.contains_key(repo_key) {
            let c: Catalog =
                serde_json::from_slice(&self.persist.read(&catalog_key(repo_key))?).ok()?;
            self.catalogs.insert(repo_key.to_string(), c);
        }
        self.catalogs.get(repo_key)
    }

    pub fn catalog_cached(&self, repo_key: &str) -> Option<&Catalog> {
        self.catalogs.get(repo_key)
    }

    pub fn add_repo(&mut self, input: &str) -> Result<String, String> {
        let repo = RepoRef::parse(input)?;
        let key = repo.key();
        if !self.state.add_repo(repo) {
            return Ok(key);
        }
        self.save_state();
        self.refresh(&key);
        Ok(key)
    }

    pub fn remove_repo(&mut self, key: &str) {
        if self.state.remove_repo(key) {
            self.persist.delete(&catalog_key(key));
            self.catalogs.remove(key);
            self.save_state();
        }
    }

    pub fn needs_refresh(&self, key: &str) -> bool {
        self.state.repo(key).is_some_and(|r| {
            r.listed_at == 0
                || ume_market::now_secs().saturating_sub(r.listed_at) > STALE_AFTER_SECS
        })
    }

    pub fn refresh(&mut self, key: &str) {
        if self.listing.contains(key) {
            return;
        }
        let Some(rec) = self.state.repo(key) else {
            return;
        };
        let (repo, saved_etag) = (rec.repo.clone(), rec.etag.clone());
        let etag = if self.catalog(key).is_some() {
            saved_etag
        } else {
            None
        };
        self.listing.insert(key.to_string());
        spawn_list(
            self.events.clone(),
            repo,
            key.to_string(),
            etag,
            self.token(),
        );
    }

    pub fn queue_install(&mut self, repo_key: &str, ids: &[String]) -> usize {
        let mut added = 0;
        for id in ids {
            let already = self.queue.iter().any(|(r, i)| r == repo_key && i == id)
                || self.current_is(repo_key, id);
            if !already
                && self
                    .catalog(repo_key)
                    .and_then(|c| c.find(id))
                    .is_some_and(|i| i.installable)
            {
                self.queue.push_back((repo_key.to_string(), id.clone()));
                added += 1;
            }
        }
        self.start_next();
        added
    }

    fn current_is(&self, repo_key: &str, id: &str) -> bool {
        self.current.as_deref() == Some(&format!("{repo_key}\u{0}{id}"))
    }

    pub fn busy(&self) -> bool {
        self.current.is_some() || !self.queue.is_empty()
    }

    pub fn cancel_all(&mut self) {
        self.queue.clear();
        self.cancel.store(true, Ordering::Relaxed);
    }

    fn start_next(&mut self) {
        if self.current.is_some() {
            return;
        }
        while let Some((repo_key, id)) = self.queue.pop_front() {
            let Some(repo) = self.state.repo(&repo_key).map(|r| r.repo.clone()) else {
                continue;
            };
            let Some(catalog) = self.catalog(&repo_key).cloned() else {
                continue;
            };
            let Some(item) = catalog.find(&id).cloned() else {
                continue;
            };
            self.cancel = Arc::new(AtomicBool::new(false));
            self.progress = Progress {
                total_files: item.files.len(),
                total_bytes: item.bytes,
                ..Default::default()
            };
            self.current = Some(format!("{repo_key}\u{0}{id}"));
            spawn_install(
                self.events.clone(),
                self.persist.clone(),
                self.cancel.clone(),
                repo,
                repo_key,
                item,
                catalog,
            );
            return;
        }
    }

    pub fn current_name(&self) -> Option<String> {
        let cur = self.current.as_deref()?;
        let (repo_key, id) = cur.split_once('\u{0}')?;
        self.catalogs
            .get(repo_key)
            .and_then(|c| c.find(id))
            .map(|i| i.name.clone())
    }

    pub fn uninstall(&mut self, repo_key: &str, item_id: &str) {
        let Some(entry) = self.state.remove_installed(repo_key, item_id) else {
            return;
        };
        self.persist.delete(&item_key(repo_key, item_id));
        let tag = entry.tag();
        self.registered.remove(&tag);
        self.requested.remove(&tag);
        self.tracks.remove(&tag);
        self.to_remove.push(tag);
        self.save_state();
    }

    pub fn set_active<'a>(
        &mut self,
        targets: impl IntoIterator<Item = (&'a str, &'a str)>,
        active: bool,
    ) -> usize {
        let n = self.state.set_active_many(targets, active);
        if n > 0 {
            self.save_state();
            self.sync_active();
        }
        n
    }

    fn sync_active(&mut self) {
        let wanted: Vec<Installed> = self.state.installed.clone();
        for it in wanted {
            let tag = it.tag();
            if it.active && !self.registered.contains(&tag) && !self.requested.contains(&tag) {
                self.requested.insert(tag);
                self.request_files(&it.repo_key, &it.item_id);
            } else if !it.active && (self.registered.remove(&tag) | self.requested.remove(&tag)) {
                self.tracks.remove(&tag);
                self.to_remove.push(tag);
            }
        }
    }

    fn request_files(&self, repo_key: &str, item_id: &str) {
        let (events, persist, key) = (
            self.events.clone(),
            self.persist.clone(),
            item_key(repo_key, item_id),
        );
        let (repo_key, item_id) = (repo_key.to_string(), item_id.to_string());
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let result = persist
                .read(&key)
                .ok_or_else(|| {
                    "Arquivos do item não encontrados no armazenamento local.".to_string()
                })
                .and_then(|b| bundle::decode(&b));
            push(
                &events,
                Event::ItemFiles {
                    repo_key,
                    item_id,
                    result,
                },
            );
        });
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (events, persist, repo_key, item_id);
            self.web.request_item(&key);
        }
    }

    pub fn pick_track(&mut self) -> Option<Arc<(Vec<u8>, MusicLoop)>> {
        if self.tracks.is_empty() {
            return None;
        }
        let mut tags: Vec<&String> = self.tracks.keys().collect();
        tags.sort();
        let t = self
            .tracks
            .get(tags[self.track_cursor % tags.len()])?
            .clone();
        self.track_cursor += 1;
        Some(t)
    }

    pub fn active_tracks(&self) -> usize {
        self.tracks.len()
    }

    #[cfg(target_arch = "wasm32")]
    fn drain_web(&mut self) {
        let (hydrated, delivered, done) = web_glue::take();
        for (k, v) in hydrated {
            self.web.hydrate(&k, &v);
        }
        if done && !self.ready {
            self.finish_hydration();
        }
        for (key, data) in delivered {
            let found = self
                .state
                .installed
                .iter()
                .find(|i| item_key(&i.repo_key, &i.item_id) == key)
                .map(|i| (i.repo_key.clone(), i.item_id.clone()));
            if let Some((repo_key, item_id)) = found {
                let result = if data.is_empty() {
                    Err("Arquivos do item não encontrados no armazenamento local.".to_string())
                } else {
                    bundle::decode(&data)
                };
                push(
                    &self.events,
                    Event::ItemFiles {
                        repo_key,
                        item_id,
                        result,
                    },
                );
            }
        }
    }

    pub fn poll(&mut self) {
        #[cfg(target_arch = "wasm32")]
        self.drain_web();
        let events: Vec<Event> = self
            .events
            .lock()
            .map(|mut q| q.drain(..).collect())
            .unwrap_or_default();
        for e in events {
            self.handle(e);
        }
    }

    fn handle(&mut self, e: Event) {
        match e {
            Event::Progress(p) => self.progress = p,
            Event::Listed { repo_key, result } => {
                self.listing.remove(&repo_key);
                match result {
                    Err(msg) => {
                        self.notice = msg;
                        self.notice_error = true;
                    }
                    Ok(None) => {
                        if self.catalog(&repo_key).is_some() {
                            if let Some(r) = self.state.repo_mut(&repo_key) {
                                r.listed_at = ume_market::now_secs();
                            }
                            self.save_state();
                        } else {
                            if let Some(r) = self.state.repo_mut(&repo_key) {
                                r.etag = None;
                            }
                            self.refresh(&repo_key);
                        }
                    }
                    Ok(Some((catalog, etag))) => {
                        let _ = self.persist.write(
                            &catalog_key(&repo_key),
                            &serde_json::to_vec(&catalog).unwrap_or_default(),
                        );
                        if let Some(r) = self.state.repo_mut(&repo_key) {
                            r.etag = etag;
                            r.listed_at = ume_market::now_secs();
                            r.item_count = catalog.items.len();
                        }
                        self.notice = if catalog.truncated {
                            crate::i18n::tr3("Repositório muito grande: a listagem do GitHub veio truncada.", "Repository too large: the GitHub listing was truncated.", "Repositorio demasiado grande: el listado de GitHub llegó truncado.").into()
                        } else if catalog.items.is_empty() {
                            crate::i18n::tr3("Nenhum personagem, cenário ou música encontrado neste repositório.", "No characters, stages or music found in this repository.", "No se encontraron personajes, escenarios ni música en este repositorio.").into()
                        } else {
                            format!("{} itens encontrados.", catalog.items.len())
                        };
                        self.catalogs.insert(repo_key, catalog);
                        self.save_state();
                    }
                }
            }
            Event::Installed {
                repo_key,
                item,
                files,
            } => {
                self.current = None;
                let entry = Installed {
                    repo_key: repo_key.clone(),
                    item_id: item.id.clone(),
                    kind: item.kind,
                    name: item.name.clone(),
                    bytes: item.bytes,
                    files: files.len(),
                    active: true,
                    installed_at: ume_market::now_secs(),
                };
                let tag = entry.tag();
                self.registered.remove(&tag);
                self.state.upsert_installed(entry);
                self.save_state();
                self.requested.remove(&tag);
                self.to_remove.push(tag.clone());
                self.register(&tag, item.kind, files);
                self.notice = format!("Instalado: {}", item.name);
                self.start_next();
            }
            Event::InstallFailed { name, error } => {
                self.current = None;
                self.notice = format!("{name}: {error}");
                self.notice_error = true;
                if error == describe(&MarketError::Cancelled) {
                    self.queue.clear();
                }
                self.start_next();
            }
            Event::ItemFiles {
                repo_key,
                item_id,
                result,
            } => {
                let Some(it) = self.state.installed_item(&repo_key, &item_id).cloned() else {
                    return;
                };
                let tag = it.tag();
                self.requested.remove(&tag);
                if !it.active || self.registered.contains(&tag) {
                    return;
                }
                match result {
                    Ok(files) => self.register(&tag, it.kind, files),
                    Err(e) => {
                        self.notice = format!("{}: {e}", it.name);
                        self.notice_error = true;
                    }
                }
            }
        }
    }

    fn register(&mut self, tag: &str, kind: Kind, files: Files) {
        if kind == Kind::Music {
            if let Some(bytes) = first_audio(&files) {
                self.tracks
                    .insert(tag.to_string(), Arc::new((bytes, MusicLoop::default())));
            }
        } else {
            self.to_register
                .push((tag.to_string(), vfs_of(kind, files)));
        }
        self.registered.insert(tag.to_string());
    }

    pub fn has_registry_work(&self) -> bool {
        !self.to_register.is_empty() || !self.to_remove.is_empty()
    }

    pub fn apply_to(&mut self, reg: &mut Registry) -> bool {
        let mut changed = false;
        for tag in self.to_remove.drain(..) {
            let (c, s) = reg.remove_tag(&tag);
            changed |= c + s > 0;
        }
        for (tag, vfs) in self.to_register.drain(..) {
            let (c, s, _) = reg.add_vfs_lazy(vfs, &tag);
            changed |= !c.is_empty() || !s.is_empty();
        }
        if changed {
            reg.sort_mugen();
        }
        changed
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn storage_root() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    if let Some(d) = ume_shell::ANDROID_APP
        .get()
        .and_then(|a| a.internal_data_path())
    {
        return d.join("market");
    }
    ume_market::native::default_root()
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_list(
    events: Queue,
    repo: RepoRef,
    repo_key: String,
    etag: Option<String>,
    token: Option<String>,
) {
    std::thread::spawn(move || {
        let t = ume_market::native::UreqTransport::default();
        let result =
            match ume_market::native::list_repo(&t, &repo, etag.as_deref(), token.as_deref()) {
                Ok(ume_market::native::ListOutcome::NotModified) => Ok(None),
                Ok(ume_market::native::ListOutcome::Fresh { catalog, etag }) => {
                    Ok(Some((catalog, etag)))
                }
                Err(e) => Err(describe(&e)),
            };
        push(&events, Event::Listed { repo_key, result });
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_list(
    events: Queue,
    repo: RepoRef,
    repo_key: String,
    etag: Option<String>,
    token: Option<String>,
) {
    ume_market::web::spawn(async move {
        let result = match ume_market::web::list_repo(&repo, etag.as_deref(), token.as_deref())
            .await
        {
            Ok(ume_market::web::ListOutcome::NotModified) => Ok(None),
            Ok(ume_market::web::ListOutcome::Fresh { catalog, etag }) => Ok(Some((catalog, etag))),
            Err(e) => Err(describe(&e)),
        };
        push(&events, Event::Listed { repo_key, result });
    });
}

fn finish_install(
    events: &Queue,
    persist: &Arc<dyn Persist>,
    repo_key: String,
    item: Item,
    result: Result<Files, MarketError>,
) {
    match result {
        Ok(files) => match persist.write(&item_key(&repo_key, &item.id), &bundle::encode(&files)) {
            Ok(()) => push(
                events,
                Event::Installed {
                    repo_key,
                    item,
                    files,
                },
            ),
            Err(e) => push(
                events,
                Event::InstallFailed {
                    name: item.name,
                    error: format!("não foi possível salvar: {e}"),
                },
            ),
        },
        Err(e) => push(
            events,
            Event::InstallFailed {
                name: item.name,
                error: describe(&e),
            },
        ),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_install(
    events: Queue,
    persist: Arc<dyn Persist>,
    cancel: Arc<AtomicBool>,
    repo: RepoRef,
    repo_key: String,
    item: Item,
    catalog: Catalog,
) {
    std::thread::spawn(move || {
        let t = ume_market::native::UreqTransport::default();
        let progress_events = events.clone();
        let result =
            ume_market::native::install_item(&t, &repo, &item, &catalog, &cancel, &move |p| {
                push(&progress_events, Event::Progress(p))
            });
        finish_install(&events, &persist, repo_key, item, result);
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_install(
    events: Queue,
    persist: Arc<dyn Persist>,
    cancel: Arc<AtomicBool>,
    repo: RepoRef,
    repo_key: String,
    item: Item,
    catalog: Catalog,
) {
    ume_market::web::spawn(async move {
        let progress_events = events.clone();
        let result = ume_market::web::install_item(&repo, &item, &catalog, &cancel, &move |p| {
            push(&progress_events, Event::Progress(p))
        })
        .await;
        finish_install(&events, &persist, repo_key, item, result);
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn ume_prompt(message: &str, default: &str) -> String;
}

pub fn ask_text(message: &str, default: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let v = ume_prompt(message, default);
        return Some(v).filter(|s| !s.trim().is_empty());
    }
    #[allow(unreachable_code)]
    {
        let _ = (message, default);
        None
    }
}

pub const fn typing_supported() -> bool {
    !cfg!(any(target_os = "android", target_os = "ios"))
}

#[cfg(target_arch = "wasm32")]
pub mod web_glue {
    use std::sync::Mutex;

    #[derive(Default)]
    struct Inbox {
        hydrated: Vec<(String, Vec<u8>)>,
        delivered: Vec<(String, Vec<u8>)>,
        done: bool,
    }

    static INBOX: Mutex<Inbox> = Mutex::new(Inbox {
        hydrated: Vec::new(),
        delivered: Vec::new(),
        done: false,
    });

    pub fn take() -> (Vec<(String, Vec<u8>)>, Vec<(String, Vec<u8>)>, bool) {
        let mut i = INBOX.lock().unwrap();
        (
            std::mem::take(&mut i.hydrated),
            std::mem::take(&mut i.delivered),
            i.done,
        )
    }

    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn ume_market_hydrate(key: String, data: Vec<u8>) {
        INBOX.lock().unwrap().hydrated.push((key, data));
    }

    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn ume_market_hydrate_done() {
        INBOX.lock().unwrap().done = true;
    }

    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn ume_market_deliver(key: String, data: Vec<u8>) {
        INBOX.lock().unwrap().delivered.push((key, data));
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use ume_market::catalog::{detect, Tree, TreeEntry};
    use ume_market::MemPersist;

    fn elecbyte_files() -> Files {
        let mut vfs = Vfs::new();
        add_zip(&mut vfs, include_bytes!("../assets/mugen/elecbyte.zip"), "").unwrap();
        vfs.into_iter().collect()
    }

    fn market_with_repo() -> (Market, String, Arc<MemPersist>) {
        let persist = Arc::new(MemPersist::new());
        let mut m = Market::with_persist(persist.clone());
        m.finish_hydration();
        m.state.add_repo(RepoRef::parse("a/b").unwrap());
        (m, "a/b".to_string(), persist)
    }

    fn character_item(files: &Files) -> Item {
        let tree = Tree {
            entries: files
                .iter()
                .map(|(p, d)| TreeEntry {
                    path: p.clone(),
                    size: d.len() as u64,
                })
                .collect(),
            truncated: false,
        };
        detect(&tree, "", "kfm-pack")
            .items
            .into_iter()
            .find(|i| i.kind == Kind::Character)
            .expect("a character in the bundled pack")
    }

    fn wait_for_files(m: &mut Market) {
        let start = Instant::now();
        while !m.has_registry_work() && start.elapsed() < Duration::from_secs(10) {
            m.poll();
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn installed_character_is_registered_deactivated_and_reactivated_from_storage() {
        let files = elecbyte_files();
        let item = character_item(&files);
        let (mut m, repo_key, persist) = market_with_repo();
        let mut reg = Registry::new();
        let before = reg.chars.len();
        finish_install(
            &m.events,
            &(persist.clone() as Arc<dyn Persist>),
            repo_key.clone(),
            item.clone(),
            Ok(files),
        );
        m.poll();
        assert!(m.apply_to(&mut reg));
        let added = reg.chars.len() - before;
        assert!(added >= 1);
        let tag = m.state.installed_item(&repo_key, &item.id).unwrap().tag();
        assert!(reg.chars.iter().any(|c| c.id.contains(&tag)));

        m.set_active([(repo_key.as_str(), item.id.as_str())], false);
        assert!(m.apply_to(&mut reg));
        assert_eq!(reg.chars.len(), before);

        m.set_active([(repo_key.as_str(), item.id.as_str())], true);
        wait_for_files(&mut m);
        assert!(m.apply_to(&mut reg));
        assert_eq!(reg.chars.len(), before + added);
    }

    #[test]
    fn state_survives_a_restart_and_active_items_come_back() {
        let files = elecbyte_files();
        let item = character_item(&files);
        let (mut m, repo_key, persist) = market_with_repo();
        finish_install(
            &m.events,
            &(persist.clone() as Arc<dyn Persist>),
            repo_key.clone(),
            item.clone(),
            Ok(files),
        );
        m.poll();
        let mut reborn = Market::with_persist(persist);
        reborn.finish_hydration();
        assert_eq!(reborn.state.installed.len(), 1);
        assert!(reborn.state.installed[0].active);
        wait_for_files(&mut reborn);
        let mut reg = Registry::new();
        let before = reg.chars.len();
        assert!(reborn.apply_to(&mut reg));
        assert!(reg.chars.len() > before);
    }

    #[test]
    fn uninstall_removes_storage_and_registry_entries() {
        let files = elecbyte_files();
        let item = character_item(&files);
        let (mut m, repo_key, persist) = market_with_repo();
        let mut reg = Registry::new();
        let before = reg.chars.len();
        finish_install(
            &m.events,
            &(persist.clone() as Arc<dyn Persist>),
            repo_key.clone(),
            item.clone(),
            Ok(files),
        );
        m.poll();
        m.apply_to(&mut reg);
        m.uninstall(&repo_key, &item.id);
        assert!(m.apply_to(&mut reg));
        assert_eq!(reg.chars.len(), before);
        assert!(m.state.installed.is_empty());
        assert!(persist.read(&item_key(&repo_key, &item.id)).is_none());
    }

    #[test]
    fn active_music_rotates_through_tracks() {
        let (mut m, repo_key, persist) = market_with_repo();
        for name in ["a", "b"] {
            let tree = Tree {
                entries: vec![TreeEntry {
                    path: format!("music/{name}.ogg"),
                    size: 5,
                }],
                truncated: false,
            };
            let item = detect(&tree, "", "r").items.remove(0);
            finish_install(
                &m.events,
                &(persist.clone() as Arc<dyn Persist>),
                repo_key.clone(),
                item,
                Ok(vec![(
                    format!("music/{name}.ogg"),
                    vec![name.as_bytes()[0]],
                )]),
            );
        }
        m.poll();
        assert_eq!(m.active_tracks(), 2);
        let first = m.pick_track().unwrap();
        let second = m.pick_track().unwrap();
        let third = m.pick_track().unwrap();
        assert_ne!(first.0, second.0);
        assert_eq!(first.0, third.0);
        m.set_active([(repo_key.as_str(), "music:music/a.ogg")], false);
        assert_eq!(m.active_tracks(), 1);
    }

    #[test]
    fn failed_listing_reports_the_error_and_unblocks_refresh() {
        let (mut m, repo_key, _) = market_with_repo();
        m.listing.insert(repo_key.clone());
        m.handle(Event::Listed {
            repo_key: repo_key.clone(),
            result: Err("sem rede".into()),
        });
        assert!(m.notice_error && m.notice == "sem rede");
        assert!(!m.listing.contains(&repo_key));
    }

    #[test]
    fn add_repo_rejects_foreign_hosts() {
        let (mut m, _, _) = market_with_repo();
        assert!(m.add_repo("https://gitlab.com/a/b").is_err());
        assert!(m.add_repo("javascript:alert(1)").is_err());
        assert_eq!(m.state.repos.len(), 1);
    }

    #[test]
    fn rate_limit_message_mentions_the_wait() {
        let msg = describe(&MarketError::RateLimited {
            reset_epoch: Some(ume_market::now_secs() + 600),
        });
        assert!(msg.contains("min") && msg.contains("token"));
    }
}
