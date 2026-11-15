//! Every ```ini block in the stage pages that holds a `.def` must load with `load_stage`,
//! and the examples that document the new keys must really take effect.

use std::path::Path;
use ume_formats::stage::load_stage;
use ume_formats::vfs::{norm_path, Vfs};

const PAGES: [&str; 2] = ["docs/wiki/Stages.md", "docs/wiki/pt-BR/Cenarios.md"];

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ini_blocks(markdown: &str) -> Vec<String> {
    let mut out = vec![];
    let mut cur: Option<String> = None;
    for line in markdown.lines() {
        let t = line.trim();
        if let Some(buf) = cur.as_mut() {
            if t == "```" {
                out.push(cur.take().unwrap_or_default());
            } else {
                buf.push_str(line);
                buf.push('\n');
            }
        } else if t == "```ini" {
            cur = Some(String::new());
        }
    }
    out
}

fn def_blocks() -> Vec<String> {
    PAGES
        .iter()
        .flat_map(|p| ini_blocks(&std::fs::read_to_string(root().join(p)).unwrap_or_else(|e| panic!("{p}: {e}"))))
        .filter(|b| ["[BGdef]", "[Info]", "[Camera]", "[Music]", "[Sky]", "[BG "].iter().any(|k| b.contains(k)))
        .collect()
}

fn load(def: &str) -> ume_formats::stage::MugenStage {
    let mut vfs = Vfs::new();
    vfs.insert(norm_path("stage/stage.def"), def.as_bytes().to_vec());
    load_stage(&vfs, "stage/stage.def").unwrap_or_else(|e| panic!("example failed to load: {e}\n{def}"))
}

#[test]
fn every_wiki_stage_example_loads() {
    let blocks = def_blocks();
    assert!(blocks.len() >= 4, "expected stage examples in the wiki, found {}", blocks.len());
    for b in &blocks {
        load(b);
    }
}

#[test]
fn sky_and_parallax_examples_take_effect() {
    let blocks = def_blocks();
    let with = |needle: &str| blocks.iter().find(|b| b.contains(needle)).cloned();
    let sky = load(&with("[Sky]").expect("[Sky] example in the wiki"));
    assert!(sky.sky.is_some(), "[Sky] example must set the sky");
    let para = load(&with("type = parallax").expect("parallax example in the wiki"));
    let layer = para.layers.iter().find(|l| l.kind == "parallax").expect("parallax layer");
    assert_eq!(layer.tile, [1, 0]);
}

#[test]
fn music_example_sets_gain_and_file() {
    let blocks = def_blocks();
    let music = blocks.iter().find(|b| b.contains("[Music]")).expect("[Music] example in the wiki");
    let m = load(music);
    let music = m.music.expect("[Music] section");
    assert!(!music.file.is_empty());
    assert!(music.volume > 0.0);
}

#[test]
fn bundled_example_stage_loads_with_its_sprites() {
    let dir = root().join("assets/stages/data/example");
    let mut vfs = Vfs::new();
    for f in ["stage.def", "example.sff"] {
        let bytes = std::fs::read(dir.join(f)).unwrap();
        vfs.insert(norm_path(&format!("example/{f}")), bytes);
    }
    let st = load_stage(&vfs, "example/stage.def").expect("bundled stage");
    assert_eq!(st.layers.len(), 4);
    assert_eq!(st.sprites.len(), 4);
    assert!(st.sky.is_some());
}
