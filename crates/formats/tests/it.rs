mod common;
mod air_duplicates;
mod audit;
mod content_key;
mod decoders;
mod font_width;
mod fuzz;
mod fuzz_resilience;
mod lz4_sprite;
mod parsers;
mod real_files;
mod screenpack;
mod sff_v1_shared_palette;
mod stage_3d;
mod stage_backup_dirs;
mod stage_bounds;
mod umsff;
mod wiki_examples;
mod zip_limits;

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
