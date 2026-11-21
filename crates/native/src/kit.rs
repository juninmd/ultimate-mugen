use super::kit_merge::layer;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub const KIT_FORMAT: &str = "umf-kit/1";
const MAX_DEPTH: usize = 8;
const PERSONAL_KEYS: [&str; 6] = ["name", "boss", "npc", "description", "author", "profile"];
const BUILT_IN: [&str; 1] = [include_str!("kits/core.json")];

pub(super) struct Kit {
    pub value: Value,
    pub anims: Vec<String>,
}

type Kits = RwLock<HashMap<String, Kit>>;

fn entry(text: &str) -> Result<(String, Kit), String> {
    let value: Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .ok_or("a kit or base fighter needs an \"id\"")?
        .to_string();
    Ok((
        id,
        Kit {
            value,
            anims: super::kit_parse::own_anims(text),
        },
    ))
}

fn registry() -> &'static Kits {
    static KITS: OnceLock<Kits> = OnceLock::new();
    KITS.get_or_init(|| {
        RwLock::new(
            BUILT_IN
                .iter()
                .filter_map(|text| entry(text).ok())
                .collect(),
        )
    })
}

type Bases = RwLock<HashMap<String, (String, Option<Vec<String>>)>>;

fn bases() -> &'static Bases {
    static BASES: OnceLock<Bases> = OnceLock::new();
    BASES.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn is_kit(value: &Value) -> bool {
    value.get("format").and_then(Value::as_str) == Some(KIT_FORMAT)
}

pub fn is_kit_text(json: &str) -> bool {
    json.contains(KIT_FORMAT)
}

pub fn register_kit(json: &str) -> Result<String, String> {
    let (id, kit) = entry(json)?;
    if !is_kit(&kit.value) {
        return Err(format!("a kit needs \"format\": \"{KIT_FORMAT}\""));
    }
    resolve(kit.value.clone(), 0)?;
    registry().write().unwrap().insert(id.clone(), kit);
    Ok(id)
}

pub fn remember_base(id: &str, json: &str, anims: Option<Vec<String>>) {
    bases()
        .write()
        .unwrap()
        .insert(id.to_string(), (json.to_string(), anims));
}

pub fn kit_ids() -> Vec<String> {
    let mut ids: Vec<String> = registry().read().unwrap().keys().cloned().collect();
    ids.sort();
    ids
}

pub(super) fn parent(id: &str) -> Result<(Value, Vec<String>), String> {
    if let Some(kit) = registry().read().unwrap().get(id) {
        return Ok((kit.value.clone(), kit.anims.clone()));
    }
    let (text, anims) = bases()
        .read()
        .unwrap()
        .get(id)
        .cloned()
        .ok_or_else(|| format!("unknown kit or fighter `{id}` in `extends`"))?;
    entry(&text).map(|(_, kit)| (kit.value, anims.unwrap_or(kit.anims)))
}

pub(super) fn parents_of(value: &Value) -> Result<Vec<String>, String> {
    match value.get("extends") {
        None => Ok(Vec::new()),
        Some(Value::String(id)) => Ok(vec![id.clone()]),
        Some(Value::Array(ids)) => Ok(ids
            .iter()
            .filter_map(|id| id.as_str().map(str::to_string))
            .collect()),
        Some(_) => Err("`extends` must be a kit id, a fighter id or a list of them".to_string()),
    }
}

pub fn expand(value: Value) -> Result<Value, String> {
    resolve(value, 0)
}

fn needs_expansion(object: &Map<String, Value>) -> bool {
    object.contains_key("extends")
        || object
            .get("moves")
            .and_then(Value::as_array)
            .is_some_and(|moves| {
                moves.iter().any(|entry| {
                    entry.is_string() || entry.get("use").is_some() || entry.get("remove").is_some()
                })
            })
}

fn resolve(value: Value, depth: usize) -> Result<Value, String> {
    let Value::Object(mut own) = value else {
        return Ok(value);
    };
    if !needs_expansion(&own) {
        return Ok(Value::Object(own));
    }
    if depth >= MAX_DEPTH {
        return Err("kit chain is too deep (is there an `extends` cycle?)".to_string());
    }
    let parents = parents_of(&Value::Object(own.clone()))?;
    own.remove("extends");
    let mut base = Map::new();
    for id in parents {
        let Value::Object(mut kit) = resolve(parent(&id)?.0, depth + 1)? else {
            continue;
        };
        if kit.remove("format").as_ref().and_then(Value::as_str) != Some(KIT_FORMAT) {
            kit.retain(|key, _| !PERSONAL_KEYS.contains(&key.as_str()));
        }
        kit.remove("id");
        base = layer(base, kit)?;
    }
    base.insert("format".to_string(), Value::String("umf/2".to_string()));
    Ok(Value::Object(layer(base, own)?))
}

#[cfg(test)]
#[path = "kit_tests.rs"]
mod tests;
