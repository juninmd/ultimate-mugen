//! Character & stage registry: built-in UMF fighters (embedded JSON) + MUGEN content (chars/ and stages/ folders, imports).
use crate::stages::{self, Stage};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;
use ume_engine::chardef::{CharDef, Sprite};
use ume_engine::loader::{load_mugen_char, load_mugen_portrait};
use ume_engine::native::{compile_native, compile_native_preview, NativeChar};
use ume_formats::ini::{find_section, parse_ini, unquote};
use ume_formats::stage::load_stage;
use ume_formats::vfs::{find_char_defs, find_stage_defs, text, Vfs};

include!(concat!(env!("OUT_DIR"), "/chars_gen.rs"));

/// "true_z_license_free_chara" -> "True Z", "Thunder_horse_GGX" -> "Thunder Horse GGX"
pub fn pretty_name(n: &str) -> String {
    let mut s = n.replace(['_', '-'], " ");
    for junk in ["license free mugen chara for official", "license free chara", "license free", "for official"] {
        if let Some(i) = s.to_ascii_lowercase().find(junk) { s.replace_range(i..i + junk.len(), ""); }
    }
    let words: Vec<String> = s.split_whitespace().map(|w| {
        if w.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) && w.len() > 1 { w.to_string() }
        else { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }
    }).collect();
    if words.is_empty() { n.to_string() } else { words.join(" ") }
}

pub enum Src { Native(Box<NativeChar>), Mugen { vfs: Arc<Vfs>, def: String } }

pub struct CharEntry {
    pub id: String, pub name: String, pub series: String, pub author: String, pub color: [u8; 3], pub description: String, pub mugen: bool, /** final boss: hidden from the roster */ pub boss: bool,
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
}
impl CharEntry {
    pub fn load(&self, pal: i32) -> Result<Arc<CharDef>, String> {
        if let Some(d) = self.cache.borrow().get(&pal) { return Ok(d.clone()); }
        if let Some(e) = self.errs.borrow().get(&pal) { return Err(e.clone()); }
        let d = Arc::new(match &self.src {
            Src::Native(nc) => compile_native(nc, pal, true),
            Src::Mugen { vfs, def } => match load_mugen_char(vfs, def, pal, true) {
                Ok(mut d) => { d.display_name = self.name.clone(); d }
                Err(e) => { self.broken.set(true); self.errs.borrow_mut().insert(pal, e.clone()); return Err(e); }
            },
        });
        if self.mugen && self.portrait.borrow().is_none() { *self.portrait.borrow_mut() = Some(portrait_of(&d)); }
        self.cache.borrow_mut().insert(pal, d.clone());
        self.preview.borrow_mut().remove(&pal);
        self.touch();
        Ok(d)
    }
    /// Cheap definition for menus: full if already loaded, else preview (natives) or a full load (MUGEN).
    pub fn load_preview(&self, pal: i32) -> Result<Arc<CharDef>, String> {
        if let Some(d) = self.peek(pal) { self.touch(); return Ok(d); }
        match &self.src {
            Src::Native(nc) => { let d = Arc::new(compile_native_preview(nc, pal)); self.preview.borrow_mut().insert(pal, d.clone()); self.touch(); Ok(d) }
            Src::Mugen { .. } => self.load(pal),
        }
    }
    /// Cached select-grid thumbnail (MUGEN only); `None` until [`Self::ensure_portrait`] ran or when the character has no usable sprite.
    pub fn portrait(&self) -> Option<Sprite> { self.portrait.borrow().clone().flatten() }
    pub fn has_portrait_attempt(&self) -> bool { self.portrait.borrow().is_some() }
    /// Decode the thumbnail without a full character load (cheap enough for one per frame).
    pub fn ensure_portrait(&self) {
        if self.portrait.borrow().is_some() { return; }
        let p = match &self.src { Src::Mugen { vfs, def } => load_mugen_portrait(vfs, def), Src::Native(_) => None };
        *self.portrait.borrow_mut() = Some(p);
    }
    /// Already-loaded definition (full or preview), never loads.
    pub fn peek(&self, pal: i32) -> Option<Arc<CharDef>> {
        if let Some(d) = self.cache.borrow().get(&pal) { return Some(d.clone()); }
        self.preview.borrow().get(&pal).cloned()
    }
    pub fn is_loaded(&self) -> bool { !self.cache.borrow().is_empty() || !self.preview.borrow().is_empty() }
    fn touch(&self) { thread_local! { static CLOCK: Cell<u64> = const { Cell::new(0) }; } CLOCK.with(|c| { c.set(c.get() + 1); self.last_use.set(c.get()); }); }
    pub fn unload(&self) { self.cache.borrow_mut().clear(); self.preview.borrow_mut().clear(); }
}

fn portrait_of(d: &CharDef) -> Option<Sprite> {
    let pick = |k: (i32, i32)| d.sprites.get(&k).filter(|s| s.image.as_ref().is_some_and(|i| i.rgba.chunks_exact(4).any(|p| p[3] > 0))).cloned();
    pick((9000, 1)).or_else(|| pick((9000, 0))).or_else(|| d.anims.get(&0).and_then(|a| a.frames.first()).and_then(|f| pick((f.group, f.index))))
}

pub enum StageSrc { Builtin(usize), Mugen { vfs: Arc<Vfs>, def: String } }
pub struct StageEntry { pub id: String, pub name: String, pub thumb: [[u8; 3]; 3], pub mugen: bool, pub src: StageSrc }

pub struct Registry { pub chars: Vec<CharEntry>, pub stages: Vec<StageEntry>, builtin_stages: Vec<stages::StageEntry> }

fn hex3(h: &str) -> [u8; 3] { let v = u32::from_str_radix(h.trim_start_matches('#'), 16).unwrap_or(0x5a648c); [(v >> 16) as u8, (v >> 8) as u8, v as u8] }

impl Registry {
    pub fn new() -> Self {
        let mut chars = vec![];
        for (_, json) in BUILTIN.iter() {
            match serde_json::from_str::<NativeChar>(json) {
                Ok(nc) => chars.push(CharEntry {
                    id: format!("umf:{}", nc.id), name: nc.name.clone(), series: nc.series.clone(), author: nc.author.clone(), color: hex3(&nc.color),
                    description: nc.description.clone().unwrap_or_default(), mugen: false, boss: nc.boss, src: Src::Native(Box::new(nc)), cache: RefCell::new(HashMap::new()), preview: RefCell::new(HashMap::new()), last_use: Cell::new(0), portrait: RefCell::new(None), broken: Cell::new(false), errs: RefCell::new(HashMap::new()),
                }),
                Err(e) => eprintln!("bad builtin char: {e}"),
            }
        }
        let builtin_stages = stages::builtin();
        let stages = builtin_stages.iter().enumerate().map(|(i, s)| StageEntry { id: format!("native:{}", s.id), name: s.name.clone(), thumb: s.thumb, mugen: false, src: StageSrc::Builtin(i) }).collect();
        Registry { chars, stages, builtin_stages }
    }
    /// Bundled, redistributable MUGEN content (Elecbyte Kung Fu Man + stages, CC BY-NC).
    pub fn add_bundled_mugen(&mut self) {
        let mut vfs = Vfs::new();
        if ume_formats::vfs::add_zip(&mut vfs, include_bytes!("../assets/mugen/elecbyte.zip"), "").is_ok() {
            self.add_vfs_lazy(vfs, "bundled");
        }
    }
    pub fn add_native_json(&mut self, json: &str) -> Result<String, String> {
        let nc: NativeChar = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = format!("umf:{}", nc.id);
        self.chars.retain(|c| c.id != id);
        let name = nc.name.clone();
        self.chars.push(CharEntry { id, name: nc.name.clone(), series: nc.series.clone(), author: nc.author.clone(), color: hex3(&nc.color), description: nc.description.clone().unwrap_or_default(), mugen: false, boss: nc.boss, src: Src::Native(Box::new(nc)), cache: RefCell::new(HashMap::new()), preview: RefCell::new(HashMap::new()), last_use: Cell::new(0), portrait: RefCell::new(None), broken: Cell::new(false), errs: RefCell::new(HashMap::new()) });
        Ok(name)
    }
    /// Like add_vfs but skips the full validation decode (fast startup for trusted bundled content).
    pub fn add_vfs_lazy(&mut self, vfs: Vfs, tag: &str) -> (Vec<String>, Vec<String>, Vec<String>) { self.add_vfs_inner(vfs, tag, false) }
    /// Register every character and stage found in a VFS. Returns (chars, stages, errors).
    pub fn add_vfs(&mut self, vfs: Vfs, tag: &str) -> (Vec<String>, Vec<String>, Vec<String>) { self.add_vfs_inner(vfs, tag, true) }
    fn add_vfs_inner(&mut self, vfs: Vfs, tag: &str, validate: bool) -> (Vec<String>, Vec<String>, Vec<String>) {
        let vfs = Arc::new(vfs);
        let (mut cs, mut ss, mut errs) = (vec![], vec![], vec![]);
        for def in find_char_defs(&vfs) {
            let secs = parse_ini(&text(vfs.get(&def).map(|v| v.as_slice()).unwrap_or(&[])));
            let info = find_section(&secs, "info");
            let mut name = info.and_then(|s| s.get("displayname").or(s.get("name"))).map(unquote).filter(|s| !s.is_empty()).unwrap_or(def.clone());
            name = pretty_name(&name);
            if ume_formats::ini::nums(info.and_then(|s| s.get("localcoord")), &[320.0])[0] > 320.0 { name.push_str(" HD"); }
            let author = info.and_then(|s| s.get("author")).map(unquote).unwrap_or_default();
            let id = format!("mugen:{tag}:{def}");
            if self.chars.iter().any(|c| c.id == id) { continue; }
            let e = CharEntry { id, name: name.clone(), series: "M.U.G.E.N".into(), author, color: [90, 110, 150], description: "Personagem M.U.G.E.N importado".into(), mugen: true, boss: false, src: Src::Mugen { vfs: vfs.clone(), def: def.clone() }, cache: RefCell::new(HashMap::new()), preview: RefCell::new(HashMap::new()), last_use: Cell::new(0), portrait: RefCell::new(None), broken: Cell::new(false), errs: RefCell::new(HashMap::new()) };
            if !validate { self.chars.push(e); cs.push(name); continue; }
            match e.load(1) { Ok(_) => { e.unload(); self.chars.push(e); cs.push(name); } Err(x) => errs.push(format!("{def}: {x}")) }
        }
        for def in find_stage_defs(&vfs) {
            let id = format!("mugenstage:{tag}:{def}");
            if self.stages.iter().any(|s| s.id == id) { continue; }
            if !validate {
                let secs = parse_ini(&text(vfs.get(&def).map(|v| v.as_slice()).unwrap_or(&[])));
                let info = find_section(&secs, "info");
                let name = info.and_then(|s| s.get("displayname").or(s.get("name"))).map(unquote).filter(|s| !s.is_empty()).unwrap_or(def.clone());
                ss.push(name.clone());
                self.stages.push(StageEntry { id, name, thumb: [[40, 40, 60], [20, 20, 30], [60, 60, 90]], mugen: true, src: StageSrc::Mugen { vfs: vfs.clone(), def } });
                continue;
            }
            match load_stage(&vfs, &def) {
                Ok(st) => { ss.push(st.name.clone()); self.stages.push(StageEntry { id, name: st.name, thumb: [[40, 40, 60], [20, 20, 30], [60, 60, 90]], mugen: true, src: StageSrc::Mugen { vfs: vfs.clone(), def } }); }
                Err(x) => errs.push(format!("{def}: {x}")),
            }
        }
        (cs, ss, errs)
    }
    /// Keep memory bounded with hundreds of fighters: drop the least-recently-used definitions beyond the caps (never `keep`).
    pub fn trim_cache(&self, keep: &[usize]) {
        const CAP_MUGEN: usize = 6; const CAP_NATIVE: usize = 40;
        for (mugen, cap) in [(true, CAP_MUGEN), (false, CAP_NATIVE)] {
            let total = self.chars.iter().filter(|c| c.mugen == mugen && c.is_loaded()).count();
            if total <= cap { continue; }
            let mut cand: Vec<(u64, usize)> = self.chars.iter().enumerate().filter(|(i, c)| c.mugen == mugen && c.is_loaded() && !keep.contains(i)).map(|(i, c)| (c.last_use.get(), i)).collect();
            cand.sort();
            for (_, i) in cand.into_iter().take(total - cap) { self.chars[i].unload(); }
        }
    }
    /// Free every cached definition (e.g. when the art quality changes → sprites must be re-rasterized).
    pub fn unload_all(&self) { for c in &self.chars { c.unload(); } }
    /// Imported fighters sorted by name (stable, natives keep their curated order).
    pub fn sort_mugen(&mut self) {
        let n = self.chars.iter().take_while(|c| !c.mugen).count();
        self.chars[n..].sort_by_key(|c| c.name.to_lowercase());
        self.stages.sort_by_key(|s| (s.mugen, if s.mugen { s.name.to_lowercase() } else { String::new() }));
    }
    pub fn build_stage(&self, i: usize) -> Stage {
        match &self.stages[i].src {
            StageSrc::Builtin(b) => (self.builtin_stages[*b].build)(),
            StageSrc::Mugen { vfs, def } => match load_stage(vfs, def) { Ok(s) => stages::from_mugen(s), Err(_) => (self.builtin_stages[0].build)() },
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn scan_umf_dir(&mut self, dir: &std::path::Path) -> usize {
        let Ok(rd) = std::fs::read_dir(dir) else { return 0 };
        let mut files: Vec<std::path::PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("json")) && p.file_name().is_some_and(|f| f != "index.json")).collect();
        files.sort();
        let mut n = 0;
        for f in files {
            match std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|j| self.add_native_json(&j)) {
                Ok(_) => n += 1,
                Err(e) => eprintln!("umf {}: {e}", f.display()),
            }
        }
        n
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn scan_folders(&mut self) -> usize {
        let mut n = 0;
        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
        let mut dirs = vec![std::path::PathBuf::from("mugen")];
        if let Some(d) = exe_dir {
            dirs.push(d.join("mugen"));
            if let Some(p) = d.parent().and_then(|p| p.parent()) { dirs.push(p.join("mugen")); }
        }
        if let Ok(extra) = std::env::var("UME_MUGEN_DIR") { dirs.insert(0, extra.into()); }
        let mut umf_dirs: Vec<std::path::PathBuf> = dirs.iter().map(|d| d.join("umf")).collect();
        umf_dirs.insert(0, "umf".into());
        if let Ok(extra) = std::env::var("UME_UMF_DIR") { umf_dirs.insert(0, extra.into()); }
        for d in umf_dirs { n += self.scan_umf_dir(&d); }
        for d in dirs {
            if !d.is_dir() { continue; }
            let mut vfs = Vfs::new();
            if ume_formats::vfs::add_dir(&mut vfs, &d, "").is_ok() { let (c, s, _) = self.add_vfs_lazy(vfs, &d.to_string_lossy()); n += c.len() + s.len(); }
            break;
        }
        n
    }
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
        let e = reg.chars.iter().find(|c| c.id == "umf:folder_test").unwrap();
        assert_eq!(e.name, "Folder Test");
        assert!(e.load(1).is_ok());
        std::fs::remove_dir_all(&dir).ok();
    }
}
