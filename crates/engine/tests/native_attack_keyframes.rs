use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use ume_engine::native::{audit_native_sprite_frames, NativeChar};

fn character_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/chars");
    let mut files: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .filter(|path| path.file_stem().is_some_and(|stem| stem != "index"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_native_attack_has_three_distinct_rendered_images() {
    let mut attacks = 0;
    let mut failures = Vec::new();
    for path in character_files() {
        let character: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let id = character["id"].as_str().unwrap_or("unknown").to_owned();
        if id.starts_with("npc_") {
            continue;
        }
        let native: NativeChar = serde_json::from_value(character.clone()).unwrap();
        let audit = audit_native_sprite_frames(&native);
        let animations: HashMap<_, _> = audit["animations"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|animation| {
                Some((
                    animation["animation"].as_str()?,
                    animation["uniqueBodyImages"].as_u64()?,
                ))
            })
            .collect();
        let forms = character["abilities"]["forms"]
            .as_array()
            .into_iter()
            .flatten();
        for attack in character["moves"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(forms.flat_map(|form| form["moves"].as_array().into_iter().flatten()))
            .filter(|attack| {
                matches!(
                    attack["kind"].as_str(),
                    Some("normal" | "special" | "super")
                )
            })
        {
            attacks += 1;
            let name = attack["anim"].as_str().unwrap_or("unknown");
            let image_count = animations.get(name).copied().unwrap_or_default();
            if image_count < 3 {
                failures.push(format!(
                    "{id}/{} ({name}): {image_count} distinct rendered images",
                    attack["id"],
                ));
            }
        }
    }
    assert_eq!(attacks, 526, "native attack inventory changed");
    assert!(
        failures.is_empty(),
        "attacks with fewer than 3 rendered images: {failures:#?}"
    );
}

#[test]
fn every_native_transformation_uses_three_distinct_powerup_images() {
    let mut characters = 0;
    let mut failures = Vec::new();
    for path in character_files() {
        let character: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let id = character["id"].as_str().unwrap_or("unknown").to_owned();
        let native: NativeChar = serde_json::from_value(character).unwrap();
        let audit = audit_native_sprite_frames(&native);
        let power_up = audit["animations"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|animation| animation["animation"] == "powerUp");
        let frames = power_up.and_then(|animation| animation["frames"].as_array());
        let image_count = power_up
            .and_then(|animation| animation["uniqueBodyImages"].as_u64())
            .unwrap_or_default();
        let sprite_files: Vec<_> = frames
            .into_iter()
            .flatten()
            .filter_map(|frame| frame["file"].as_str())
            .collect();
        if image_count < 3 || sprite_files.len() != 3 {
            failures.push(format!("{id}: {image_count} distinct powerUp images from {} resolved files", sprite_files.len()));
        }
        if id == "garan"
            && !["15_power0.png", "16_power1.png", "17_power2.png"]
                .iter()
                .all(|name| sprite_files.iter().any(|file| file.ends_with(name)))
        {
            failures.push(format!("{id}: transformation does not resolve all three authored powerUp sprites"));
        }
        characters += 1;
    }
    assert_eq!(characters, 21, "native character inventory changed");
    assert!(failures.is_empty(), "transformation keyframe failures: {failures:#?}");
}

#[test]
fn every_native_combat_sprite_is_crisp_pixel_art() {
    let mut checked = 0;
    let sprite_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/chars/sprites");
    for directory in fs::read_dir(sprite_root).unwrap() {
        let directory = directory.unwrap().path();
        if !directory.is_dir() {
            continue;
        }
        for entry in fs::read_dir(directory).unwrap() {
            let sprite_path = entry.unwrap().path();
            if sprite_path
                .extension()
                .is_none_or(|extension| extension != "png")
                || sprite_path
                    .file_stem()
                    .is_some_and(|stem| stem == "portrait")
            {
                continue;
            }
            let decoded = ume_formats::png::decode_png(
                &fs::read(&sprite_path)
                    .unwrap_or_else(|error| panic!("{}: {error}", sprite_path.display())),
            )
            .unwrap();
            assert_eq!(
                (decoded.width, decoded.height),
                (400, 400),
                "{}",
                sprite_path.display()
            );
            let rgba = decoded
                .rgba
                .as_ref()
                .cloned()
                .unwrap_or_else(|| ume_formats::png::png_indexed_to_rgba(&decoded));
            let mut colors = HashSet::new();
            for pixel in rgba.chunks_exact(4) {
                assert!(
                    pixel[3] == 0 || pixel[3] == 255,
                    "partial alpha in {}",
                    sprite_path.display()
                );
                if pixel[3] != 0 {
                    colors.insert([pixel[0], pixel[1], pixel[2]]);
                }
            }
            assert!(
                colors.len() <= 48,
                "{} colors in {}",
                colors.len(),
                sprite_path.display()
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 2162, "native combat sprite inventory changed");
}
