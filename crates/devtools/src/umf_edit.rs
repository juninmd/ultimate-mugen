use crate::json_order::{parse, to_string, Json, Style};
use std::path::Path;

pub struct Document {
    pub data: Json,
    eol: &'static str,
    crlf: bool,
    indented: bool,
}

pub fn load(path: &Path) -> Result<Document, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let tail = &raw[raw.len().saturating_sub(3)..];
    let eol = if tail.contains("\r\n") { "\r\n" } else if raw.ends_with('\n') { "\n" } else { "" };
    let indented = raw.starts_with("{\n") || raw.starts_with("{\r\n");
    Ok(Document { data: parse(&raw)?, eol, crlf: raw.contains("\r\n"), indented })
}

impl Document {
    pub fn render(&self) -> String {
        let style = if self.indented { Style::Indent(2) } else { Style::Compact };
        let mut text = to_string(&self.data, style);
        if self.indented && self.crlf {
            text = text.replace('\n', "\r\n");
        }
        text + self.eol
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        std::fs::write(path, self.render()).map_err(|e| format!("{}: {e}", path.display()))
    }
}
