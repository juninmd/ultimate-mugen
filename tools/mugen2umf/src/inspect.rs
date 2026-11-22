use crate::routes::routes;
use crate::source::Source;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use ume_engine::loader::{assemble_mugen_char, read_mugen_sources};

const HITDEF_KEYS: [&str; 10] = ["attr", "damage", "animtype", "hitflag", "guardflag", "pausetime", "ground.velocity", "air.velocity", "p1stateno", "p2stateno"];

pub fn inspect(src: &Source) -> Result<Value, String> {
    let sources = read_mugen_sources(&src.vfs, &src.def)?;
    let def = assemble_mugen_char(sources.clone(), 1, false);
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for st in def.states.values() {
        for c in &st.controllers {
            *kinds.entry(c.kind.to_ascii_lowercase()).or_default() += 1;
        }
    }
    let routes = routes(&sources, &def.commands);
    let sff = sources.sff.as_ref();
    Ok(json!({
        "def": src.def, "name": def.name, "author": def.author, "mugenversion": sources.mugen_version, "localcoord": sources.localcoord,
        "files": {"stcommon": sources.stcommon.as_ref().map(|t| &t.name), "cns": sources.cns.name, "st": sources.st.iter().map(|t| &t.name).collect::<Vec<_>>(), "cmd": sources.cmd.name},
        "sff": {"version": sff.map(|s| s.version), "sprites": sff.map_or(0, |s| s.sprites.len()), "palettes": sff.map_or(0, |s| s.palettes.len())},
        "sounds": sources.sounds.len(), "anims": sources.anims.len(), "commands": def.commands.len(), "states": def.states.len(),
        "palettes": sources.pal_files.iter().filter(|p| p.is_some()).count(),
        "controllers": kinds, "routes": routes,
        "engine": def.diagnostics().report(&src.def, false),
    }))
}

pub fn hitdefs(src: &Source) -> Result<Value, String> {
    let def = assemble_mugen_char(read_mugen_sources(&src.vfs, &src.def)?, 1, false);
    let mut ids: Vec<&i32> = def.states.keys().collect();
    ids.sort();
    let rows: Vec<Value> = ids
        .into_iter()
        .flat_map(|id| def.states[id].controllers.iter().filter(|c| c.kind.eq_ignore_ascii_case("hitdef")).map(move |c| (id, c)))
        .map(|(id, c)| {
            let params: BTreeMap<&str, &str> = HITDEF_KEYS.iter().filter_map(|k| Some((*k, c.params.iter().find(|(n, _)| n == k)?.1.raw.as_str()))).collect();
            json!({"state": id, "label": c.label, "params": params})
        })
        .collect();
    Ok(Value::Array(rows))
}
