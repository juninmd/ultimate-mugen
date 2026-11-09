use std::collections::HashMap;
use std::path::{Path, PathBuf};
use ume_charforge::movekit::{build_from_spec, build_moves, expand_phases, hit_frames, SpriteLibrary};
use ume_charforge::spritekit::{
    anatomical_boxes, attack_boxes, interpolate_boxes, label_components, load_act, save_act, slice_sheet, variant_colors, write_indexed_png,
    GridOptions, Mask, SliceMode, SliceOptions,
};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("charforge-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_sheet(dir: &Path) -> PathBuf {
    let (width, height) = (160usize, 70usize);
    let mut indices = vec![0u8; width * height];
    for (x0, x1, color) in [(10, 30, 1u8), (80, 120, 2u8)] {
        for y in 10..66 {
            for x in x0..x1 {
                indices[y * width + x] = color;
            }
        }
    }
    let path = dir.join("sheet.png");
    write_indexed_png(&path, width, height, &indices, &[[255, 0, 255], [200, 40, 30], [200, 90, 30]]).unwrap();
    path
}

#[test]
fn labels_separate_blobs() {
    let mut mask = Mask::new(10, 10);
    for (x, y) in [(1, 1), (2, 1), (1, 2), (7, 7), (9, 9)] {
        mask.set(x, y, true);
    }
    let labels = label_components(&mask);
    let distinct: std::collections::HashSet<_> = labels.iter().filter(|l| **l != 0).collect();
    assert_eq!(distinct.len(), 3);
}

#[test]
fn slice_auto_exports_indexed_pngs_with_feet_axis() {
    let dir = temp_dir("auto");
    let options = SliceOptions { group: 9000, ..SliceOptions::default() };
    let manifest = slice_sheet(&write_sheet(&dir), &dir.join("out"), &options).unwrap();
    let sprites = manifest["sprites"].as_array().unwrap();
    assert_eq!(sprites.len(), 2);
    assert_eq!(sprites[0]["item"], 0);
    assert_eq!(manifest["exact_palette"], true);
    assert_eq!(sprites[0]["axis"], serde_json::json!([10, 55]));
    assert!(dir.join("out").join("9000_1.png").exists());
}

#[test]
fn slice_grid_cuts_one_sprite_per_cell() {
    let dir = temp_dir("grid");
    let grid = GridOptions { columns: Some(2), rows: Some(1), ..GridOptions::default() };
    let options = SliceOptions { mode: SliceMode::Grid(grid), ..SliceOptions::default() };
    let manifest = slice_sheet(&write_sheet(&dir), &dir.join("out"), &options).unwrap();
    assert_eq!(manifest["sprites"].as_array().unwrap().len(), 2);
}

#[test]
fn palette_variants_keep_index_zero_and_round_trip_act() {
    let mut colors = vec![[0u8, 0, 0]; 256];
    colors[0] = [255, 0, 255];
    colors[1] = [200, 40, 30];
    let shifted = variant_colors(&colors, 120.0, 1.0, 1.0, &[]);
    assert_eq!(shifted[0], colors[0]);
    assert_ne!(shifted[1], colors[1]);
    let dir = temp_dir("act");
    save_act(&colors, &dir.join("base.act")).unwrap();
    let loaded = load_act(&dir.join("base.act")).unwrap();
    assert_eq!(loaded[1], colors[1]);
}

#[test]
fn boxes_are_relative_to_the_axis() {
    let mut mask = Mask::new(60, 80);
    for y in 10..78 {
        for x in 20..40 {
            mask.set(x, y, true);
        }
    }
    let boxes = anatomical_boxes(&mask, (30, 77), 3, 2);
    assert_eq!(boxes.len(), 3);
    assert!(boxes.iter().all(|b| b.0 < b.2 && b.1 < b.3));
    assert!(attack_boxes(&mask, (30, 77), 2).is_empty());
    for y in 40..44 {
        for x in 40..58 {
            mask.set(x, y, true);
        }
    }
    assert_eq!(attack_boxes(&mask, (30, 77), 2).len(), 1);
    assert_eq!(interpolate_boxes(&boxes, &boxes, 2).unwrap().len(), 2);
}

fn build_example(name: &str) -> (ume_charforge::movekit::Built, PathBuf) {
    let dir = temp_dir(name);
    let options = SliceOptions { group: 9000, ..SliceOptions::default() };
    slice_sheet(&write_sheet(&dir), &dir.join("sprites"), &options).unwrap();
    let spec = dir.join("moves.toml");
    std::fs::write(&spec, include_str!("../examples/moves.toml")).unwrap();
    (build_from_spec(&spec, &dir.join("out")).unwrap(), dir)
}

#[test]
fn generated_text_parses_with_the_engine() {
    let (built, dir) = build_example("moves");
    for file in ["moves.air", "moves.cns", "moves.cmd", "statedef-1.cns"] {
        assert!(!std::fs::read_to_string(dir.join("out").join(file)).unwrap().trim().is_empty());
    }
    let anims = ume_formats::air::parse_air(&built.air);
    assert_eq!(anims.len(), 4);
    assert!(anims[&200].frames.iter().any(|f| !f.clsn1.is_empty()));
    let mut states = HashMap::new();
    ume_engine::cns::parse_states(&built.cns, &mut states, false);
    let mut numbers: Vec<i32> = states.keys().copied().collect();
    numbers.sort();
    assert_eq!(numbers, vec![200, 1000, 3000, 3001]);
    let mut entry_states = HashMap::new();
    ume_engine::cns::parse_states(&built.entry, &mut entry_states, false);
    assert!(entry_states.contains_key(&-1));
    let (commands, _) = ume_engine::cns::parse_cmd(&built.cmd);
    let names: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, vec!["spec_200", "spec_1000", "spec_3000"]);
}

#[test]
fn super_has_pause_cost_chain_and_priority_in_entry_block() {
    let (built, _) = build_example("super");
    assert!(built.cns.contains("type = SuperPause"));
    assert!(built.cns.contains("poweradd = -3000"));
    assert!(built.cns.contains("value = 3001"));
    assert!(built.entry.find("spec_3000").unwrap() < built.entry.find("spec_200").unwrap());
    assert!(built.entry.contains("power >= 3000"));
    assert_eq!(hit_frames("Clsn1: 1\n  Clsn1[0] = 0,0,1,1\n0,0,0,0,3"), vec![1]);
}

#[test]
fn invalid_specs_are_rejected() {
    let mut library = SpriteLibrary::default();
    let table = |text: &str| -> toml::Table { text.parse().unwrap() };
    assert!(build_moves(&[table("state = 1\nkind = \"bogus\"")], &toml::Table::new(), &mut library).is_err());
    assert!(build_moves(&[table("state = 1"), table("state = 1")], &toml::Table::new(), &mut library).is_err());
    assert!(expand_phases(&table("state = 5\nphases = [{ like = \"missing\" }]")).is_err());
}
