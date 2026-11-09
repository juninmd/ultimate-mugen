#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn int(value: i64) -> Json {
        Json::Num(value.to_string())
    }

    pub fn float(value: f64) -> Json {
        Json::Num(format!("{value:?}"))
    }

    pub fn str(value: &str) -> Json {
        Json::Str(value.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(items) => items.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Json> {
        match self {
            Json::Obj(items) => items.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: Json) {
        if let Json::Obj(items) = self {
            match items.iter_mut().find(|(k, _)| k == key) {
                Some(entry) => entry.1 = value,
                None => items.push((key.to_string(), value)),
            }
        }
    }

    pub fn remove(&mut self, key: &str) {
        if let Json::Obj(items) = self {
            items.retain(|(k, _)| k != key);
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(text) => text.parse().ok(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Arr(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Json>> {
        match self {
            Json::Arr(items) => Some(items),
            _ => None,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Json::Num(text) if !text.contains(['.', 'e', 'E']))
    }
}
