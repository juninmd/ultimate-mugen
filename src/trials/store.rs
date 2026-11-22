use serde_json::{Map, Value};

const FILE: &str = "trials.json";

fn table() -> Map<String, Value> {
    crate::storage::read(FILE)
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

pub fn load(fighter: &str) -> usize {
    table().get(fighter).and_then(Value::as_u64).map_or(0, |n| (n as usize).min(super::LADDER.len()))
}

pub fn save(fighter: &str, cleared: usize) {
    let mut t = table();
    let best = load(fighter).max(cleared.min(super::LADDER.len()));
    t.insert(fighter.to_string(), Value::from(best as u64));
    crate::storage::write(FILE, &Value::Object(t).to_string());
}
