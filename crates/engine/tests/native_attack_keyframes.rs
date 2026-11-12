use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use ume_engine::native::{native_frame_sprite_key, NativeFrame};

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

fn pose_key(frame: &Value) -> Option<String> {
    if let Some(sprite) = frame.get("sprite").and_then(Value::as_str) {
        return Some(format!("sprite:{sprite}"));
    }
    frame.get("pose").map(|pose| format!("pose:{pose}"))
}

#[test]
fn every_native_attack_has_three_distinct_poses() {
    let mut attacks = 0;
    let mut failures = Vec::new();
    for path in character_files() {
        let character: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let id = character["id"].as_str().unwrap_or("unknown");
        if id.starts_with("npc_") {
            continue;
        }
        let animations = &character["anims"];
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
            let keys: HashSet<_> = animations[name]["frames"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(pose_key)
                .collect();
            if keys.len() < 3 {
                failures.push(format!(
                    "{id}/{} ({name}): {} distinct poses",
                    attack["id"],
                    keys.len()
                ));
            }
        }
    }
    assert_eq!(attacks, 510, "native attack inventory changed");
    assert!(
        failures.is_empty(),
        "attacks with fewer than 3 poses: {failures:#?}"
    );
}

#[test]
fn generated_attack_keyframes_are_crisp_400_pixel_sprites() {
    let mut checked = 0;
    for path in character_files() {
        let character: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let id = character["id"].as_str().unwrap();
        let sprite_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../assets/chars/sprites/{id}"));
        for name in ["clk", "ajab", "akick"] {
            let Some(frames) = character["anims"][name]["frames"].as_array() else {
                continue;
            };
            let Some(frame) = frames.get(2) else {
                continue;
            };
            if frame.get("pose").is_none_or(|pose| !pose.is_object()) {
                continue;
            }
            let frame: NativeFrame = serde_json::from_value(frame.clone()).unwrap();
            let sprite_path = sprite_root.join(format!("{}.png", native_frame_sprite_key(&frame)));
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
            let rgba = decoded.rgba.expect("generated sprite must be RGBA");
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
    assert_eq!(
        checked, 44,
        "expected all 44 generated keyframes to be checked"
    );
}
