use super::rig::{art_res, RigRender};
use super::types::{NativeChar, NativeFrame};
use crate::chardef::ImageData;
use serde_json::Value;
use std::path::{Path, PathBuf};

const SPRITE_ROOT: &str = "assets/chars/sprites";
const BASE_STEM: &str = "01_idle";
const STAND_HURT: [f32; 4] = [-18.0, -96.0, 18.0, 0.0];
const CROUCH_HURT: [f32; 4] = [-16.0, -60.0, 16.0, 0.0];
const LYING_HURT: [f32; 4] = [-46.0, -26.0, 46.0, 0.0];
const STRIKE_BOX: [f32; 4] = [2.0, -84.0, 38.0, -32.0];
const STAND_UNITS: f32 = 100.0;

pub(super) fn try_load(nc: &NativeChar, fr: &NativeFrame) -> Option<RigRender> {
    load_pose(&sprite_roots(&nc.id), &pose_name(fr), fr.hit.is_some())
}

fn pose_name(fr: &NativeFrame) -> String {
    match (&fr.sprite, &fr.pose) {
        (Some(s), _) => s.clone(),
        (None, Some(Value::String(p))) => p.clone(),
        _ => "idle".to_string(),
    }
}

fn sprite_roots(id: &str) -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from(SPRITE_ROOT).join(id),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(SPRITE_ROOT).join(id),
    ];
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
        roots.push(dir.join(SPRITE_ROOT).join(id));
    }
    roots
}

fn load_pose(roots: &[PathBuf], pose: &str, striking: bool) -> Option<RigRender> {
    let lower = pose.to_ascii_lowercase();
    let stem = pose_stem(&lower);
    [pose, stem, BASE_STEM, "idle"]
        .iter()
        .flat_map(|name| roots.iter().map(move |root| (root, name)))
        .find_map(|(root, name)| read_render(root, name, &lower, stem, striking))
}

fn read_render(root: &Path, name: &str, lower: &str, stem: &str, striking: bool) -> Option<RigRender> {
    let bytes = std::fs::read(root.join(format!("{name}.png"))).ok()?;
    let dec = ume_formats::png::decode_png(&bytes).ok()?;
    let rgba = dec.rgba?;
    let res = art_res().max(0.1);
    let units = reference_height(root).unwrap_or(dec.height as f32).max(1.0);
    Some(RigRender {
        image: Some(ImageData::new(dec.width, dec.height, rgba)),
        width: dec.width as f32 * res,
        height: dec.height as f32 * res,
        axis_x: dec.width as f32 / 2.0 * res,
        axis_y: dec.height as f32 * res,
        scale: STAND_UNITS / units / res,
        hurt: vec![hurtbox(lower, stem)],
        hit: if striking { vec![STRIKE_BOX] } else { vec![] },
    })
}

fn reference_height(root: &Path) -> Option<f32> {
    let meta: Value = serde_json::from_slice(&std::fs::read(root.join("meta.json")).ok()?).ok()?;
    meta.get("refHeight")?.as_f64().filter(|h| *h > 0.0).map(|h| h as f32)
}

fn hurtbox(lower: &str, stem: &str) -> [f32; 4] {
    if stem == "14_lie" {
        LYING_HURT
    } else if lower.contains("crouch") {
        CROUCH_HURT
    } else {
        STAND_HURT
    }
}

fn pose_stem(p: &str) -> &'static str {
    match p {
        "idle1" | "idle3" => "07_idle1",
        "idle2" => "08_idle2",
        "hith1" => "09_hit1",
        "hith2" => "10_hit2",
        "hith3" => "11_hit3",
        "hitair1" | "fall1" | "trip" => "12_fall1",
        "hitair2" | "fall2" | "bounce" => "13_fall2",
        "lie" | "dead" | "lose" => "14_lie",
        "powerup0" => "15_power0",
        "powerup1" => "16_power1",
        "powerup2" => "17_power2",
        _ if p.starts_with("df") => "14_lie",
        _ => basic_stem(p),
    }
}

fn basic_stem(p: &str) -> &'static str {
    let has = |keys: &[&str]| keys.iter().any(|k| p.contains(k));
    if has(&["idle", "walk", "run", "stand", "turn", "crouch", "land", "hop", "guard", "getup", "recover"]) {
        BASE_STEM
    } else if has(&["jab", "punch"]) || p.starts_with("str") {
        "02_attack1"
    } else if has(&["kick", "lk", "hk", "sweep"]) {
        "03_attack2"
    } else if has(&["special", "super", "arc", "frost", "cut", "rise"]) || p.starts_with("sp") {
        "04_special"
    } else if has(&["proj", "blast", "fireball", "wave", "gale"]) {
        "05_projectile"
    } else if has(&["win", "vic", "taunt", "intro"]) {
        "06_victory"
    } else {
        BASE_STEM
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    fn write_png(dir: &Path, name: &str, w: u32, h: u32) {
        let mut pm = Pixmap::new(w, h).unwrap();
        pm.fill(tiny_skia::Color::from_rgba8(200, 20, 20, 255));
        pm.save_png(dir.join(format!("{name}.png"))).unwrap();
    }

    fn fixture(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ume-pixel-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn every_standard_state_pose_has_its_own_stem() {
        let expected = [
            ("idle0", "01_idle"), ("idle1", "07_idle1"), ("idle2", "08_idle2"), ("idle3", "07_idle1"),
            ("hith1", "09_hit1"), ("hith2", "10_hit2"), ("hith3", "11_hit3"),
            ("hitair1", "12_fall1"), ("fall1", "12_fall1"), ("trip", "12_fall1"),
            ("hitair2", "13_fall2"), ("fall2", "13_fall2"), ("bounce", "13_fall2"),
            ("lie", "14_lie"), ("dead", "14_lie"), ("lose", "14_lie"), ("dflie", "14_lie"), ("dfprop2", "14_lie"), ("dfreach1", "14_lie"),
            ("powerup0", "15_power0"), ("powerup1", "16_power1"), ("powerup2", "17_power2"),
        ];
        for (pose, stem) in expected {
            assert_eq!(pose_stem(pose), stem, "{pose}");
        }
    }

    #[test]
    fn attack_poses_keep_their_base_stems() {
        assert_eq!(pose_stem("jab1"), "02_attack1");
        assert_eq!(pose_stem("lkick0"), "03_attack2");
        assert_eq!(pose_stem("rise2"), "04_special");
        assert_eq!(pose_stem("blast1"), "05_projectile");
        assert_eq!(pose_stem("win0"), "06_victory");
        assert_eq!(pose_stem("white"), "01_idle", "a pose merely containing 'hit' is not a damage pose");
    }

    #[test]
    fn exact_file_wins_then_stem_then_idle() {
        let dir = fixture("order");
        write_png(&dir, "01_idle", 10, 20);
        write_png(&dir, "09_hit1", 30, 20);
        write_png(&dir, "jab0", 50, 20);
        let roots = [dir.clone()];
        let art_width = |pose: &str| load_pose(&roots, pose, false).unwrap().image.unwrap().w;
        assert_eq!(art_width("jab0"), 50);
        assert_eq!(art_width("hitH1"), 30);
        assert_eq!(art_width("hitH3"), 10, "no hit3 art: falls back to the base pose");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reference_height_keeps_every_pose_at_one_scale() {
        let dir = fixture("scale");
        write_png(&dir, "01_idle", 40, 200);
        write_png(&dir, "14_lie", 200, 200);
        std::fs::write(dir.join("meta.json"), r#"{"refHeight": 160}"#).unwrap();
        let roots = [dir.clone()];
        let stand = load_pose(&roots, "idle0", false).unwrap();
        let lying = load_pose(&roots, "lie", false).unwrap();
        assert_eq!(stand.scale, lying.scale);
        assert!((stand.scale * 160.0 - STAND_UNITS / art_res()).abs() < 1e-3, "standing body is {STAND_UNITS} units tall");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn lying_body_gets_a_low_wide_hurtbox() {
        let dir = fixture("hurt");
        write_png(&dir, "01_idle", 10, 10);
        write_png(&dir, "14_lie", 10, 10);
        let roots = [dir.clone()];
        assert_eq!(load_pose(&roots, "lie", false).unwrap().hurt, vec![LYING_HURT]);
        assert_eq!(load_pose(&roots, "idle0", false).unwrap().hurt, vec![STAND_HURT]);
        assert_eq!(load_pose(&roots, "crouch1", false).unwrap().hurt, vec![CROUCH_HURT]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_art_returns_none_so_the_rig_takes_over() {
        let dir = fixture("none");
        assert!(load_pose(std::slice::from_ref(&dir), "idle0", false).is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
