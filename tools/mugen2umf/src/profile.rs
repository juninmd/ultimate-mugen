use crate::source::{base_dir, Source};
use serde_json::{json, Map, Value};
use std::path::Path;
use ume_formats::vfs::text;

const NOTES: [&str; 4] = [".txt", ".nfo", ".md", ".diz"];
const TOP_LEVEL: [&str; 8] = ["series", "description", "intro", "taunt", "winQuote", "color", "author", "name"];
const NOTE_BYTES: usize = 6000;

pub fn fields() -> Value {
    json!({
        "series": "franchise or game the character comes from (string)",
        "description": "one sentence about the character (string)",
        "intro": "round-start line", "taunt": "provocation line", "winQuote": "victory line",
        "profile": {
            "country": "ISO 3166-1 alpha-2 code of the character's nationality, e.g. RU, JP, US",
            "nationality": "{en, pt, es}", "origin": "{en, pt, es} birthplace or homeworld", "bio": "{en, pt, es} 2-3 sentences",
            "title": "{en, pt, es} epithet", "fightingStyle": "{en, pt, es}", "affiliation": "{en, pt, es}",
            "element": "optional element id used by the game (fire, ice, lightning, wind, water, ki, dark, light)",
            "alignment": "hero | villain | neutral", "tags": "[strings]"
        }
    })
}

pub fn facts(src: &Source) -> Value {
    let secs = src.vfs.get(&src.def).map(|b| ume_formats::ini::parse_ini_no_lines(&text(b))).unwrap_or_default();
    let info: Map<String, Value> = ume_formats::ini::find_section(&secs, "info").map(|s| s.entries.iter().map(|(k, v)| (k.clone(), json!(v.trim_matches('"')))).collect()).unwrap_or_default();
    let base = base_dir(&src.def).to_ascii_lowercase();
    let mut notes = Map::new();
    for key in src.vfs.keys() {
        let lower = key.to_ascii_lowercase();
        if lower.starts_with(&base) && NOTES.iter().any(|ext| lower.ends_with(ext)) {
            if let Some(bytes) = src.vfs.get(key) {
                let body: String = text(bytes).chars().take(NOTE_BYTES).collect();
                notes.insert(key.clone(), json!(body));
            }
        }
    }
    json!({"def": src.def, "info": info, "notes": notes, "fill": fields()})
}

pub fn apply(json_path: &Path, patch: &Value) -> Result<String, String> {
    let raw = std::fs::read_to_string(json_path).map_err(|e| format!("{}: {e}", json_path.display()))?;
    let mut fighter: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let patch = patch.as_object().ok_or("profile patch must be a JSON object")?;
    let obj = fighter.as_object_mut().ok_or("UMF root must be an object")?;
    let mut changed = vec![];
    for (k, v) in patch {
        if k == "profile" {
            let merged = match (obj.get("profile").cloned(), v) {
                (Some(Value::Object(mut old)), Value::Object(new)) => {
                    old.extend(new.clone());
                    Value::Object(old)
                }
                _ => v.clone(),
            };
            serde_json::from_value::<ume_native::profile::CharProfile>(merged.clone()).map_err(|e| format!("profile: {e}"))?;
            obj.insert("profile".into(), merged);
        } else if TOP_LEVEL.contains(&k.as_str()) {
            obj.insert(k.clone(), v.clone());
        } else {
            return Err(format!("field `{k}` is not a UMF profile field (allowed: profile, {})", TOP_LEVEL.join(", ")));
        }
        changed.push(k.clone());
    }
    let text = serde_json::to_string_pretty(&fighter).map_err(|e| e.to_string())?;
    ume_native::parse_char(&text)?;
    std::fs::write(json_path, text).map_err(|e| e.to_string())?;
    Ok(format!("updated {}: {}", json_path.display(), changed.join(", ")))
}
