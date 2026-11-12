//! Resolution rules mirroring `crates/engine/src/native/pixel.rs`: which poses fall back to the
//! idle canvas and must therefore get a dedicated PNG rendered by `generate_missing_sprites`.
use serde_json::Value;
use std::collections::BTreeSet;

use tiny_skia::Pixmap;

const BASE_STEM: &str = "01_idle";
const MAX_NAME: usize = 64;
const MAX_SRC_PIXELS: u64 = 24_000_000;

/// Only plain identifiers may become file names: UMF JSON is importable user content, so ids and
/// pose names must never carry path separators, absolute paths or drive letters.
pub fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_NAME
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub fn to_pixmap(img: &ume_engine::chardef::ImageData) -> Option<Pixmap> {
    if img.w as u64 * img.h as u64 > MAX_SRC_PIXELS {
        return None;
    }
    let mut pm = Pixmap::new(img.w, img.h)?;
    for (d, s) in pm
        .data_mut()
        .chunks_exact_mut(4)
        .zip(img.rgba.chunks_exact(4))
    {
        let a = s[3] as u32;
        for i in 0..3 {
            d[i] = ((s[i] as u32 * a + 127) / 255) as u8;
        }
        d[3] = s[3];
    }
    Some(pm)
}

fn has(needles: &[&str], lower: &str) -> bool {
    needles.iter().any(|needle| lower.contains(needle))
}

fn basic_stem(lower: &str) -> &'static str {
    if has(
        &[
            "idle", "walk", "run", "stand", "turn", "crouch", "land", "hop", "guard", "getup",
            "recover",
        ],
        lower,
    ) {
        BASE_STEM
    } else if has(&["jab", "punch"], lower) || lower.starts_with("str") {
        "02_attack1"
    } else if has(&["kick", "lk", "hk", "sweep"], lower) {
        "03_attack2"
    } else if has(&["special", "super", "arc", "frost", "cut", "rise"], lower)
        || lower.starts_with("sp")
    {
        "04_special"
    } else if has(&["proj", "blast", "fireball", "wave", "gale"], lower) {
        "05_projectile"
    } else if has(&["win", "vic", "taunt", "intro"], lower) {
        "06_victory"
    } else {
        BASE_STEM
    }
}

fn pose_stem(lower: &str) -> &'static str {
    match lower {
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
        _ if lower.starts_with("df") => "14_lie",
        _ => basic_stem(lower),
    }
}

fn inline_pose_key(pose: &Value) -> String {
    let bytes = serde_json::to_vec(pose).unwrap_or_default();
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ *byte as u64).wrapping_mul(0x100000001b3)
    });
    format!("inline_{hash:016x}")
}

pub fn frame_targets(
    frame: &Value,
    files: &BTreeSet<String>,
    poses: &Value,
    exact_only: bool,
) -> Option<(String, Value)> {
    let is_effect = frame
        .get("effect")
        .map_or(false, |effect| !effect.is_null());
    if is_effect || frame.get("sprite").is_some() {
        return None;
    }
    let pose = frame.get("pose").filter(|pose| !pose.is_null())?;
    if let Some(name) = pose.as_str() {
        let lower = name.to_ascii_lowercase();
        let pose_value = poses.get(name)?;
        if lower.starts_with("df") {
            if files.contains(name) {
                return None;
            }
            return Some((name.to_string(), pose_value.clone()));
        }
        let stem = pose_stem(&lower);
        if lower.starts_with("idle") {
            return None;
        }
        if files.contains(name) {
            return None;
        }
        if !exact_only && stem != BASE_STEM && files.contains(stem) {
            return None;
        }
        return Some((name.to_string(), pose_value.clone()));
    }
    if pose.is_object() {
        let key = inline_pose_key(pose);
        if files.contains(&key) {
            return None;
        }
        return Some((key, pose.clone()));
    }
    None
}

pub fn attack_animations(character: &Value) -> BTreeSet<String> {
    character
        .get("moves")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .chain(
            character
                .pointer("/abilities/forms")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .flat_map(|form| {
                    form.get("moves")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                }),
        )
        .filter_map(|attack| {
            attack
                .get("anim")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect()
}
