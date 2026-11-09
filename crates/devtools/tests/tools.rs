use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;
use ume_devtools::audit_moves::audit;
use ume_devtools::pack::{encode_sprite, lanczos_resize, pack_character, pack_tiers, rescale_meta, Image};
use ume_formats::png::{decode_png, png_indexed_to_rgba};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("devtools-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn gradient(width: usize, height: usize) -> Image {
    let mut rgba = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let inside = x > 4 && y > 4 && x < width - 4;
            rgba.extend_from_slice(&[(x * 255 / width) as u8, (y * 255 / height) as u8, 90, if inside { 255 } else { 0 }]);
        }
    }
    Image { width, height, rgba }
}

fn png_bytes(image: &Image) -> Vec<u8> {
    let palette: Vec<[u8; 4]> = (0..8u8).map(|i| if i == 0 { [0, 0, 0, 0] } else { [i * 30, 255 - i * 30, 90, 255] }).collect();
    let indices: Vec<u8> = image.rgba.chunks_exact(4).enumerate().map(|(i, p)| if p[3] == 0 { 0 } else { 1 + ((i % image.width) / 130 % 7) as u8 }).collect();
    ume_charforge::spritekit::encode_palette_png(image.width, image.height, &indices, &palette, 6)
}

#[test]
fn lanczos_keeps_flat_color_and_alpha() {
    let flat = Image { width: 40, height: 20, rgba: [10u8, 200, 30, 255].repeat(800) };
    let small = lanczos_resize(&flat, 10, 5);
    assert_eq!((small.width, small.height), (10, 5));
    assert!(small.rgba.chunks_exact(4).all(|p| p == [10, 200, 30, 255]));
    let clear = Image { width: 8, height: 8, rgba: [0u8; 256].to_vec() };
    assert!(lanczos_resize(&clear, 4, 4).rgba.iter().all(|&b| b == 0));
}

#[test]
fn encoded_sprites_decode_with_at_most_256_colors_and_keep_transparency() {
    let source = gradient(900, 300);
    let (encoded, scale) = encode_sprite("pose.png", &png_bytes(&source)).unwrap();
    assert!((scale - 768.0 / 900.0).abs() < 1e-9);
    let decoded = decode_png(&encoded).unwrap();
    assert_eq!((decoded.width, decoded.height), (768, 256));
    let palette = decoded.palette.as_ref().unwrap();
    let used: std::collections::HashSet<u8> = decoded.indexed.as_ref().unwrap().iter().copied().collect();
    assert!(used.len() <= 256 && palette.len() == 1024);
    let rgba = png_indexed_to_rgba(&decoded);
    assert_eq!(rgba[3], 0);
    assert_eq!(rgba[(100 * 768 + 400) * 4 + 3], 255);
}

#[test]
fn portraits_are_capped_at_256() {
    let (encoded, scale) = encode_sprite("portrait.png", &png_bytes(&gradient(512, 512))).unwrap();
    assert_eq!(scale, 0.5);
    assert_eq!(decode_png(&encoded).unwrap().width, 256);
}

#[test]
fn meta_anchors_follow_the_image_scale() {
    let meta = json!({ "refHeight": 200, "frames": { "idle": { "axisX": 100, "axisY": 300, "awakeningHair": [10, 11, 12, 13] } } });
    let scales = BTreeMap::from([("idle.png".to_string(), 0.5), ("walk.png".to_string(), 0.5), ("same.png".to_string(), 1.0)]);
    let out: serde_json::Value = serde_json::from_slice(&rescale_meta(&serde_json::to_vec(&meta).unwrap(), &scales).unwrap()).unwrap();
    assert_eq!(out["frames"]["idle"]["axisX"], 50.0);
    assert_eq!(out["frames"]["idle"]["awakeningHair"], json!([5, 5, 6, 7]));
    assert_eq!(out["frames"]["walk"]["refHeight"], 100.0);
    assert!(out["frames"].get("same").is_none());
}

#[test]
fn menu_tier_keeps_only_the_wanted_sprites_and_the_metadata() {
    let dir = temp_dir("tiers");
    let folder = dir.join("hero");
    std::fs::create_dir_all(&folder).unwrap();
    for name in ["idle.png", "kick.png", "portrait.png"] {
        std::fs::write(folder.join(name), png_bytes(&gradient(64, 64))).unwrap();
    }
    std::fs::write(folder.join("meta.json"), "{}").unwrap();
    let wanted = vec!["idle.png".to_string(), "portrait.png".to_string()];
    let menu = dir.join("hero.menu.pack");
    pack_tiers("hero", &folder, &dir.join("hero.pack"), Some((menu.as_path(), wanted.as_slice()))).unwrap();
    let count = |path: &std::path::Path| {
        let bytes = std::fs::read(path).unwrap();
        (u32::from_le_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]), bytes)
    };
    let (full, _) = count(&dir.join("hero.pack"));
    let (slim, bytes) = count(&menu);
    assert_eq!((full, slim), (4, 3));
    assert!(!bytes.windows(8).any(|window| window == b"kick.png"));
}

#[test]
fn pack_layout_matches_the_engine_format() {
    let dir = temp_dir("pack");
    let folder = dir.join("hero");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("idle.png"), png_bytes(&gradient(64, 64))).unwrap();
    std::fs::write(folder.join("meta.json"), "{}").unwrap();
    let size = pack_character("hero", &folder, &dir.join("hero.pack")).unwrap();
    let bytes = std::fs::read(dir.join("hero.pack")).unwrap();
    assert_eq!(bytes.len(), size);
    assert_eq!(&bytes[..4], b"UMEP");
    assert_eq!(bytes[4], 2);
    assert_eq!(u16::from_le_bytes([bytes[5], bytes[6]]), 4);
    assert_eq!(u32::from_le_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]), 2);
    assert_eq!(pack_character("empty", &dir, &dir.join("none.pack")).unwrap(), 0);
}

#[test]
fn move_audit_flags_missing_anims_and_duplicate_inputs() {
    let character = json!({
        "id": "t", "abilities": {},
        "anims": { "a": { "frames": [{ "ticks": 3, "hit": { "radius": 10 } }, { "ticks": -1 }] } },
        "moves": [
            { "id": "m1", "kind": "normal", "input": "a", "anim": "a", "hits": [{ "at": 1, "damage": 10 }] },
            { "id": "m2", "kind": "normal", "input": "a", "anim": "gone", "cancels": ["nope"] },
        ],
    });
    let problems: Vec<String> = audit(&character)["problems"].as_array().unwrap().iter().map(|p| p.as_str().unwrap().to_string()).collect();
    assert!(problems.contains(&"m2: anim 'gone' missing".to_string()));
    assert!(problems.contains(&"m2: input 'a' duplicates m1".to_string()));
    assert!(problems.contains(&"m2: cancels unknown move 'nope'".to_string()));
    assert!(problems.contains(&"m1: ground move ends on an infinite frame (never returns control)".to_string()));
    assert!(problems.contains(&"no throw".to_string()));
}

#[test]
fn roster_audit_is_clean() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/chars");
    let results = ume_devtools::audit_moves::audit_directory(&root).unwrap();
    assert!(results.len() >= 10);
    for result in results {
        assert!(result["problems"].as_array().unwrap().is_empty(), "{}: {}", result["id"], result["problems"]);
    }
}

#[test]
fn umf_edit_preserves_style() {
    let dir = std::env::temp_dir().join(format!("ume-umf-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (name, text) in [("a.json", "{\r\n  \"b\": 1,\r\n  \"a\": [1, 2]\r\n}\r\n"), ("c.json", "{\"z\":1,\"y\":2}")] {
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        let mut doc = ume_devtools::umf_edit::load(&path).unwrap();
        doc.data.set("n", ume_devtools::json_order::Json::int(3));
        doc.save(&path).unwrap();
        let out = std::fs::read_to_string(&path).unwrap();
        assert!(out.contains("\"n\""));
        assert_eq!(out.ends_with("\r\n"), text.ends_with("\r\n"));
        assert_eq!(out.contains('\n'), text.contains('\n'));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn effective_ticks_matches_runtime_rounding() {
    use ume_devtools::{json_order::Json, motion::effective_ticks};
    assert_eq!(effective_ticks(Some(&Json::int(6)), Some(&Json::float(1.5))).unwrap(), 4);
    assert_eq!(effective_ticks(Some(&Json::int(6)), None).unwrap(), 6);
    assert!(effective_ticks(Some(&Json::int(0)), None).is_err());
}
