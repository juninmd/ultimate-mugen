//! Built-in stages defined only by data: every folder `assets/stages/data/<id>/` is embedded at
//! build time (see `build.rs`) and loaded as a MUGEN stage from `<id>/stage.def`.

use crate::registry::stage_bgm;
use crate::stages::{from_mugen, builtin, Stage, StageEntry};
use std::sync::Arc;
use ume_formats::ini::{find_section, parse_ini, unquote};
use ume_formats::stage::load_stage;
use ume_formats::vfs::{norm_path, Vfs};

include!(concat!(env!("OUT_DIR"), "/stage_data_gen.rs"));

const THUMB: [[u8; 3]; 3] = [[24, 40, 64], [70, 110, 150], [236, 196, 120]];

fn def_path(id: &str) -> String {
    format!("{id}/stage.def")
}

fn vfs_of(files: &[(&str, &[u8])]) -> Vfs {
    let mut vfs = Vfs::new();
    for (path, bytes) in files {
        vfs.insert(norm_path(path), bytes.to_vec());
    }
    vfs
}

fn build(vfs: &Vfs, def: &str) -> Stage {
    let mut stage = match load_stage(vfs, def) {
        Ok(mugen) => from_mugen(mugen),
        Err(e) => {
            eprintln!("bundled stage {def}: {e}");
            (builtin().swap_remove(0).build)()
        }
    };
    stage.bgm = stage_bgm(vfs, def).map(Arc::new);
    stage
}

fn entries() -> Vec<StageEntry> {
    STAGE_DATA
        .iter()
        .map(|(id, files)| {
            let vfs = Arc::new(vfs_of(files));
            let def = def_path(id);
            let name = vfs
                .get(&def)
                .map(|bytes| {
                    let secs = parse_ini(&String::from_utf8_lossy(bytes.as_slice()));
                    let info = find_section(&secs, "info");
                    let shown = info.and_then(|s| s.get("displayname").or_else(|| s.get("name")));
                    unquote(shown.unwrap_or(id))
                })
                .unwrap_or_else(|| id.to_string());
            StageEntry {
                id: id.to_string(),
                name,
                thumb: THUMB,
                build: std::sync::Arc::new(move || build(&vfs, &def)),
            }
        })
        .collect()
}

/// Every built-in stage: the procedural/illustrated ones first, then the data-only ones.
pub fn builtin_all() -> Vec<StageEntry> {
    let mut all = builtin();
    all.extend(entries());
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stages::StageArt;

    #[test]
    fn bundled_example_is_listed_and_builds_with_its_sky() {
        let entry = builtin_all().into_iter().find(|e| e.id == "example").expect("example stage is listed");
        assert_eq!(entry.name, "Cais da Aurora (exemplo)");
        match (entry.build)().art {
            StageArt::Mugen(bg) => {
                assert!(bg.stage.sky.is_some());
                assert_eq!(bg.stage.layers.len(), 4);
            }
            _ => panic!("data stages are MUGEN-style layers"),
        }
    }
}
