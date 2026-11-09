use std::collections::HashMap;

pub struct Args {
    pub positional: Vec<String>,
    flags: HashMap<String, Vec<String>>,
}

const SWITCHES: [&str; 3] = ["attack", "strict", "help"];

impl Args {
    pub fn parse(raw: &[String]) -> Args {
        let mut positional = Vec::new();
        let mut flags: HashMap<String, Vec<String>> = HashMap::new();
        let mut index = 0;
        while index < raw.len() {
            match raw[index].strip_prefix("--") {
                Some(name) => {
                    let entry = flags.entry(name.to_string()).or_default();
                    index += 1;
                    while !SWITCHES.contains(&name) && index < raw.len() && !raw[index].starts_with("--") {
                        entry.push(raw[index].clone());
                        index += 1;
                    }
                }
                None => {
                    positional.push(raw[index].clone());
                    index += 1;
                }
            }
        }
        Args { positional, flags }
    }

    pub fn has(&self, name: &str) -> bool {
        self.flags.contains_key(name)
    }

    pub fn values(&self, name: &str) -> &[String] {
        self.flags.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn value(&self, name: &str) -> Option<&str> {
        self.values(name).first().map(String::as_str)
    }

    pub fn number<T: std::str::FromStr>(&self, name: &str, default: T) -> Result<T, String> {
        match self.value(name) {
            Some(text) => text.parse().map_err(|_| format!("--{name} needs a number, got {text}")),
            None => Ok(default),
        }
    }

    pub fn required(&self, index: usize, what: &str) -> Result<&str, String> {
        self.positional.get(index).map(String::as_str).ok_or_else(|| format!("missing {what}"))
    }
}
