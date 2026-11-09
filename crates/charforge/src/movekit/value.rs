use toml::{Table, Value};

pub fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => (*b as i32).to_string(),
        Value::Array(items) => items.iter().map(text).collect::<Vec<_>>().join(", "),
        other => other.to_string(),
    }
}

pub fn get_text(table: &Table, key: &str) -> Option<String> {
    table.get(key).map(text)
}

pub fn get_int(table: &Table, key: &str, default: i64) -> i64 {
    table.get(key).and_then(Value::as_integer).unwrap_or(default)
}

pub fn get_bool(table: &Table, key: &str) -> Option<bool> {
    table.get(key).and_then(Value::as_bool)
}

pub fn get_table<'a>(table: &'a Table, key: &str) -> Option<&'a Table> {
    table.get(key).and_then(Value::as_table)
}

pub fn list<'a>(table: &'a Table, key: &str) -> Vec<&'a Value> {
    match table.get(key) {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(other) => vec![other],
        None => Vec::new(),
    }
}

pub fn tables<'a>(table: &'a Table, key: &str) -> Vec<&'a Table> {
    list(table, key).into_iter().filter_map(Value::as_table).collect()
}

pub fn pair(value: Option<&Value>, default: &str) -> String {
    value.map(text).unwrap_or_else(|| default.to_string())
}
