use super::value::{get_int, get_text, tables};
use std::collections::HashMap;
use toml::{Table, Value};

fn resolve(names: &HashMap<String, i64>, target: &str, base: i64) -> Result<i64, String> {
    if let Ok(number) = target.parse::<i64>() {
        return Ok(number);
    }
    names.get(target).copied().ok_or_else(|| format!("move {base}: phase {target:?} does not exist"))
}

pub fn expand_phases(move_: &Table) -> Result<Vec<Table>, String> {
    let phases = tables(move_, "phases");
    if phases.is_empty() {
        return Ok(vec![move_.clone()]);
    }
    let base = get_int(move_, "state", 0);
    let kind = get_text(move_, "kind").unwrap_or_else(|| "normal".into());
    let mut names: HashMap<String, i64> = HashMap::from([("start".to_string(), base)]);
    let mut built: Vec<Table> = Vec::new();
    for (position, phase) in phases.iter().enumerate() {
        let mut merged = match get_text(phase, "like") {
            Some(like) => {
                let source = built.iter().find(|p| get_text(p, "name").as_deref() == Some(like.as_str()))
                    .ok_or_else(|| format!("move {base}: like = {like:?} must name an earlier phase"))?;
                let mut copy = source.clone();
                for key in ["name", "state", "_phase_of"] {
                    copy.remove(key);
                }
                copy
            }
            None => Table::new(),
        };
        for (key, value) in phase.iter().filter(|(k, _)| k.as_str() != "like") {
            merged.insert(key.clone(), value.clone());
        }
        merged.insert("kind".into(), Value::String(kind.clone()));
        merged.insert("_phase_of".into(), Value::Integer(base));
        let state = merged.get("state").and_then(Value::as_integer).unwrap_or(base + position as i64 + 1);
        merged.insert("state".into(), Value::Integer(state));
        if let Some(name) = get_text(&merged, "name") {
            names.insert(name, state);
        }
        built.push(merged);
    }
    let mut root = move_.clone();
    root.remove("phases");
    let mut all = vec![root];
    all.extend(built);
    let chain: Vec<usize> = (0..all.len()).filter(|&i| i == 0 || all[i].get("helper").and_then(Value::as_bool) != Some(true)).collect();
    for (position, &index) in chain.iter().enumerate() {
        let next = match get_text(&all[index], "next") {
            Some(target) => Some(resolve(&names, &target, base)?),
            None => chain.get(position + 1).map(|&n| get_int(&all[n], "state", 0)),
        };
        if let Some(next) = next {
            all[index].insert("_next".into(), Value::Integer(next));
        }
    }
    Ok(all)
}
