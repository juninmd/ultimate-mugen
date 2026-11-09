use super::abilities::{blade, brute, colossus, phantom, pyro};
use super::color::shift;
use crate::json_order::{parse, to_string, Json, Style};
use std::path::Path;

pub struct Npc {
    pub base: &'static str,
    pub id: &'static str,
    pub name: &'static str,
    pub hue: f64,
    pub sat: f64,
    pub val: f64,
    pub life: f64,
    pub atk: f64,
    pub blurb: &'static str,
    pub ability: fn(&mut Json) -> Result<(), String>,
}

pub const NPCS: [Npc; 5] = [
    Npc { base: "bruno", id: "npc_brute", name: "Brute", hue: 38.0, sat: 1.0, val: 1.05, life: 0.55, atk: 0.8, blurb: "Street bruiser. His heavy swings carry armor: a light hit does not interrupt them.", ability: brute },
    Npc { base: "suzume", id: "npc_razor", name: "Razor", hue: 150.0, sat: 1.1, val: 1.0, life: 0.5, atk: 0.85, blurb: "Knife runner. Fast on his feet and lunges forward with every kick.", ability: blade },
    Npc { base: "akio", id: "npc_spitfire", name: "Spitfire", hue: 95.0, sat: 1.1, val: 1.0, life: 0.5, atk: 0.8, blurb: "Firebug. His jab throws a small orb, so he pressures from range.", ability: pyro },
    Npc { base: "daichi", id: "npc_colossus", name: "Colossus", hue: -60.0, sat: 0.9, val: 0.9, life: 0.9, atk: 0.95, blurb: "Mad Gear giant. Slow, tall and armored on every basic strike.", ability: colossus },
    Npc { base: "jin", id: "npc_phantom", name: "Phantom", hue: 210.0, sat: 0.8, val: 0.85, life: 0.5, atk: 0.85, blurb: "Shadow runner. Quick on foot and invulnerable while he kicks.", ability: phantom },
];

const KEEP_KEYS: [&str; 5] = ["skin", "eye", "lips", "beard", "weapon"];

fn recolor(colors: &Json, degrees: f64, sat: f64, val: f64) -> Json {
    let Json::Obj(items) = colors else { return colors.clone() };
    Json::Obj(items.iter().map(|(k, v)| {
        let shifted = match v.as_str() {
            Some(text) if !KEEP_KEYS.contains(&k.as_str()) => Json::Str(shift(text, degrees, sat, val)),
            _ => v.clone(),
        };
        (k.clone(), shifted)
    }).collect())
}

fn scaled(data: &mut Json, key: &str, factor: f64) {
    let value = data.get("stats").and_then(|s| s.get(key)).and_then(Json::as_f64).unwrap_or(0.0);
    if let Some(stats) = data.get_mut("stats") {
        stats.set(key, Json::int((value * factor).round_ties_even() as i64));
    }
}

fn build(spec: &Npc, chars: &Path) -> Result<Json, String> {
    let path = chars.join(format!("{}.json", spec.base));
    let mut data = parse(&std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?)?;
    data.set("id", Json::str(spec.id));
    data.set("name", Json::str(spec.name));
    data.set("series", Json::str("Story Mode · Street Thugs"));
    data.set("description", Json::str(spec.blurb));
    data.set("boss", Json::Bool(false));
    data.set("npc", Json::Bool(true));
    for key in ["intro", "taunt", "winQuote", "form", "abilities"] {
        data.remove(key);
    }
    data.set("spriteSet", Json::str(spec.base));
    let colors = data.get("art").and_then(|a| a.get("style")).and_then(|s| s.get("colors")).cloned().ok_or("art.style.colors missing")?;
    let kept = match &colors {
        Json::Obj(items) => Json::Obj(items.iter().filter(|(k, _)| !KEEP_KEYS.contains(&k.as_str())).cloned().collect()),
        other => other.clone(),
    };
    let first = recolor(&kept, spec.hue, spec.sat, spec.val);
    let second = recolor(&kept, spec.hue + 120.0, spec.sat, spec.val);
    data.set("palettes", Json::Arr(vec![first.clone(), second]));
    scaled(&mut data, "life", spec.life);
    scaled(&mut data, "attack", spec.atk);
    let color = first.get("top").or_else(|| data.get("color")).cloned().unwrap_or_else(|| Json::str("#cccccc"));
    data.set("color", color);
    (spec.ability)(&mut data)?;
    Ok(data)
}

pub fn generate(chars: &Path) -> Result<Vec<String>, String> {
    let index_path = chars.join("index.json");
    let mut index = parse(&std::fs::read_to_string(&index_path).map_err(|e| format!("{}: {e}", index_path.display()))?)?;
    let mut written = Vec::new();
    for spec in &NPCS {
        let text = to_string(&build(spec, chars)?, Style::Indent(1)) + "\n";
        let target = chars.join(format!("{}.json", spec.id));
        std::fs::write(&target, text).map_err(|e| format!("{}: {e}", target.display()))?;
        written.push(spec.id.to_string());
        let items = index.as_array_mut().ok_or("index.json must be an array")?;
        if !items.iter().any(|i| i.as_str() == Some(spec.id)) {
            items.push(Json::str(spec.id));
        }
    }
    std::fs::write(&index_path, to_string(&index, Style::Compact) + "\n").map_err(|e| e.to_string())?;
    Ok(written)
}
