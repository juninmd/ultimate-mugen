//! Character & stage registry: built-in UMF fighters (embedded JSON) + MUGEN content (chars/ and stages/ folders, imports).
use crate::stages::{self, Stage};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::loader::load_mugen_char;
use ume_engine::native::{compile_native, NativeChar};
use ume_formats::ini::{find_section, parse_ini, unquote};
use ume_formats::stage::load_stage;
use ume_formats::vfs::{find_char_defs, find_stage_defs, text, Vfs};

pub const BUILTIN: [(&str, &str); 10] = [
    ("kaito", include_str!("../assets/chars/kaito.json")), ("shion", include_str!("../assets/chars/shion.json")),
    ("akio", include_str!("../assets/chars/akio.json")), ("suzume", include_str!("../assets/chars/suzume.json")),
    ("bruno", include_str!("../assets/chars/bruno.json")), ("sylene", include_str!("../assets/chars/sylene.json")),
    ("volta", include_str!("../assets/chars/volta.json")), ("lian", include_str!("../assets/chars/lian.json")),
    ("garan", include_str!("../assets/chars/garan.json")), ("marlow", include_str!("../assets/chars/marlow.json")),
];

pub enum Src { Native(Box<NativeChar>), Mugen { vfs: Arc<Vfs>, def: String } }

pub struct CharEntry {
    pub id: String, pub name: String, pub series: String, pub author: String, pub color: [u8; 3], pub description: String, pub mugen: bool,
    pub src: Src,
    cache: RefCell<HashMap<i32, Arc<CharDef>>>,
}
impl CharEntry {
    pub fn load(&self, pal: i32) -> Result<Arc<CharDef>, String> {
        if let Some(d) = self.cache.borrow().get(&pal) { return Ok(d.clone()); }
        let d = Arc::new(match &self.src {
            Src::Native(nc) => compile_native(nc, pal, true),
            Src::Mugen { vfs, def } => load_mugen_char(vfs, def, pal, true)?,
        });
        self.cache.borrow_mut().insert(pal, d.clone());
        Ok(d)
    }
    pub fn unload(&self) { self.cache.borrow_mut().clear(); }
}

pub enum StageSrc { Builtin(usize), Mugen { vfs: Arc<Vfs>, def: String } }
pub struct StageEntry { pub id: String, pub name: String, pub thumb: [[u8; 3]; 3], pub mugen: bool, pub src: StageSrc }

pub struct Registry { pub chars: Vec<CharEntry>, pub stages: Vec<StageEntry>, builtin_stages: Vec<stages::StageEntry> }

fn hex3(h: &str) -> [u8; 3] { let v = u32::from_str_radix(h.trim_start_matches('#'), 16).unwrap_or(0x5a648c); [(v >> 16) as u8, (v >> 8) as u8, v as u8] }

impl Registry {
    pub fn new() -> Self {
        let mut chars = vec![];
        for (_, json) in BUILTIN {
            match serde_json::from_str::<NativeChar>(json) {
                Ok(nc) => chars.push(CharEntry {
                    id: format!("umf:{}", nc.id), name: nc.name.clone(), series: nc.series.clone(), author: nc.author.clone(), color: hex3(&nc.color),
                    description: nc.description.clone().unwrap_or_default(), mugen: false, src: Src::Native(Box::new(nc)), cache: RefCell::new(HashMap::new()),
                }),
                Err(e) => eprintln!("bad builtin char: {e}"),
            }
        }
        let builtin_stages = stages::builtin();
        let stages = builtin_stages.iter().enumerate().map(|(i, s)| StageEntry { id: format!("native:{}", s.id), name: s.name.clone(), thumb: s.thumb, mugen: false, src: StageSrc::Builtin(i) }).collect();
        Registry { chars, stages, builtin_stages }
    }
    pub fn add_native_json(&mut self, json: &str) -> Result<String, String> {
        let nc: NativeChar = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = format!("umf:{}", nc.id);
        self.chars.retain(|c| c.id != id);
        let name = nc.name.clone();
        self.chars.push(CharEntry { id, name: nc.name.clone(), series: nc.series.clone(), author: nc.author.clone(), color: hex3(&nc.color), description: nc.description.clone().unwrap_or_default(), mugen: false, src: Src::Native(Box::new(nc)), cache: RefCell::new(HashMap::new()) });
        Ok(name)
    }
    /// Register every character and stage found in a VFS. Returns (chars, stages, errors).
    pub fn add_vfs(&mut self, vfs: Vfs, tag: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
        let vfs = Arc::new(vfs);
        let (mut cs, mut ss, mut errs) = (vec![], vec![], vec![]);
        for def in find_char_defs(&vfs) {
            let secs = parse_ini(&text(vfs.get(&def).map(|v| v.as_slice()).unwrap_or(&[])));
            let info = find_section(&secs, "info");
            let name = info.and_then(|s| s.get("displayname").or(s.get("name"))).map(unquote).filter(|s| !s.is_empty()).unwrap_or(def.clone());
            let author = info.and_then(|s| s.get("author")).map(unquote).unwrap_or_default();
            let id = format!("mugen:{tag}:{def}");
            if self.chars.iter().any(|c| c.id == id) { continue; }
            let e = CharEntry { id, name: name.clone(), series: "M.U.G.E.N".into(), author, color: [90, 110, 150], description: "Personagem M.U.G.E.N importado".into(), mugen: true, src: Src::Mugen { vfs: vfs.clone(), def: def.clone() }, cache: RefCell::new(HashMap::new()) };
            match e.load(1) { Ok(_) => { e.unload(); self.chars.push(e); cs.push(name); } Err(x) => errs.push(format!("{def}: {x}")) }
        }
        for def in find_stage_defs(&vfs) {
            let id = format!("mugenstage:{tag}:{def}");
            if self.stages.iter().any(|s| s.id == id) { continue; }
            match load_stage(&vfs, &def) {
                Ok(st) => { ss.push(st.name.clone()); self.stages.push(StageEntry { id, name: st.name, thumb: [[40, 40, 60], [20, 20, 30], [60, 60, 90]], mugen: true, src: StageSrc::Mugen { vfs: vfs.clone(), def } }); }
                Err(x) => errs.push(format!("{def}: {x}")),
            }
        }
        (cs, ss, errs)
    }
    pub fn build_stage(&self, i: usize) -> Stage {
        match &self.stages[i].src {
            StageSrc::Builtin(b) => (self.builtin_stages[*b].build)(),
            StageSrc::Mugen { vfs, def } => match load_stage(vfs, def) { Ok(s) => stages::from_mugen(s), Err(_) => (self.builtin_stages[0].build)() },
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn scan_folders(&mut self) -> usize {
        let mut n = 0;
        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
        let mut dirs = vec![std::path::PathBuf::from("mugen")];
        if let Some(d) = exe_dir { dirs.push(d.join("mugen")); }
        if let Ok(extra) = std::env::var("UME_MUGEN_DIR") { dirs.insert(0, extra.into()); }
        for d in dirs {
            if !d.is_dir() { continue; }
            let mut vfs = Vfs::new();
            if ume_formats::vfs::add_dir(&mut vfs, &d, "").is_ok() { let (c, s, _) = self.add_vfs(vfs, &d.to_string_lossy()); n += c.len() + s.len(); }
            break;
        }
        n
    }
}
