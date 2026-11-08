//! Integration tests for Dragon Ball Z M.U.G.E.N characters in mugen/ folder.
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_dir, Vfs};

fn chars_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mugen/chars")
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
fn test_dbz_characters_load_and_fight() {
    let root = chars_root();
    if !root.exists() {
        println!("skipping: {} not found (run scripts/fetch-jump-chars.ps1 to populate)", root.display());
        return;
    }

    let dbz_roster = [
        ("ssj_gokuz2", "ssj_gokuz2.def", "SSJ Goku Z2"),
        ("buuz2", "buuz2.def", "Buu Z2"),
        ("c18z2", "c18z2.def", "C18 Z2"),
        ("gohanz2", "gohanz2.def", "Gohan Z2"),
        ("gotenksz2", "gotenksz2.def", "Gotenks Z2"),
        ("tenshinhanz2", "tenshinhanz2.def", "TenShinHan Z2"),
        ("cell", "cell.def", "Cell"),
        ("vegeta", "vegeta gt.def", "Vegeta"),
        ("bardock", "bardock.def", "Bardock"),
        ("frieza", "frieza.def", "Frieza"),
        ("piccolo", "piccolo.def", "Piccolo"),
        ("kid_goku", "kid_goku.def", "Kid Goku"),
        ("bills", "bills.def", "Bills"),
        ("raditz", "raditz.def", "Raditz"),
    ];

    let mut loaded = 0;
    for (dir, def, expected_name) in dbz_roster {
        if let Some(c) = load_char(dir, def) {
            println!("Loaded DBZ character: {} (anims={}, states={}, sprites={})", c.name, c.anims.len(), c.states.len(), c.sprites.len());
            assert!(!c.anims.is_empty(), "No anims for {}", expected_name);
            assert!(!c.states.is_empty(), "No states for {}", expected_name);
            loaded += 1;

            // Smoke round test: run 60 ticks
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
        assert!(loaded >= 10, "Expected at least 10 DBZ characters loaded, got {}", loaded);
    }
}
