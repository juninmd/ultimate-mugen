use super::model::{Tuning, NEUTRAL, PARAMS};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use toml::{Table, Value};

const FILE: &str = "char_tuning.toml";

pub type Map = BTreeMap<String, Tuning>;

fn state() -> &'static Mutex<Map> {
    static STATE: OnceLock<Mutex<Map>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(crate::storage::read(FILE).map(|t| parse(&t)).unwrap_or_default()))
}

pub fn parse(text: &str) -> Map {
    let Ok(root) = text.parse::<Table>() else {
        return Map::new();
    };
    root.iter()
        .filter_map(|(key, value)| {
            let table = value.as_table()?;
            let mut t = Tuning::default();
            for (i, p) in PARAMS.iter().enumerate() {
                if let Some(n) = table.get(p.key).and_then(Value::as_integer) {
                    t.0[i] = n.clamp(p.min as i64, p.max as i64) as u16;
                }
            }
            Some((key.clone(), t))
        })
        .collect()
}

pub fn render(map: &Map) -> String {
    let mut root = Table::new();
    for (key, t) in map.iter().filter(|(_, t)| !t.is_default()) {
        let mut table = Table::new();
        for (i, p) in PARAMS.iter().enumerate() {
            if t.0[i] != NEUTRAL {
                table.insert(p.key.into(), Value::Integer(t.0[i] as i64));
            }
        }
        root.insert(key.clone(), Value::Table(table));
    }
    toml::to_string(&root).unwrap_or_default()
}

pub fn get(key: &str) -> Tuning {
    state().lock().ok().and_then(|m| m.get(key).copied()).unwrap_or_default()
}

pub fn set(key: &str, t: Tuning) {
    let Ok(mut m) = state().lock() else {
        return;
    };
    if t.is_default() {
        m.remove(key);
    } else {
        m.insert(key.into(), t);
    }
    crate::storage::write(FILE, &render(&m));
}
