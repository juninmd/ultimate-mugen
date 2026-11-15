use serde_json::{Map, Value};

pub(super) fn layer(
    mut base: Map<String, Value>,
    own: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    for (key, value) in own {
        match (key.as_str(), value) {
            (_, Value::Null) => {
                base.remove(&key);
            }
            ("moves", Value::Array(moves)) => {
                let inherited = base
                    .remove("moves")
                    .and_then(|moves| moves.as_array().cloned())
                    .unwrap_or_default();
                base.insert(key, Value::Array(layer_moves(inherited, moves)?));
            }
            ("anims", Value::Object(anims)) => {
                let mut merged = base
                    .remove("anims")
                    .and_then(|anims| anims.as_object().cloned())
                    .unwrap_or_default();
                for (name, anim) in anims {
                    if anim.is_null() {
                        merged.remove(&name);
                    } else {
                        merged.insert(name, anim);
                    }
                }
                base.insert(key, Value::Object(merged));
            }
            (_, value) => {
                let merged = match base.remove(&key) {
                    Some(old) => deep(old, value),
                    None => value,
                };
                base.insert(key, merged);
            }
        }
    }
    Ok(base)
}

fn deep(base: Value, own: Value) -> Value {
    match (base, own) {
        (Value::Object(mut base), Value::Object(own)) => {
            for (key, value) in own {
                if value.is_null() {
                    base.remove(&key);
                    continue;
                }
                let merged = match base.remove(&key) {
                    Some(old) => deep(old, value),
                    None => value,
                };
                base.insert(key, merged);
            }
            Value::Object(base)
        }
        (_, own) => own,
    }
}

fn move_id(entry: &Value) -> Option<&str> {
    entry
        .as_str()
        .or_else(|| entry.get("id").and_then(Value::as_str))
}

fn layer_moves(inherited: Vec<Value>, own: Vec<Value>) -> Result<Vec<Value>, String> {
    let mut out: Vec<Value> = Vec::with_capacity(inherited.len() + own.len());
    let mut taken: Vec<String> = Vec::new();
    for entry in own {
        let id = move_id(&entry)
            .ok_or("every move needs an `id`")?
            .to_string();
        let mut fields = match entry {
            Value::Object(fields) => fields,
            _ => Map::new(),
        };
        let removed = fields
            .remove("remove")
            .is_some_and(|flag| flag.as_bool() == Some(true));
        let source = fields
            .remove("use")
            .and_then(|name| name.as_str().map(str::to_string));
        taken.push(id.clone());
        if removed {
            continue;
        }
        let wanted = source.as_deref().unwrap_or(&id);
        let template = inherited
            .iter()
            .chain(out.iter())
            .find(|candidate| move_id(candidate) == Some(wanted))
            .and_then(Value::as_object)
            .cloned();
        if template.is_none() && (source.is_some() || fields.is_empty()) {
            return Err(format!(
                "move `{id}` uses `{wanted}`, which no kit or earlier move defines"
            ));
        }
        let mut merged = template.unwrap_or_default();
        for (key, value) in fields {
            if value.is_null() {
                merged.remove(&key);
            } else {
                merged.insert(key, value);
            }
        }
        merged.insert("id".to_string(), Value::String(id));
        out.push(Value::Object(merged));
    }
    for entry in inherited {
        if move_id(&entry).is_some_and(|id| !taken.iter().any(|seen| seen == id)) {
            out.push(entry);
        }
    }
    Ok(out)
}
