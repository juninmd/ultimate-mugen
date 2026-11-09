use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Text(String),
}

pub const KNOWN_VARIABLES: [&str; 7] = [
    "stats.arcade.clear",
    "consecutive_wins",
    "match_no",
    "mode",
    "netplay",
    "team_size",
    "player",
];

#[derive(Clone, Debug, Default)]
pub struct Context {
    values: HashMap<&'static str, Value>,
}

impl Context {
    pub fn with_int(mut self, name: &'static str, value: i64) -> Self {
        self.values.insert(name, Value::Int(value));
        self
    }

    pub fn with_text(mut self, name: &'static str, value: &str) -> Self {
        self.values.insert(name, Value::Text(value.to_string()));
        self
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }
}
