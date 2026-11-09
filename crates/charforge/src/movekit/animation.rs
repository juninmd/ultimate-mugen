use super::kinds::KINDS;
use super::sprites::SpriteLibrary;
use super::value::{get_int, get_text};
use crate::spritekit::{anatomical_boxes, attack_boxes, box_lines, Box4};
use toml::{Table, Value};

const SPECIAL_KINDS: [&str; 4] = ["special", "air_special", "super", "projectile"];

pub fn grow_attack_box(boxes: &[Box4], height: f64, kind: &str) -> Vec<Box4> {
    if boxes.is_empty() || height <= 0.0 {
        return boxes.to_vec();
    }
    let x0 = boxes.iter().map(|b| b.0).min().unwrap_or(0);
    let mut y0 = boxes.iter().map(|b| b.1).min().unwrap_or(0) as f64;
    let x1 = boxes.iter().map(|b| b.2).max().unwrap_or(0);
    let mut y1 = boxes.iter().map(|b| b.3).max().unwrap_or(0) as f64;
    if x1 <= 0 {
        return boxes.to_vec();
    }
    let special = SPECIAL_KINDS.contains(&kind);
    let base = (if special { 0.85 } else { 0.7 }) * height;
    let cap = if special { 0.5 } else { 0.35 };
    let reach = (x1 as f64 + 0.12 * height).max(base).min(x1 as f64 + cap * height);
    let minimum = 0.6 * height;
    if y1 - y0 < minimum {
        let center = (y0 + y1) / 2.0;
        y0 = center - minimum / 2.0;
        y1 = center + minimum / 2.0;
        if y1 > 0.0 {
            y0 -= y1;
            y1 = 0.0;
        }
    }
    vec![(x0.min(0), y0.round() as i32, reach.round() as i32, y1.round() as i32)]
}

fn explicit_boxes(options: &Table, key: &str) -> Option<Vec<Box4>> {
    let list = options.get(key)?.as_array()?;
    Some(list.iter().filter_map(Value::as_array).map(|b| {
        let n = |i: usize| b.get(i).and_then(Value::as_integer).unwrap_or(0) as i32;
        (n(0), n(1), n(2), n(3))
    }).collect())
}

pub fn build_action(move_: &Table, library: &mut SpriteLibrary, height: Option<f64>) -> Result<(i64, String), String> {
    let number = get_int(move_, "anim", get_int(move_, "state", 0));
    let kind = get_text(move_, "kind").unwrap_or_else(|| "normal".into());
    if !KINDS.contains(&kind.as_str()) {
        return Err(format!("unknown kind {kind}"));
    }
    let loop_start = get_int(move_, "loop", 0) as usize;
    let mut lines = vec![format!("[Begin Action {number}]")];
    let frames = move_.get("frames").and_then(Value::as_array).ok_or("move has no frames")?;
    for (index, frame) in frames.iter().enumerate() {
        let parts = frame.as_array().ok_or("a frame must be [group, item, ticks, ...]")?;
        let at = |i: usize| parts.get(i).and_then(Value::as_integer).ok_or("frame needs integer group, item and ticks");
        let (group, item, ticks) = (at(0)?, at(1)?, at(2)?);
        let hit = parts.iter().skip(3).any(|p| p.as_str() == Some("hit"));
        let options = parts.iter().skip(3).find_map(Value::as_table).cloned().unwrap_or_default();
        if loop_start > 0 && index == loop_start {
            lines.push("Loopstart".into());
        }
        let found = if group >= 0 { library.mask_and_axis(group, item)? } else { None };
        let body = explicit_boxes(&options, "clsn2").unwrap_or_else(|| found.map(|(m, a)| anatomical_boxes(m, a, 2, 2)).unwrap_or_default());
        let attack = explicit_boxes(&options, "clsn1").unwrap_or_else(|| {
            let detected = if hit { found.map(|(m, a)| attack_boxes(m, a, 2)).unwrap_or_default() } else { Vec::new() };
            match height {
                Some(h) => grow_attack_box(&detected, h, &kind),
                None => detected,
            }
        });
        lines.extend(box_lines(2, &body));
        lines.extend(box_lines(1, &attack));
        let mut line = format!("{group},{item}, {},{}, {ticks}", get_int(&options, "x", 0), get_int(&options, "y", 0));
        if options.contains_key("flip") || options.contains_key("trans") {
            line += &format!(", {}, {}", get_text(&options, "flip").unwrap_or_default(), get_text(&options, "trans").unwrap_or_default());
        }
        lines.push(line);
    }
    Ok((number, lines.join("\n") + "\n"))
}

pub fn hit_frames(action_text: &str) -> Vec<i64> {
    let mut frames = Vec::new();
    let mut index = 0;
    for line in action_text.lines().map(str::trim) {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("clsn1:") {
            frames.push(index + 1);
        } else if !line.is_empty() && !lower.starts_with("clsn") && !lower.starts_with('[') && !lower.starts_with("loopstart") {
            index += 1;
        }
    }
    frames
}
