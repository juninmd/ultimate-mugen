use super::rig::{art_res, RigRender};
use super::types::{NativeChar, NativeFrame};
use crate::chardef::{ImageData, Sprite};
use serde_json::Value;
use std::path::{Path, PathBuf};

const SPRITE_ROOT: &str = "assets/chars/sprites";
const BASE_STEM: &str = "01_idle";
const STAND_HURT: [f32; 4] = [-18.0, -96.0, 18.0, 0.0];
const CROUCH_HURT: [f32; 4] = [-16.0, -60.0, 16.0, 0.0];
const LYING_HURT: [f32; 4] = [-46.0, -26.0, 46.0, 0.0];
const STRIKE_BOX: [f32; 4] = [2.0, -84.0, 38.0, -32.0];
const STAND_UNITS: f32 = 100.0;
const MAX_PORTRAIT_EDGE: u32 = 256;

pub(super) fn try_load(nc: &NativeChar, fr: &NativeFrame) -> Option<RigRender> {
    if fr.effect.is_some() { return None; }
    let mut render = load_pose(&sprite_roots(&nc.id), &native_frame_sprite_key(fr), fr.hit.is_some())?;
    let height = nc.stats.height.filter(|height| height.is_finite() && *height > 0.0).unwrap_or(STAND_UNITS as f64) as f32;
    let height_ratio = height / STAND_UNITS;
    render.scale *= height_ratio;
    for bounds in &mut render.hurt {
        bounds[1] *= height_ratio;
        bounds[3] *= height_ratio;
    }
    Some(render)
}

pub(super) fn portrait(nc: &NativeChar) -> Option<Sprite> {
    sprite_roots(&nc.id).iter().find_map(|root| {
        let bytes = std::fs::read(root.join("portrait.png")).ok()?;
        let png = ume_formats::png::decode_png(&bytes).ok()?;
        let (width, height, rgba) = portrait_thumbnail(png.width, png.height, png.rgba?);
        Some(Sprite { image: Some(ImageData::new(width, height, rgba)), width: width as f32, height: height as f32, axis_x: 0.0, axis_y: 0.0, scale: 1.0 })
    })
}

fn portrait_thumbnail(width: u32, height: u32, rgba: Vec<u8>) -> (u32, u32, Vec<u8>) {
    if width.max(height) <= MAX_PORTRAIT_EDGE { return (width, height, rgba); }
    let (target_width, target_height) = if width >= height {
        (MAX_PORTRAIT_EDGE, ((height as u64 * MAX_PORTRAIT_EDGE as u64 + width as u64 / 2) / width as u64).max(1) as u32)
    } else {
        (((width as u64 * MAX_PORTRAIT_EDGE as u64 + height as u64 / 2) / height as u64).max(1) as u32, MAX_PORTRAIT_EDGE)
    };
    let mut resized = Vec::with_capacity((target_width * target_height * 4) as usize);
    for y in 0..target_height {
        let source_y = ((2 * y as u64 + 1) * height as u64 / (2 * target_height as u64)).min(height as u64 - 1) as u32;
        for x in 0..target_width {
            let source_x = ((2 * x as u64 + 1) * width as u64 / (2 * target_width as u64)).min(width as u64 - 1) as u32;
            let source = ((source_y as u64 * width as u64 + source_x as u64) * 4) as usize;
            resized.extend_from_slice(&rgba[source..source + 4]);
        }
    }
    (target_width, target_height, resized)
}

pub fn native_frame_sprite_key(fr: &NativeFrame) -> String {
    match (&fr.sprite, &fr.pose) {
        (Some(s), _) => s.clone(),
        (None, Some(Value::String(p))) => p.clone(),
        (None, Some(pose)) => {
            let bytes = serde_json::to_vec(pose).unwrap_or_default();
            let digest = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| (hash ^ *byte as u64).wrapping_mul(0x100000001b3));
            format!("inline_{digest:016x}")
        }
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
    load_pose_with_source(roots, pose, striking).map(|(render, _)| render)
}

fn load_pose_with_source(roots: &[PathBuf], pose: &str, striking: bool) -> Option<(RigRender, PathBuf)> {
    let lower = pose.to_ascii_lowercase();
    let stem = pose_stem(&lower);
    [pose, stem, BASE_STEM, "idle"]
        .iter()
        .flat_map(|name| roots.iter().map(move |root| (root, name)))
        .find_map(|(root, name)| read_render(root, name, &lower, stem, striking).map(|render| (render, root.join(format!("{name}.png")))))
}

pub fn audit_native_sprite_frames(character: &NativeChar) -> Value {
    let character = &super::compile::with_defaults(character);
    use std::hash::{Hash, Hasher};
    let roots = sprite_roots(&character.id);
    let animations: Vec<Value> = character.anims.iter().map(|(animation, data)| {
        let frames: Vec<Value> = data.frames.iter().enumerate().map(|(index, frame)| {
            let pose = native_frame_sprite_key(frame);
            if frame.effect.is_some() {
                return serde_json::json!({"frame": index, "pose": pose, "resolution": "procedural-effect"});
            }
            match load_pose_with_source(&roots, &pose, frame.hit.is_some()) {
                Some((render, source)) => {
                    let stem = source.file_stem().and_then(|name| name.to_str()).unwrap_or_default();
                    let lower = pose.to_ascii_lowercase();
                    let resolution = if stem == pose { "exact" } else if stem == BASE_STEM && !lower.starts_with("idle") { "idle-reuse" } else if stem == pose_stem(&lower) { "shared-pose" } else { "fallback-idle" };
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    if let Some(image) = render.image { image.w.hash(&mut hasher); image.h.hash(&mut hasher); image.rgba.hash(&mut hasher); }
                    serde_json::json!({"frame": index, "pose": pose, "resolution": resolution, "file": source, "pixelFingerprint": format!("{:016x}", hasher.finish())})
                }
                None => serde_json::json!({"frame": index, "pose": pose, "resolution": "missing"}),
            }
        }).collect();
        let unique: std::collections::HashSet<&str> = frames.iter().filter_map(|frame| frame.get("pixelFingerprint").and_then(Value::as_str)).collect();
        serde_json::json!({"animation": animation, "uniqueBodyImages": unique.len(), "frames": frames})
    }).collect();
    serde_json::json!({"id": character.id, "declaredHeight": character.stats.height, "avatar": roots.iter().any(|root| root.join("portrait.png").is_file()), "animations": animations})
}

fn read_render(root: &Path, name: &str, lower: &str, stem: &str, striking: bool) -> Option<RigRender> {
    let bytes = std::fs::read(root.join(format!("{name}.png"))).ok()?;
    let dec = ume_formats::png::decode_png(&bytes).ok()?;
    let rgba = dec.rgba?;
    let res = art_res().max(0.1);
    let meta: Value = std::fs::read(root.join("meta.json")).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or(Value::Null);
    let frame = meta.get("frames").and_then(|frames| frames.get(name)).unwrap_or(&Value::Null);
    let positive = |value: Option<&Value>| value.and_then(Value::as_f64).filter(|number| number.is_finite() && *number > 0.0 && *number <= f32::MAX as f64).map(|number| number as f32);
    let units = positive(frame.get("refHeight")).or_else(|| positive(meta.get("refHeight"))).unwrap_or(dec.height as f32).max(1.0);
    let axis = |key: &str, default: f32, limit: f32| frame.get(key).and_then(Value::as_f64).filter(|number| number.is_finite() && *number >= 0.0 && *number <= limit as f64).map_or(default, |number| number as f32);
    let axis_x = axis("axisX", dec.width as f32 / 2.0, dec.width as f32);
    let axis_y = axis("axisY", dec.height as f32, dec.height as f32);
    Some(RigRender {
        image: Some(ImageData::new(dec.width, dec.height, rgba)),
        width: dec.width as f32 * res,
        height: dec.height as f32 * res,
        axis_x: axis_x * res,
        axis_y: axis_y * res,
        scale: STAND_UNITS / units / res,
        hurt: vec![hurtbox(lower, stem)],
        hit: if striking { vec![STRIKE_BOX] } else { vec![] },
    })
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
    fn frame_geometry_overrides_preserve_floor_anchor_and_body_scale() {
        let dir = fixture("frame-geometry");
        write_png(&dir, "walk0", 400, 600);
        std::fs::write(dir.join("meta.json"), r#"{"refHeight":200,"frames":{"walk0":{"refHeight":400,"axisX":180,"axisY":560}}}"#).unwrap();
        let render = load_pose(std::slice::from_ref(&dir), "walk0", false).unwrap();
        assert!((render.axis_y * render.scale - 140.0).abs() < 0.001);
        assert!((render.axis_x * render.scale - 45.0).abs() < 0.001);
        assert!((400.0 * art_res() * render.scale - 100.0).abs() < 0.001);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn declared_height_scales_body_and_hurtboxes_together() {
        let mut fighter: NativeChar = serde_json::from_str(include_str!("../../../../assets/chars/bruno.json")).unwrap();
        let frame: NativeFrame = serde_json::from_value(serde_json::json!({"pose": "idle0", "ticks": 1})).unwrap();
        fighter.stats.height = Some(100.0);
        let baseline = try_load(&fighter, &frame).unwrap();
        fighter.stats.height = Some(116.0);
        let taller = try_load(&fighter, &frame).unwrap();
        assert!((taller.scale / baseline.scale - 1.16).abs() < 0.001);
        assert!((taller.hurt[0][1] / baseline.hurt[0][1] - 1.16).abs() < 0.001);
        fighter.stats.height = Some(f64::NAN);
        assert_eq!(try_load(&fighter, &frame).unwrap().scale, baseline.scale);
    }

    #[test]
    fn effect_frames_do_not_fall_back_to_fighter_idle() {
        let fighter: NativeChar = serde_json::from_str(include_str!("../../../../assets/chars/bruno.json")).unwrap();
        let frame: NativeFrame = serde_json::from_value(serde_json::json!({"effect": {"kind": "orb"}, "ticks": 1})).unwrap();
        assert!(try_load(&fighter, &frame).is_none());
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
