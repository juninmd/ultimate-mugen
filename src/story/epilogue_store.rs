use super::epilogue::{merged, parse, Epilogue, Table};
use std::sync::RwLock;

pub const FILE: &str = "story-epilogues.toml";

static ACTIVE: RwLock<Option<&'static Table>> = RwLock::new(None);

pub fn parse_toml(text: &str) -> Option<Table> {
    toml::from_str(text).ok()
}

pub fn overrides() -> Table {
    crate::storage::read(FILE)
        .and_then(|text| parse_toml(&text))
        .unwrap_or_default()
}

fn build() -> Table {
    let mut table = merged(crate::storage::read(super::epilogue::FILE).and_then(|text| parse(&text)));
    table.extend(overrides());
    table
}

pub fn table() -> &'static Table {
    if let Some(table) = *ACTIVE.read().unwrap() {
        return table;
    }
    let mut slot = ACTIVE.write().unwrap();
    *slot.get_or_insert_with(|| Box::leak(Box::new(build())))
}

fn persist(over: &Table) -> Result<(), String> {
    let text = if over.is_empty() { String::new() } else { toml::to_string(over).map_err(|e| format!("epilogue toml: {e}"))? };
    if !text.is_empty() && parse_toml(&text).is_none() {
        return Err("epilogue toml: saved text does not parse back".into());
    }
    crate::storage::write(FILE, &text);
    *ACTIVE.write().unwrap() = Some(Box::leak(Box::new(build())));
    Ok(())
}

pub fn save_override(id: &str, epilogue: Epilogue) -> Result<(), String> {
    let mut over = overrides();
    over.insert(id.to_string(), epilogue);
    persist(&over)
}

pub fn clear_override(id: &str) -> Result<(), String> {
    let mut over = overrides();
    over.remove(id);
    persist(&over)
}

pub fn has_override(id: &str) -> bool {
    overrides().contains_key(id)
}

pub fn ids() -> Vec<String> {
    table().keys().cloned().collect()
}
