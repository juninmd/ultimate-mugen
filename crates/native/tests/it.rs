mod combat_stats;
mod fighter_3d_jumps;
mod fighter_3d_kits;
mod idle_walk_pacing;
mod move_fx_and_kits;
mod teleport_and_genkidama;

#[test]
fn every_test_file_is_registered_in_it_rs() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let registered = include_str!("it.rs");
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            assert!(
                stem == "it" || registered.contains(&format!("mod {stem};")),
                "tests/{stem}.rs is not compiled: add `mod {stem};` to tests/it.rs"
            );
        }
    }
}
