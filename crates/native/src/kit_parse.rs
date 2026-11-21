use super::kit::{expand, parent, parents_of, remember_base};
use super::types::indexmap_like::OrderedMap;
use super::types::NativeChar;
use serde::de::IgnoredAny;
use serde::Deserialize;
use serde_json::Value;

const MAX_DEPTH: usize = 8;

#[derive(Deserialize)]
struct Outline {
    #[serde(default)]
    anims: Option<OrderedMap<IgnoredAny>>,
}

pub(super) fn own_anims(text: &str) -> Vec<String> {
    serde_json::from_str::<Outline>(text)
        .ok()
        .and_then(|outline| outline.anims)
        .map(|anims| anims.0.into_iter().map(|(name, _)| name).collect())
        .unwrap_or_default()
}

fn inherited_anims(value: &Value, depth: usize, out: &mut Vec<String>) -> Result<(), String> {
    if depth >= MAX_DEPTH {
        return Ok(());
    }
    for id in parents_of(value)? {
        let (kit, anims) = parent(&id)?;
        inherited_anims(&kit, depth + 1, out)?;
        for name in anims {
            if !out.contains(&name) {
                out.push(name);
            }
        }
    }
    Ok(())
}

pub fn uses_kits(json: &str) -> bool {
    json.contains("\"extends\"") || json.contains("\"use\"")
}

pub fn parse_char(json: &str) -> Result<NativeChar, String> {
    if !uses_kits(json) {
        let fighter: NativeChar = serde_json::from_str(json).map_err(|error| error.to_string())?;
        remember_base(&fighter.id, json, None);
        return Ok(fighter);
    }
    let raw: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let mut order = Vec::new();
    inherited_anims(&raw, 0, &mut order)?;
    for name in own_anims(json) {
        if !order.contains(&name) {
            order.push(name);
        }
    }
    let expanded = expand(raw)?;
    let mut fighter: NativeChar = serde_json::from_value(expanded.clone())
        .map_err(|error| format!("after applying `extends`/`use`: {error}"))?;
    fighter.anims.0.sort_by_key(|(name, _)| {
        order
            .iter()
            .position(|known| known == name)
            .unwrap_or(usize::MAX)
    });
    remember_base(&fighter.id, &expanded.to_string(), Some(order));
    Ok(fighter)
}
