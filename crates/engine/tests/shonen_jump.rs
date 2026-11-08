//! Integration tests for Shonen Jump M.U.G.E.N characters and stages.
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::stage::load_stage;
use ume_formats::vfs::{add_dir, Vfs};

fn chars_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mugen/chars")
}

fn stages_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mugen/stages")
}

fn find_case_insensitive_dir(root: &Path, name: &str) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().eq_ignore_ascii_case(name) {
                return Some(entry.path());
            }
        }
    }
    None
}

fn load_char(dir_name: &str, def: &str) -> Option<Arc<ume_engine::chardef::CharDef>> {
    let root = chars_root();
    if !root.exists() {
        return None;
    }
    let p = find_case_insensitive_dir(&root, dir_name)?;
    let mut v = Vfs::new();
    add_dir(&mut v, &p, dir_name).ok()?;
    let key = format!("{dir_name}/{def}");
    let d = load_mugen_char(&v, &key, 1, false).ok()?;
    Some(Arc::new(d))
}

#[test]
fn test_shonen_jump_naruto_roster() {
    let root = chars_root();
    if !root.exists() {
        println!("skipping: {} not found (run scripts/fetch-jump-chars.ps1 to populate)", root.display());
        return;
    }

    let naruto_roster = [
        ("naruto_hokage", "naruto nanadaime.def", "Naruto"),
        ("naruto_the_last", "naruto the last.def", "Naruto"),
        ("sasuke", "usasuke.def", "Sasuke"),
        ("Kakashi Kid", "kakashi kid.def", "Kakashi"),
        ("itachi", "uchiha itachi.def", "Uchiha Itachi"),
        ("Naruto Sage", "naruto sage.def", "Sage Naruto by MG"),
        ("AkatsukiMadara299", "akatsukimadara299.def", "AkatsukiMadara"),
        ("rock_lee", "rlee.def", "LEE"),
        ("minato", "yondaime.def", "Yondaime"),
        ("sakura", "haruno sakura.def", "Sakura"),
    ];

    let mut loaded = 0;
    for (dir, def, expected) in naruto_roster {
        if let Some(c) = load_char(dir, def) {
            println!("Loaded Naruto character: {} (anims={}, states={}, sprites={})", c.name, c.anims.len(), c.states.len(), c.sprites.len());
            assert!(!c.anims.is_empty(), "No anims for {}", expected);
            assert!(!c.states.is_empty(), "No states for {}", expected);
            loaded += 1;

            let mut m = Match::new(common_def(), None, flat_stage());
            m.add_player(c.clone(), 0, 0, 1);
            m.add_player(c.clone(), 1, 0, 2);
            m.start_round();
            for _ in 0..60 {
                m.tick(&[0, 0]);
            }
            assert!(m.round_state > 0);
        }
    }
    if loaded > 0 {
        assert!(loaded >= 8, "Expected at least 8 Naruto fighters, got {}", loaded);
    }
}

#[test]
fn test_shonen_jump_jojo_roster() {
    let root = chars_root();
    if !root.exists() {
        println!("skipping: {} not found", root.display());
        return;
    }

    let jojo_roster = [
        ("Jotaro", "jotaro.def", "Jotaro Kujo"),
        ("Dio", "dio.def", "dio"),
    ];

    let mut loaded = 0;
    for (dir, def, expected) in jojo_roster {
        if let Some(c) = load_char(dir, def) {
            println!("Loaded JoJo character: {} (anims={}, states={}, sprites={})", c.name, c.anims.len(), c.states.len(), c.sprites.len());
            assert!(!c.anims.is_empty(), "No anims for {}", expected);
            assert!(!c.states.is_empty(), "No states for {}", expected);
            loaded += 1;

            let mut m = Match::new(common_def(), None, flat_stage());
            m.add_player(c.clone(), 0, 0, 1);
            m.add_player(c.clone(), 1, 0, 2);
            m.start_round();
            for _ in 0..60 {
                m.tick(&[0, 0]);
            }
            assert!(m.round_state > 0);
        }
    }
    if loaded > 0 {
        assert_eq!(loaded, 2, "Both JoJo fighters (Jotaro, Dio) must load successfully");
    }
}

#[test]
fn test_shonen_jump_dbgt_roster() {
    let root = chars_root();
    if !root.exists() {
        println!("skipping: {} not found", root.display());
        return;
    }

    let gt_roster = [
        ("Gogeta-Ssj4", "gogeta-ssj4.def", "Gogeta Ssj4"),
        ("Goku-Ssj4", "goku-ssj4.def", "Goku Ssj4"),
        ("Vegeta-ssj4", "vegeta-ssj4.def", "Vegeta Ssj4"),
    ];

    let mut loaded = 0;
    for (dir, def, expected) in gt_roster {
        if let Some(c) = load_char(dir, def) {
            println!("Loaded DB GT character: {} (anims={}, states={}, sprites={})", c.name, c.anims.len(), c.states.len(), c.sprites.len());
            assert!(!c.anims.is_empty(), "No anims for {}", expected);
            assert!(!c.states.is_empty(), "No states for {}", expected);
            loaded += 1;

            let mut m = Match::new(common_def(), None, flat_stage());
            m.add_player(c.clone(), 0, 0, 1);
            m.add_player(c.clone(), 1, 0, 2);
            m.start_round();
            for _ in 0..60 {
                m.tick(&[0, 0]);
            }
            assert!(m.round_state > 0);
        }
    }
    if loaded > 0 {
        assert_eq!(loaded, 3, "All 3 DB GT fighters must load successfully");
    }
}

#[test]
fn test_shonen_jump_bleach_roster() {
    let root = chars_root();
    if !root.exists() {
        println!("skipping: {} not found (run scripts/fetch-jump-chars.ps1 to populate)", root.display());
        return;
    }

    let bleach_roster = [
        ("Kurosaki Ichigo", "kurosaki ichigo.def", "ichigo"),
        ("Bankai Ichigo", "bankai ichigo.def", "Bankai Ichigo"),
        ("Rukia", "rukia.def", "rukia"),
        ("Renji", "renji.def", "Renji"),
        ("Aizem", "aizem.def", "Aizem"),
        ("Grimmjow", "grimmjow.def", "Grimmjow"),
        ("Hitsugaya1", "hitsugaya1.def", "Hitsugaya"),
        ("KucchikiByakuya", "kucchikibyakuya.def", "Kucchiki Byakuya"),
        ("Yoruichi", "yoruichi.def", "Yoruichi Shihoin"),
        ("Ulquiorra", "ulquiorra.def", "Ulquiorra"),
    ];

    let mut loaded = 0;
    for (dir, def, expected) in bleach_roster {
        if let Some(c) = load_char(dir, def) {
            println!("Loaded Bleach character: {} (anims={}, states={}, sprites={})", c.name, c.anims.len(), c.states.len(), c.sprites.len());
            assert!(!c.anims.is_empty(), "No anims for {}", expected);
            assert!(!c.states.is_empty(), "No states for {}", expected);
            loaded += 1;

            let mut m = Match::new(common_def(), None, flat_stage());
            m.add_player(c.clone(), 0, 0, 1);
            m.add_player(c.clone(), 1, 0, 2);
            m.start_round();
            for _ in 0..60 {
                m.tick(&[0, 0]);
            }
            assert!(m.round_state > 0);
        }
    }
    if loaded > 0 {
        assert!(loaded >= 8, "Expected at least 8 Bleach fighters, got {}", loaded);
    }
}

#[test]
fn test_shonen_jump_stages() {
    let stages_dir = stages_root();
    if !stages_dir.exists() {
        println!("skipping: stages folder not found");
        return;
    }
    let mut v = Vfs::new();
    let _ = add_dir(&mut v, &stages_dir, "");
    if v.is_empty() {
        println!("skipping: no stages found in {}", stages_dir.display());
        return;
    }

    let stage_defs = [
        "sala-do-tempo-dbz1.def",
        "mont-dbz2-dia.def",
        "campo-des-dbz1.def",
        "mthokage.def",
        "tourn.def",
        "forest.def",
    ];

    let mut verified = 0;
    for sdef in stage_defs {
        if v.contains_key(&ume_formats::vfs::norm_path(sdef)) {
            match load_stage(&v, sdef) {
                Ok(st) => {
                    println!("Loaded stage: {} ({})", st.name, sdef);
                    verified += 1;
                }
                Err(e) => panic!("Failed to load stage {}: {}", sdef, e),
            }
        }
    }
    if !v.is_empty() {
        assert!(verified > 0, "At least one Shonen Jump stage should be verified if folder populated");
    }
}
