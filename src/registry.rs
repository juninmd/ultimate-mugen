//! Character & stage registry: built-in UMF fighters (embedded JSON) + MUGEN content (chars/ and stages/ folders, imports).
use crate::audio::MusicLoop;
use crate::stages::{self, Stage};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;
use ume_engine::chardef::{CharDef, Sprite};
use ume_engine::loader::{load_mugen_char, load_mugen_portrait};
use ume_engine::native::{compile_native, NativeChar};
use ume_formats::ini::{find_section, parse_ini, unquote};
use ume_formats::stage::load_stage;
use ume_formats::vfs::{char_key, find_char_defs, find_stage_defs, stage_key, text, Vfs};

#[path = "registry_sprite_cache.rs"]
mod sprite_cache;

include!(concat!(env!("OUT_DIR"), "/chars_gen.rs"));

/// "true_z_license_free_chara" -> "True Z", "Thunder_horse_GGX" -> "Thunder Horse GGX"
pub fn pretty_name(n: &str) -> String {
    let mut s = n.replace(['_', '-'], " ");
    for junk in [
        "license free mugen chara for official",
        "license free chara",
        "license free",
        "for official",
    ] {
        if let Some(i) = s.to_ascii_lowercase().find(junk) {
            s.replace_range(i..i + junk.len(), "");
        }
    }
    let words: Vec<String> = s
        .split_whitespace()
        .map(|w| {
            if w.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                && w.len() > 1
            {
                w.to_string()
            } else {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default()
            }
        })
        .collect();
    if words.is_empty() {
        n.to_string()
    } else {
        words.join(" ")
    }
}

#[derive(Clone)]
pub enum Src {
    Native(Arc<NativeChar>),
    Mugen { vfs: Arc<Vfs>, def: String },
}

pub type StageJob = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>>>>;

pub type LoadJob = Option<std::thread::JoinHandle<Result<CharDef, String>>>;

fn build(src: &Src, pal: i32, name: &str) -> Result<CharDef, String> {
    match src {
        Src::Native(nc) => Ok(compile_native(nc, pal, true)),
        Src::Mugen { vfs, def } => load_mugen_char(vfs, def, pal, true).map(|mut d| {
            d.display_name = name.to_string();
            d
        }),
    }
}

#[derive(Default)]
struct Inflight {
    load: HashMap<i32, std::thread::JoinHandle<Result<CharDef, String>>>,
    portrait: Option<std::thread::JoinHandle<Option<Sprite>>>,
}

const MAX_STREAMS: usize = 2;

pub struct CharEntry {
    pub key: String,
    pub id: String,
    pub name: String,
    pub series: String,
    pub author: String,
    pub color: [u8; 3],
    pub description: String,
    pub mugen: bool,
    /** final boss: hidden from the roster */
    pub boss: bool,
    pub src: Src,
    cache: RefCell<HashMap<i32, Arc<CharDef>>>,
    /// idle/win only (native fighters): ~10x cheaper than a full compile, used by the select grid and VS screen
    preview: RefCell<HashMap<i32, Arc<CharDef>>>,
    last_use: Cell<u64>,
    /// select-grid thumbnail (MUGEN): survives `unload()` so tiles keep their picture after the cache is trimmed; `Some(None)` = none available
    portrait: RefCell<Option<Option<Sprite>>>,
    pub broken: Cell<bool>,
    /// failed loads are remembered (no retry storm every frame)
    errs: RefCell<HashMap<i32, String>>,
    jobs: RefCell<Inflight>,
    sprite_revision: Cell<u64>,
}
impl CharEntry {
    pub fn load(&self, pal: i32) -> Result<Arc<CharDef>, String> {
        self.refresh_sprite_pack();
        if let Some(d) = self.cache.borrow().get(&pal) {
            return Ok(d.clone());
        }
        if let Some(e) = self.errs.borrow().get(&pal) {
            return Err(e.clone());
        }
        self.store(pal, build(&self.src, pal, &self.name))
    }
    pub fn spawn_load(&self, pal: i32) -> LoadJob {
        if cfg!(target_arch = "wasm32")
            || self.cache.borrow().contains_key(&pal)
            || self.errs.borrow().contains_key(&pal)
        {
            return None;
        }
        let (src, name) = (self.src.clone(), self.name.clone());
        std::thread::Builder::new()
            .name("char-load".into())
            .spawn(move || build(&src, pal, &name))
            .ok()
    }
    pub fn finish_load(&self, pal: i32, job: LoadJob) -> Result<Arc<CharDef>, String> {
        match job {
            Some(h) => self.store(
                pal,
                h.join()
                    .unwrap_or_else(|_| Err("loader thread panicked".into())),
            ),
            None => self.load(pal),
        }
    }
    fn store(&self, pal: i32, r: Result<CharDef, String>) -> Result<Arc<CharDef>, String> {
        let d = match r {
            Ok(d) => Arc::new(d),
            Err(e) => {
                if self.mugen {
                    self.broken.set(true);
                }
                self.errs.borrow_mut().insert(pal, e.clone());
                return Err(e);
            }
        };
        if self.mugen && self.portrait.borrow().is_none() {
            *self.portrait.borrow_mut() = Some(portrait_of(&d));
        }
        self.cache.borrow_mut().insert(pal, d.clone());
        self.preview.borrow_mut().remove(&pal);
        self.touch();
        Ok(d)
    }
    /// Background variant of [`Self::load_preview`] for MUGEN characters: spawns a thread, collects the result on later frames, never blocks. Sequential (still one per frame) on wasm and for UMF.
    pub fn stream_preview(&self, pal: i32) {
        if !self.mugen || cfg!(target_arch = "wasm32") {
            let _ = self.load_preview(pal);
            return;
        }
        if self.peek(pal).is_some() || self.errs.borrow().contains_key(&pal) {
            return;
        }
        if self.jobs.borrow().load.get(&pal).is_some() {
            self.reap();
            return;
        }
        if self.jobs.borrow().load.len() >= MAX_STREAMS {
            return;
        }
        let (src, name) = (self.src.clone(), self.name.clone());
        if let Ok(h) = std::thread::Builder::new()
            .name("char-preview".into())
            .spawn(move || build(&src, pal, &name))
        {
            self.jobs.borrow_mut().load.insert(pal, h);
        }
    }
    pub fn stream_portrait(&self) {
        if self.portrait.borrow().is_some() {
            return;
        }
        let Src::Mugen { vfs, def } = &self.src else {
            return self.ensure_portrait();
        };
        if cfg!(target_arch = "wasm32") {
            return self.ensure_portrait();
        }
        if self.jobs.borrow().portrait.is_some() {
            self.reap();
            return;
        }
        let (vfs, def) = (vfs.clone(), def.clone());
        if let Ok(h) = std::thread::Builder::new()
            .name("char-portrait".into())
            .spawn(move || load_mugen_portrait(&vfs, &def))
        {
            self.jobs.borrow_mut().portrait = Some(h);
        }
    }
    /// Collects finished background jobs (previews and portraits) into the caches.
    pub fn reap(&self) {
        let (done_load, done_portrait) = {
            let mut j = self.jobs.borrow_mut();
            let pals: Vec<i32> = j
                .load
                .iter()
                .filter(|(_, h)| h.is_finished())
                .map(|(p, _)| *p)
                .collect();
            let loads: Vec<(i32, std::thread::JoinHandle<Result<CharDef, String>>)> = pals
                .into_iter()
                .filter_map(|p| j.load.remove(&p).map(|h| (p, h)))
                .collect();
            let portrait = if j.portrait.as_ref().is_some_and(|h| h.is_finished()) {
                j.portrait.take()
            } else {
                None
            };
            (loads, portrait)
        };
        for (pal, h) in done_load {
            let _ = self.store(
                pal,
                h.join()
                    .unwrap_or_else(|_| Err("loader thread panicked".into())),
            );
        }
        if let Some(h) = done_portrait {
            *self.portrait.borrow_mut() = Some(h.join().ok().flatten());
        }
    }
    pub fn errs_has(&self, pal: i32) -> bool {
        self.errs.borrow().contains_key(&pal)
    }
    pub fn streaming(&self) -> bool {
        let j = self.jobs.borrow();
        !j.load.is_empty() || j.portrait.is_some()
    }
    /// Cached select-grid thumbnail (MUGEN only); `None` until [`Self::ensure_portrait`] ran or when the character has no usable sprite.
    pub fn portrait(&self) -> Option<Sprite> {
        self.portrait.borrow().clone().flatten()
    }
    pub fn has_portrait_attempt(&self) -> bool {
        self.portrait.borrow().is_some()
    }
    /// Decode the thumbnail without a full character load (cheap enough for one per frame).
    pub fn ensure_portrait(&self) {
        if self.portrait.borrow().is_some() {
            return;
        }
        let p = match &self.src {
            Src::Mugen { vfs, def } => load_mugen_portrait(vfs, def),
            Src::Native(_) => None,
        };
        *self.portrait.borrow_mut() = Some(p);
    }
    pub fn is_loaded(&self) -> bool {
        !self.cache.borrow().is_empty() || !self.preview.borrow().is_empty()
    }
    fn touch(&self) {
        thread_local! { static CLOCK: Cell<u64> = const { Cell::new(0) }; }
        CLOCK.with(|c| {
            c.set(c.get() + 1);
            self.last_use.set(c.get());
        });
    }
    pub fn unload(&self) {
        self.cache.borrow_mut().clear();
        self.preview.borrow_mut().clear();
    }
}

fn portrait_of(d: &CharDef) -> Option<Sprite> {
    let pick = |k: (i32, i32)| {
        d.sprites
            .get(&k)
            .filter(|s| {
                s.image
                    .as_ref()
                    .is_some_and(|i| i.rgba.as_chunks::<4>().0.iter().any(|p| p[3] > 0))
            })
            .cloned()
    };
    pick((9000, 1)).or_else(|| pick((9000, 0))).or_else(|| {
        d.anims
            .get(&0)
            .and_then(|a| a.frames.first())
            .and_then(|f| pick((f.group, f.index)))
    })
}

pub enum StageSrc {
    Builtin(usize),
    Mugen { vfs: Arc<Vfs>, def: String },
}
pub struct StageEntry {
    pub key: String,
    pub id: String,
    pub name: String,
    pub thumb: [[u8; 3]; 3],
    pub mugen: bool,
    pub src: StageSrc,
    preview: RefCell<Option<Option<Arc<ume_engine::chardef::ImageData>>>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub chars: usize,
    pub stages: usize,
    pub errors: usize,
    pub duplicate_pack: bool,
}

pub struct Registry {
    pub chars: Vec<CharEntry>,
    pub stages: Vec<StageEntry>,
    builtin_stages: Vec<stages::StageEntry>,
    imported_packs: std::collections::HashSet<String>,
    pub import_errors: Vec<String>,
}

fn hex3(h: &str) -> [u8; 3] {
    let v = u32::from_str_radix(h.trim_start_matches('#'), 16).unwrap_or(0x5a648c);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

impl Registry {
    pub fn new() -> Self {
        let mut chars = vec![];
        for (_, json) in BUILTIN.iter() {
            match serde_json::from_str::<NativeChar>(json) {
                Ok(nc) => chars.push(CharEntry {
                    key: format!("umf:{}", nc.id),
                    id: format!("umf:{}", nc.id),
                    name: nc.name.clone(),
                    series: nc.series.clone(),
                    author: nc.author.clone(),
                    color: hex3(&nc.color),
                    description: nc.description.clone().unwrap_or_default(),
                    mugen: false,
                    boss: nc.boss,
                    src: Src::Native(Arc::new(nc)),
                    cache: RefCell::new(HashMap::new()),
                    preview: RefCell::new(HashMap::new()),
                    last_use: Cell::new(0),
                    portrait: RefCell::new(None),
                    broken: Cell::new(false),
                    errs: RefCell::new(HashMap::new()),
                    jobs: Default::default(),
                    sprite_revision: Cell::new(0),
                }),
                Err(e) => eprintln!("bad builtin char: {e}"),
            }
        }
        let builtin_stages = stages::builtin();
        let stages = builtin_stages
            .iter()
            .enumerate()
            .map(|(i, s)| StageEntry {
                key: format!("native:{}", s.id),
                id: format!("native:{}", s.id),
                name: s.name.clone(),
                thumb: s.thumb,
                mugen: false,
                src: StageSrc::Builtin(i),
                preview: RefCell::new(None),
            })
            .collect();
        Registry {
            chars,
            stages,
            builtin_stages,
            imported_packs: Default::default(),
            import_errors: Default::default(),
        }
    }
    /// Bundled, redistributable MUGEN content (Elecbyte Kung Fu Man + stages, CC BY-NC).
    pub fn add_bundled_mugen(&mut self) {
        let mut vfs = Vfs::new();
        if ume_formats::vfs::add_zip(&mut vfs, include_bytes!("../assets/mugen/elecbyte.zip"), "")
            .is_ok()
        {
            self.add_vfs_lazy(vfs, "bundled");
        }
    }
    pub fn add_native_json(&mut self, json: &str) -> Result<String, String> {
        let nc: NativeChar = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = format!("umf:{}", nc.id);
        if let Some(old) = self.chars.iter().find(|c| c.id == id) {
            eprintln!(
                "replacing character '{}' with the imported UMF definition",
                old.name
            );
        }
        self.chars.retain(|c| c.id != id);
        let name = nc.name.clone();
        self.chars.push(CharEntry {
            key: id.clone(),
            id,
            name: nc.name.clone(),
            series: nc.series.clone(),
            author: nc.author.clone(),
            color: hex3(&nc.color),
            description: nc.description.clone().unwrap_or_default(),
            mugen: false,
            boss: nc.boss,
            src: Src::Native(Arc::new(nc)),
            cache: RefCell::new(HashMap::new()),
            preview: RefCell::new(HashMap::new()),
            last_use: Cell::new(0),
            portrait: RefCell::new(None),
            broken: Cell::new(false),
            errs: RefCell::new(HashMap::new()),
            jobs: Default::default(),
            sprite_revision: Cell::new(0),
        });
        Ok(name)
    }
    /// Idempotent .zip import: the same archive bytes are processed once per session, and fighters/stages already in the roster (same content key) are skipped.
    pub fn import_zip(&mut self, bytes: &[u8]) -> Result<ImportReport, String> {
        let (pack, vfs) = read_zip(bytes)?;
        Ok(self.finish_import(pack, vfs))
    }
    pub fn finish_import(&mut self, pack: String, vfs: Vfs) -> ImportReport {
        if !self.imported_packs.insert(pack.clone()) {
            return ImportReport {
                duplicate_pack: true,
                ..Default::default()
            };
        }
        let (chars, stages, errors) = self.add_vfs_lazy(vfs, &format!("pack-{pack}"));
        ImportReport {
            chars: chars.len(),
            stages: stages.len(),
            errors: errors.len(),
            duplicate_pack: false,
        }
    }
    /// Drop repeated entries that share a content key, keeping the first occurrence (cleans rosters built by older versions).
    pub fn dedupe(&mut self) -> usize {
        let before = self.chars.len() + self.stages.len();
        let mut seen = std::collections::HashSet::new();
        self.chars.retain(|c| seen.insert(c.key.clone()));
        seen.clear();
        self.stages.retain(|s| seen.insert(s.key.clone()));
        before - self.chars.len() - self.stages.len()
    }
    /// Like add_vfs but skips the full validation decode (fast startup for trusted bundled content).
    pub fn add_vfs_lazy(&mut self, vfs: Vfs, tag: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
        self.add_vfs_inner(vfs, tag, false)
    }
    /// Register every character and stage found in a VFS. Returns (chars, stages, errors).
    pub fn add_vfs(&mut self, vfs: Vfs, tag: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
        self.add_vfs_inner(vfs, tag, true)
    }
    fn add_vfs_inner(
        &mut self,
        vfs: Vfs,
        tag: &str,
        validate: bool,
    ) -> (Vec<String>, Vec<String>, Vec<String>) {
        let vfs = Arc::new(vfs);
        let (mut cs, mut ss, mut errs) = (vec![], vec![], vec![]);
        for def in find_char_defs(&vfs) {
            let secs = parse_ini(&text(vfs.get(&def).map(|v| v.as_slice()).unwrap_or(&[])));
            let info = find_section(&secs, "info");
            let mut name = info
                .and_then(|s| s.get("displayname").or(s.get("name")))
                .map(unquote)
                .filter(|s| !s.is_empty())
                .unwrap_or(def.clone());
            name = pretty_name(&name);
            if ume_formats::ini::nums(info.and_then(|s| s.get("localcoord")), &[320.0])[0] > 320.0 {
                name.push_str(" HD");
            }
            let author = info
                .and_then(|s| s.get("author"))
                .map(unquote)
                .unwrap_or_default();
            let id = format!("mugen:{tag}:{def}");
            let key = char_key(&vfs, &def);
            if self.chars.iter().any(|c| c.id == id || c.key == key) {
                continue;
            }
            let e = CharEntry {
                key,
                id,
                name: name.clone(),
                series: "M.U.G.E.N".into(),
                author,
                color: [90, 110, 150],
                description: "Personagem M.U.G.E.N importado".into(),
                mugen: true,
                boss: false,
                src: Src::Mugen {
                    vfs: vfs.clone(),
                    def: def.clone(),
                },
                cache: RefCell::new(HashMap::new()),
                preview: RefCell::new(HashMap::new()),
                last_use: Cell::new(0),
                portrait: RefCell::new(None),
                broken: Cell::new(false),
                errs: RefCell::new(HashMap::new()),
                jobs: Default::default(),
                sprite_revision: Cell::new(0),
            };
            if !validate {
                self.chars.push(e);
                cs.push(name);
                continue;
            }
            match e.load(1) {
                Ok(_) => {
                    e.unload();
                    self.chars.push(e);
                    cs.push(name);
                }
                Err(x) => errs.push(format!("{def}: {x}")),
            }
        }
        for def in find_stage_defs(&vfs) {
            let id = format!("mugenstage:{tag}:{def}");
            let key = stage_key(&vfs, &def);
            if self.stages.iter().any(|s| s.id == id || s.key == key) {
                continue;
            }
            if !validate {
                let secs = parse_ini(&text(vfs.get(&def).map(|v| v.as_slice()).unwrap_or(&[])));
                let info = find_section(&secs, "info");
                let name = info
                    .and_then(|s| s.get("displayname").or(s.get("name")))
                    .map(unquote)
                    .filter(|s| !s.is_empty())
                    .unwrap_or(def.clone());
                ss.push(name.clone());
                self.stages.push(StageEntry {
                    key,
                    id,
                    name,
                    thumb: [[40, 40, 60], [20, 20, 30], [60, 60, 90]],
                    mugen: true,
                    src: StageSrc::Mugen {
                        vfs: vfs.clone(),
                        def,
                    },
                    preview: RefCell::new(None),
                });
                continue;
            }
            match load_stage(&vfs, &def) {
                Ok(st) => {
                    let prev = stages::mugen_preview(&st);
                    ss.push(st.name.clone());
                    self.stages.push(StageEntry {
                        key,
                        id,
                        name: st.name,
                        thumb: [[40, 40, 60], [20, 20, 30], [60, 60, 90]],
                        mugen: true,
                        src: StageSrc::Mugen {
                            vfs: vfs.clone(),
                            def,
                        },
                        preview: RefCell::new(Some(prev)),
                    });
                }
                Err(x) => errs.push(format!("{def}: {x}")),
            }
        }
        (cs, ss, errs)
    }
    /// Keep memory bounded with hundreds of fighters: drop the least-recently-used definitions beyond the caps (never `keep`).
    pub fn reap_streams(&self) {
        for c in &self.chars {
            if c.streaming() {
                c.reap();
            }
        }
    }
    pub fn trim_cache(&self, keep: &[usize]) {
        const CAP_MUGEN: usize = 6;
        const CAP_NATIVE: usize = 40;
        for (mugen, cap) in [(true, CAP_MUGEN), (false, CAP_NATIVE)] {
            let total = self
                .chars
                .iter()
                .filter(|c| c.mugen == mugen && c.is_loaded())
                .count();
            if total <= cap {
                continue;
            }
            let mut cand: Vec<(u64, usize)> = self
                .chars
                .iter()
                .enumerate()
                .filter(|(i, c)| c.mugen == mugen && c.is_loaded() && !keep.contains(i))
                .map(|(i, c)| (c.last_use.get(), i))
                .collect();
            cand.sort();
            for (_, i) in cand.into_iter().take(total - cap) {
                self.chars[i].unload();
            }
        }
    }
    pub fn remove_tag(&mut self, tag: &str) -> (usize, usize) {
        let (chars_prefix, stages_prefix) = (format!("mugen:{tag}:"), format!("mugenstage:{tag}:"));
        let (c0, s0) = (self.chars.len(), self.stages.len());
        self.chars.retain(|c| !c.id.starts_with(&chars_prefix));
        self.stages.retain(|s| !s.id.starts_with(&stages_prefix));
        (c0 - self.chars.len(), s0 - self.stages.len())
    }
    /// Free every cached definition (e.g. when the art quality changes → sprites must be re-rasterized).
    pub fn unload_all(&self) {
        for c in &self.chars {
            c.unload();
        }
    }
    /// Imported fighters sorted by name (stable, natives keep their curated order).
    pub fn sort_mugen(&mut self) {
        let n = self.chars.iter().take_while(|c| !c.mugen).count();
        self.chars[n..].sort_by_key(|c| c.name.to_lowercase());
        self.stages.sort_by_key(|s| {
            (
                s.mugen,
                if s.mugen {
                    s.name.to_lowercase()
                } else {
                    String::new()
                },
            )
        });
    }
    pub fn stage_preview(&self, i: usize) -> Option<Arc<ume_engine::chardef::ImageData>> {
        match &self.stages.get(i)?.src {
            StageSrc::Builtin(b) => stages::illustration_preview(&self.builtin_stages[*b].id),
            StageSrc::Mugen { vfs, def } => {
                if let Some(p) = self.stages[i].preview.borrow().as_ref() {
                    return p.clone();
                }
                let p = load_stage(vfs, def)
                    .ok()
                    .and_then(|st| stages::mugen_preview(&st));
                *self.stages[i].preview.borrow_mut() = Some(p.clone());
                p
            }
        }
    }
    pub fn prepare_stage(&self, i: usize) -> StageJob {
        #[cfg(target_arch = "wasm32")]
        if let StageSrc::Builtin(b) = &self.stages[i].src {
            let id = self.builtin_stages[*b].id.clone();
            return Box::pin(async move { stages::prepare_illustration(&id).await });
        }
        let _ = i;
        Box::pin(async { Ok(()) })
    }
    pub fn build_stage(&self, i: usize) -> Stage {
        match &self.stages[i].src {
            StageSrc::Builtin(b) => (self.builtin_stages[*b].build)(),
            StageSrc::Mugen { vfs, def } => match load_stage(vfs, def) {
                Ok(s) => {
                    if self.stages[i].preview.borrow().is_none() {
                        *self.stages[i].preview.borrow_mut() = Some(stages::mugen_preview(&s));
                    }
                    let mut st = stages::from_mugen(s);
                    st.bgm = stage_bgm(vfs, def).map(Arc::new);
                    st
                }
                Err(e) => {
                    eprintln!(
                        "stage '{}' failed to load ({e}); using the built-in fallback",
                        self.stages[i].name
                    );
                    (self.builtin_stages[0].build)()
                }
            },
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn scan_umf_dir(&mut self, dir: &std::path::Path) -> usize {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return 0;
        };
        let mut files: Vec<std::path::PathBuf> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("json"))
                    && p.file_name().is_some_and(|f| f != "index.json")
            })
            .collect();
        files.sort();
        let mut n = 0;
        for f in files {
            match std::fs::read_to_string(&f)
                .map_err(|e| e.to_string())
                .and_then(|j| self.add_native_json(&j))
            {
                Ok(_) => n += 1,
                Err(e) => eprintln!("umf {}: {e}", f.display()),
            }
        }
        n
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn scan_folders(&mut self) -> usize {
        let mut n = 0;
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        let mut dirs = vec![std::path::PathBuf::from("mugen")];
        if let Some(d) = exe_dir {
            dirs.push(d.join("mugen"));
            if let Some(p) = d.parent().and_then(|p| p.parent()) {
                dirs.push(p.join("mugen"));
            }
        }
        if let Ok(extra) = std::env::var("UME_MUGEN_DIR") {
            dirs.insert(0, extra.into());
        }
        let mut umf_dirs: Vec<std::path::PathBuf> = dirs.iter().map(|d| d.join("umf")).collect();
        umf_dirs.insert(0, "umf".into());
        if let Ok(extra) = std::env::var("UME_UMF_DIR") {
            umf_dirs.insert(0, extra.into());
        }
        for d in umf_dirs {
            n += self.scan_umf_dir(&d);
        }
        for d in dirs {
            if !d.is_dir() {
                continue;
            }
            let mut vfs = Vfs::new();
            match ume_formats::vfs::add_dir(&mut vfs, &d, "") {
                Ok(_) => {
                    let (c, s, errs) = self.add_vfs_lazy(vfs, &d.to_string_lossy());
                    n += c.len() + s.len();
                    self.import_errors
                        .extend(errs.into_iter().map(|e| format!("{}: {e}", d.display())));
                }
                Err(e) => self.import_errors.push(format!("{}: {e}", d.display())),
            }
            break;
        }
        n
    }
}

/// Heavy half of an import (hash + decompression), safe to run on a worker thread.
pub fn read_zip(bytes: &[u8]) -> Result<(String, Vfs), String> {
    let pack = ume_formats::vfs::content_hash(bytes);
    let mut vfs = Vfs::new();
    let clock = ume_shell::Clock::now();
    ume_formats::vfs::add_zip(&mut vfs, bytes, "")?;
    if ume_formats::perf::enabled() {
        let mut by_ext: HashMap<String, (usize, usize)> = HashMap::new();
        for (k, v) in &vfs {
            let ext = k.rsplit_once('.').map(|(_, e)| e.to_string()).unwrap_or_else(|| "(none)".into());
            let e = by_ext.entry(ext).or_default();
            e.0 += 1;
            e.1 += v.len();
        }
        let mut rows: Vec<_> = by_ext.into_iter().collect();
        rows.sort_by_key(|(_, (_, b))| std::cmp::Reverse(*b));
        let total: usize = rows.iter().map(|(_, (_, b))| *b).sum();
        let listing: Vec<String> = rows.iter().take(12).map(|(e, (n, b))| format!("{e}:{n}/{:.1}MB", *b as f64 / 1048576.0)).collect();
        ume_formats::perf::emit(&format!("[perf] zip {:.1}MB -> vfs {:.1}MB in {:.0} ms ({})", bytes.len() as f64 / 1048576.0, total as f64 / 1048576.0, clock.elapsed().as_secs_f64() * 1000.0, listing.join(" ")));
    }
    Ok((pack, vfs))
}

pub fn stage_bgm(vfs: &Vfs, def: &str) -> Option<(Vec<u8>, MusicLoop)> {
    let secs = parse_ini(&text(vfs.get(&ume_formats::vfs::norm_path(def))?));
    let m = find_section(&secs, "music")?;
    let name = unquote(m.get("bgmusic")?);
    if name.is_empty() {
        return None;
    }
    let base = def.rfind('/').map(|i| &def[..i]).unwrap_or("");
    let bytes = ume_formats::vfs::resolve(vfs, base, &name)
        .or_else(|| ume_formats::vfs::resolve(vfs, "", &name))?;
    let n = |k: &str| {
        m.get(k)
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(0)
    };
    Some((
        bytes.to_vec(),
        MusicLoop {
            start: n("bgmloopstart"),
            end: n("bgmloopend"),
        },
    ))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn loads_umf_json_from_folder_without_rebuild() {
        let dir = std::env::temp_dir().join(format!("ume-umf-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut v: serde_json::Value = serde_json::from_str(BUILTIN[0].1).unwrap();
        v["id"] = "folder_test".into();
        v["name"] = "Folder Test".into();
        std::fs::write(dir.join("folder_test.json"), v.to_string()).unwrap();
        std::fs::write(dir.join("broken.json"), "{").unwrap();
        let mut reg = Registry::new();
        let before = reg.chars.len();
        assert_eq!(reg.scan_umf_dir(&dir), 1);
        assert_eq!(reg.chars.len(), before + 1);
        let e = reg
            .chars
            .iter()
            .find(|c| c.id == "umf:folder_test")
            .unwrap();
        assert_eq!(e.name, "Folder Test");
        assert!(e.load(1).is_ok());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn mugen_previews_stream_without_blocking_the_caller() {
        let zip = include_bytes!("../assets/mugen/elecbyte.zip");
        let mut reg = Registry::new();
        reg.import_zip(zip).unwrap();
        let e = reg
            .chars
            .iter()
            .find(|c| c.mugen && !c.broken.get())
            .unwrap();
        let t = std::time::Instant::now();
        e.stream_preview(1);
        e.stream_portrait();
        let call_ms = t.elapsed().as_secs_f64() * 1000.0;
        assert!(call_ms < 25.0, "stream calls took {call_ms} ms");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while (e.peek(1).is_none() || !e.has_portrait_attempt())
            && std::time::Instant::now() < deadline
        {
            e.reap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(e.peek(1).is_some() && e.has_portrait_attempt() && !e.streaming());
    }

    #[test]
    fn read_zip_matches_import_zip() {
        let zip = include_bytes!("../assets/mugen/elecbyte.zip");
        let (pack, vfs) = read_zip(zip).unwrap();
        let mut reg = Registry::new();
        let r = reg.finish_import(pack.clone(), vfs);
        assert!(r.chars > 0 && !r.duplicate_pack);
        let (_, vfs2) = read_zip(zip).unwrap();
        assert!(reg.finish_import(pack, vfs2).duplicate_pack);
    }

    #[test]
    fn importing_the_same_pack_twice_does_not_duplicate() {
        let zip = include_bytes!("../assets/mugen/elecbyte.zip");
        let mut reg = Registry::new();
        let r1 = reg.import_zip(zip).unwrap();
        assert!(r1.chars > 0 && !r1.duplicate_pack);
        let (c, s) = (reg.chars.len(), reg.stages.len());
        let r2 = reg.import_zip(zip).unwrap();
        assert!(r2.duplicate_pack);
        assert_eq!((reg.chars.len(), reg.stages.len()), (c, s));
    }

    #[test]
    fn same_content_from_another_source_is_skipped_and_dedupe_cleans_old_rosters() {
        let load = || {
            let mut v = Vfs::new();
            ume_formats::vfs::add_zip(&mut v, include_bytes!("../assets/mugen/elecbyte.zip"), "")
                .unwrap();
            v
        };
        let mut reg = Registry::new();
        reg.add_vfs_lazy(load(), "bundled");
        let (c, s) = (reg.chars.len(), reg.stages.len());
        reg.add_vfs_lazy(load(), "web123-0");
        reg.add_vfs_lazy(load(), "/some/other/dir");
        assert_eq!((reg.chars.len(), reg.stages.len()), (c, s));
        let dup_key = reg.chars.iter().find(|c| c.mugen).unwrap().key.clone();
        let e = CharEntry {
            key: dup_key,
            id: "mugen:legacy:chars/kfm/kfm.def".into(),
            name: "Legacy".into(),
            series: String::new(),
            author: String::new(),
            color: [0; 3],
            description: String::new(),
            mugen: true,
            boss: false,
            src: Src::Mugen {
                vfs: Arc::new(Vfs::new()),
                def: String::new(),
            },
            cache: RefCell::new(HashMap::new()),
            preview: RefCell::new(HashMap::new()),
            last_use: Cell::new(0),
            portrait: RefCell::new(None),
            broken: Cell::new(false),
            errs: RefCell::new(HashMap::new()),
            jobs: Default::default(),
            sprite_revision: Cell::new(0),
        };
        reg.chars.push(e);
        assert_eq!(reg.dedupe(), 1);
        assert_eq!(reg.chars.len(), c);
    }

    #[test]
    fn scanning_folders_repeatedly_is_idempotent() {
        let mut reg = Registry::new();
        reg.add_bundled_mugen();
        reg.scan_folders();
        let (c, s) = (reg.chars.len(), reg.stages.len());
        reg.scan_folders();
        reg.add_bundled_mugen();
        assert_eq!((reg.chars.len(), reg.stages.len()), (c, s));
    }

    #[test]
    fn mugen_stages_produce_cached_previews() {
        let mut reg = Registry::new();
        reg.add_bundled_mugen();
        reg.scan_folders();
        let mugen_indices: Vec<usize> = reg
            .stages
            .iter()
            .enumerate()
            .filter(|(_, s)| s.mugen)
            .map(|(i, _)| i)
            .collect();
        assert!(!mugen_indices.is_empty());
        for i in mugen_indices {
            let p1 = reg
                .stage_preview(i)
                .unwrap_or_else(|| panic!("missing preview for {}", reg.stages[i].id));
            let p2 = reg.stage_preview(i).unwrap();
            assert_eq!(p1.id, p2.id);
            assert_eq!((p1.w, p1.h), (320, 180));
            assert!(p1
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .any(|px| px[..3] != [20, 20, 30]));
        }
    }

    #[test]
    fn mugen_preview_handles_extreme_tiling_and_position_link_visibility() {
        let mut vfs = Vfs::new();
        ume_formats::vfs::add_zip(&mut vfs, include_bytes!("../assets/mugen/elecbyte.zip"), "")
            .unwrap();
        let mut st = load_stage(&vfs, "stages/stage0.def").unwrap();
        if st.layers.len() > 1 {
            st.layers[0].id = 77;
            st.layers[1].position_link = true;
            st.ctrl_defs.push(ume_formats::stage::BgCtrlDef {
                loop_time: -1,
                ids: vec![77],
                ctrls: vec![ume_formats::stage::BgCtrl {
                    kind: ume_formats::stage::BgCtrlKind::Visible(false),
                    start: 0,
                    end: 0,
                    loop_time: -1,
                    ids: vec![77],
                }],
            });
        }
        let last = st.layers.last_mut().unwrap();
        last.tile = [i32::MAX, i32::MAX];
        last.start = [-1e20, 0.0];
        let prev = stages::mugen_preview(&st);
        if st.layers.len() <= 2 {
            assert!(prev.is_none());
        } else {
            assert!(prev.is_some());
        }
    }
}

#[cfg(test)]
#[path = "registry_sprite_cache_tests.rs"]
mod sprite_cache_tests;
