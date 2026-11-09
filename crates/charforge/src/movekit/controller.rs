use std::fmt::Display;

pub struct Ctrl {
    lines: Vec<String>,
}

impl Ctrl {
    pub fn new(kind: &str, label: &str) -> Ctrl {
        Ctrl { lines: vec![format!("[State 0, {label}]"), format!("type = {kind}")] }
    }

    pub fn field(mut self, key: &str, value: impl Display) -> Ctrl {
        self.lines.push(format!("{key} = {value}"));
        self
    }

    pub fn opt(self, key: &str, value: Option<impl Display>) -> Ctrl {
        match value {
            Some(v) => self.field(key, v),
            None => self,
        }
    }

    pub fn many(mut self, key: &str, values: &[String]) -> Ctrl {
        self.lines.extend(values.iter().map(|v| format!("{key} = {v}")));
        self
    }

    pub fn finish(self) -> String {
        self.lines.join("\n") + "\n\n"
    }
}
