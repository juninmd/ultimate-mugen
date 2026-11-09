use serde_json::{json, Map, Value};
use std::path::Path;

const FEATURES: [&str; 11] = ["cancels", "armor", "parry", "throw", "timestop", "teleport", "flight", "burst", "invuln", "projectile", "airPhysics"];

fn truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        Some(Value::Bool(true)) => true,
    }
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn number(value: &Value, key: &str, default: f64) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(default)
}

fn projectiles(m: &Value) -> Vec<&Value> {
    match m.get("projectile") {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(single) if truthy(Some(single)) => vec![single],
        _ => Vec::new(),
    }
}

fn kind_count(moves: &[Value], kind: &str) -> usize {
    moves.iter().filter(|m| text(m, "kind") == kind).count()
}

fn hit_problems(m: &Value, anims: &Map<String, Value>, problems: &mut Vec<String>) {
    let id = text(m, "id");
    let Some(anim) = anims.get(text(m, "anim")) else {
        problems.push(format!("{id}: anim '{}' missing", text(m, "anim")));
        return;
    };
    let frames = anim["frames"].as_array().cloned().unwrap_or_default();
    for hit in m.get("hits").and_then(Value::as_array).into_iter().flatten() {
        let at = hit["at"].as_i64().unwrap_or(0);
        if at < 1 || at as usize > frames.len() {
            problems.push(format!("{id}: hit at {at} outside anim ({} frames)", frames.len()));
            continue;
        }
        let frame = &frames[at as usize - 1];
        if frame.get("hit").is_none() && frame.get("effect").is_none() {
            problems.push(format!("{id}: hit at frame {at} has no hit box"));
        } else if let Some(h) = frame.get("hit") {
            if number(h, "radius", 99.0) < 7.0 && text(m, "kind") != "normal" {
                problems.push(format!("{id}: tiny hit radius {}", number(h, "radius", 99.0)));
            }
        }
    }
    for p in projectiles(m) {
        if !anims.contains_key(text(p, "anim")) {
            problems.push(format!("{id}: projectile anim {} missing", text(p, "anim")));
        }
    }
    if text(m, "kind") != "normal" && !truthy(m.get("hits")) && !truthy(m.get("projectile")) && !truthy(m.get("throw")) {
        problems.push(format!("{id}: special has no damage source"));
    }
}

fn ground_problems(m: &Value, anims: &Map<String, Value>, problems: &mut Vec<String>) {
    let Some(anim) = anims.get(text(m, "anim")) else { return };
    let airborne = text(m, "when") == "air" || m.get("moves").and_then(Value::as_array).into_iter().flatten().any(|v| truthy(v.pointer("/vel/1")));
    let last = anim["frames"].as_array().and_then(|f| f.last());
    if !airborne && last.is_some_and(|f| number(f, "ticks", 1.0) < 0.0) {
        problems.push(format!("{}: ground move ends on an infinite frame (never returns control)", text(m, "id")));
    }
}

fn link_problems(moves: &[Value], anims: &Map<String, Value>, problems: &mut Vec<String>) {
    let ids: Vec<&str> = moves.iter().map(|m| text(m, "id")).collect();
    let mut seen: Vec<((String, String), String)> = Vec::new();
    for m in moves {
        let id = text(m, "id");
        let when = if text(m, "kind") == "normal" { m.get("when").and_then(Value::as_str).unwrap_or("stand").to_string() } else { String::new() };
        let key = (text(m, "input").replace(' ', ""), when);
        if let Some((_, other)) = seen.iter().find(|(k, _)| *k == key) {
            problems.push(format!("{id}: input '{}' duplicates {other}", text(m, "input")));
        }
        seen.retain(|(k, _)| *k != key);
        seen.push((key, id.to_string()));
        for cancel in m.get("cancels").and_then(Value::as_array).into_iter().flatten() {
            if !ids.contains(&cancel.as_str().unwrap_or("")) {
                problems.push(format!("{id}: cancels unknown move '{}'", cancel.as_str().unwrap_or("")));
            }
        }
        let Some(throw) = m.get("throw").filter(|t| truthy(Some(*t))) else { continue };
        let Some(anim) = anims.get(text(m, "anim")) else { continue };
        let frames = anim["frames"].as_array().cloned().unwrap_or_default();
        let at = throw.get("at").and_then(Value::as_i64).unwrap_or(1) as usize;
        if frames.len() < at + 2 {
            problems.push(format!("{id}: throw anim needs at least {} frames", at + 2));
        } else {
            let hit = frames[at - 1].get("hit");
            if hit.is_none() || (hit.is_some_and(|h| h.get("box").is_none()) && number(hit.unwrap_or(&Value::Null), "radius", 0.0) < 20.0) {
                problems.push(format!("{id}: throw grab frame {at} needs a hit box (radius >= 20 or box)"));
            }
        }
    }
}

fn max_super_damage(moves: &[Value]) -> f64 {
    moves.iter().filter(|m| text(m, "kind") == "super").map(|m| {
        let hits: f64 = m.get("hits").and_then(Value::as_array).into_iter().flatten().map(|h| number(h, "damage", 0.0)).sum();
        let shots: f64 = projectiles(m).iter().map(|p| number(p, "damage", 0.0) * number(p, "hits", 1.0)).sum();
        hits + shots
    }).fold(0.0, f64::max)
}

pub fn audit(character: &Value) -> Value {
    let moves = character["moves"].as_array().cloned().unwrap_or_default();
    let empty = Map::new();
    let anims = character["anims"].as_object().unwrap_or(&empty);
    let mut problems = Vec::new();
    let mut features: Vec<(&str, u32)> = Vec::new();
    for m in &moves {
        for key in FEATURES.iter().filter(|k| truthy(m.get(**k))) {
            match features.iter_mut().find(|(k, _)| k == key) {
                Some(entry) => entry.1 += 1,
                None => features.push((key, 1)),
            }
        }
    }
    for m in &moves {
        hit_problems(m, anims, &mut problems);
    }
    for m in &moves {
        ground_problems(m, anims, &mut problems);
    }
    link_problems(&moves, anims, &mut problems);
    let (specials, supers) = (kind_count(&moves, "special"), kind_count(&moves, "super"));
    if !moves.iter().any(|m| truthy(m.get("cancels"))) {
        problems.push("no explicit cancel chains (only default normal>special cancel)".into());
    }
    if specials < 5 {
        problems.push(format!("only {specials} specials"));
    }
    if supers < 2 {
        problems.push(format!("only {supers} super(s)"));
    }
    let area = moves.iter().filter(|m| text(m, "kind") == "super").any(|m| projectiles(m).first().is_some_and(|p| number(p, "hits", 1.0) > 1.0));
    if !area {
        problems.push("no multi-hit area super".into());
    }
    if !moves.iter().any(|m| truthy(m.get("throw"))) {
        problems.push("no throw".into());
    }
    if !moves.iter().any(|m| truthy(m.get("armor")) || truthy(m.get("parry"))) {
        problems.push("no armor/parry tool".into());
    }
    let feature_map: Map<String, Value> = features.iter().map(|(k, v)| (k.to_string(), json!(v))).collect();
    json!({
        "id": character["id"], "problems": problems, "specials": specials, "supers": supers,
        "normals": kind_count(&moves, "normal"), "features": feature_map,
        "abilities": character.get("abilities").and_then(Value::as_object).map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
        "maxSuperDamage": max_super_damage(&moves),
    })
}

pub fn audit_directory(dir: &Path) -> Result<Vec<Value>, String> {
    let mut paths: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.filter_map(Result::ok).map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json") && !p.to_string_lossy().ends_with("index.json")).collect();
    paths.sort();
    paths.iter().map(|p| serde_json::from_slice::<Value>(&std::fs::read(p).map_err(|e| e.to_string())?).map(|c| audit(&c)).map_err(|e| format!("{}: {e}", p.display()))).collect()
}

pub fn report_lines(results: &[Value]) -> Vec<String> {
    let mut lines = Vec::new();
    for r in results {
        let features = r["features"].as_object().map(|o| o.iter().map(|(k, v)| format!("'{k}': {v}")).collect::<Vec<_>>().join(", ")).unwrap_or_default();
        let abilities = r["abilities"].as_array().map(|a| a.iter().map(|v| format!("'{}'", v.as_str().unwrap_or(""))).collect::<Vec<_>>().join(", ")).unwrap_or_default();
        lines.push(format!("{:8} n={} sp={} su={} superDmg={:.0} feats={{{features}}} abil=[{abilities}]", r["id"].as_str().unwrap_or(""), r["normals"], r["specials"], r["supers"], r["maxSuperDamage"].as_f64().unwrap_or(0.0)));
        lines.extend(r["problems"].as_array().into_iter().flatten().map(|p| format!("   - {}", p.as_str().unwrap_or(""))));
    }
    lines.push(format!("total problems: {}", results.iter().map(|r| r["problems"].as_array().map(Vec::len).unwrap_or(0)).sum::<usize>()));
    lines
}
