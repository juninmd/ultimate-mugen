use crate::source::Source;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::Path;
use ume_engine::loader::{read_mugen_sources, MugenSources, MugenText};
use ume_formats::ini::{find_section, parse_ini_no_lines};
use ume_formats::vfs::text;
use ume_native::exact::{encode_pack, export_anims, MugenFile, MugenSpec, META};

pub struct Bundle {
    pub json: Value,
    pub files: Vec<(String, Vec<u8>)>,
}

fn file(t: &MugenText) -> MugenFile {
    MugenFile { name: t.name.clone(), text: t.text.clone() }
}

fn info_section(src: &Source) -> BTreeMap<String, String> {
    let secs = src.vfs.get(&src.def).map(|b| parse_ini_no_lines(&text(b))).unwrap_or_default();
    find_section(&secs, "info").map(|s| s.entries.iter().map(|(k, v)| (k.clone(), v.trim_matches('"').to_string())).collect()).unwrap_or_default()
}

pub fn spec_of(src: &MugenSources, info: BTreeMap<String, String>) -> MugenSpec {
    MugenSpec {
        info,
        mugen_version: src.mugen_version.clone(),
        localcoord: src.localcoord.clone(),
        stcommon: src.stcommon.as_ref().map(file),
        cns: file(&src.cns),
        st: src.st.iter().map(file).collect(),
        cmd: file(&src.cmd),
        anims: export_anims(&src.anims),
        names: BTreeMap::new(),
    }
}

fn stats(def: &ume_engine::chardef::CharDef) -> Value {
    let k = |key: &str, d: f64| def.constants.get(key).copied().unwrap_or(d);
    json!({
        "life": k("data.life", 1000.0), "power": k("data.power", 3000.0), "attack": k("data.attack", 100.0), "defense": k("data.defence", 100.0),
        "walk": [k("velocity.walk.fwd.x", 2.4), k("velocity.walk.back.x", -2.2).abs()],
        "run": [k("velocity.run.fwd.x", 4.6), k("velocity.run.back.x", -4.5).abs()],
        "jump": [k("velocity.jump.y", -8.4), k("velocity.jump.back.x", -2.55), k("velocity.jump.neu.x", 0.0), k("velocity.jump.fwd.x", 2.5)],
        "height": k("size.height", 60.0), "width": [k("size.ground.front", 16.0), k("size.ground.back", 15.0)],
    })
}

pub fn convert(src: &Source, id: &str, name: Option<&str>) -> Result<Bundle, String> {
    let sources = read_mugen_sources(&src.vfs, &src.def)?;
    let info = info_section(src);
    let def = ume_engine::loader::assemble_mugen_char(sources.clone(), 1, false);
    let display = name.map(str::to_string).unwrap_or_else(|| sources.display_name.clone());
    let (meta, mut files) = match &sources.sff {
        Some(sff) => encode_pack(sff, &sources.sounds, &sources.pal_files),
        None => return Err("the character has no readable .sff".into()),
    };
    files.push((META.into(), serde_json::to_vec(&meta).map_err(|e| e.to_string())?));
    let spec = spec_of(&sources, info);
    let author = Some(sources.author.trim()).filter(|a| !a.is_empty()).unwrap_or("M.U.G.E.N");
    let mut fighter = Map::new();
    fighter.insert("format".into(), json!("umf/2"));
    fighter.insert("id".into(), json!(id));
    fighter.insert("name".into(), json!(display));
    fighter.insert("series".into(), json!("M.U.G.E.N"));
    fighter.insert("author".into(), json!(author));
    fighter.insert("description".into(), json!(format!("{display} migrated from M.U.G.E.N to UMF with identical behavior.")));
    fighter.insert("color".into(), json!("#c8321e"));
    fighter.insert("stats".into(), stats(&def));
    fighter.insert("art".into(), json!({"type": "mugen"}));
    fighter.insert("anims".into(), json!({}));
    fighter.insert("moves".into(), json!([]));
    fighter.insert("mugen".into(), serde_json::to_value(&spec).map_err(|e| e.to_string())?);
    Ok(Bundle { json: Value::Object(fighter), files })
}

pub fn write(bundle: &Bundle, out: &Path, id: &str) -> Result<(), String> {
    let sprites = out.join("sprites");
    std::fs::create_dir_all(&sprites).map_err(|e| format!("{}: {e}", sprites.display()))?;
    for (name, bytes) in &bundle.files {
        std::fs::write(sprites.join(name), bytes).map_err(|e| format!("{name}: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&bundle.json).map_err(|e| e.to_string())?;
    std::fs::write(out.join(format!("{id}.json")), json).map_err(|e| e.to_string())
}
