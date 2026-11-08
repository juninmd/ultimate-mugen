//! Generic MUGEN-style INI parser (keeps duplicate keys & order; used by
//! DEF/CNS/CMD/AIR/stage files). Port of `src/mugen/ini.ts`.

pub(crate) use crate::num::parse_i32;
pub use crate::num::{parse_float, parse_int};

#[derive(Clone, Debug, Default)]
pub struct Section {
    /// Original case, trimmed.
    pub name: String,
    /// Lowercase name.
    pub lname: String,
    /// `(lowercase key, trimmed raw value)` in file order (duplicates kept).
    pub entries: Vec<(String, String)>,
    /// Every non-empty, comment-stripped, trimmed line of the section
    /// (including `key = value` lines; AIR frames etc. live here).
    pub lines: Vec<String>,
}

impl Section {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub fn get_all<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.entries
            .iter()
            .filter(move |(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }
}

/// Remove a `;` comment (ignoring `;` inside double quotes).
pub fn strip_comment(line: &str) -> &str {
    if !line.contains(';') {
        return line;
    }
    let mut in_q = false;
    for (i, c) in line.bytes().enumerate() {
        if c == b'"' {
            in_q = !in_q;
        } else if c == b';' && !in_q {
            return &line[..i];
        }
    }
    line
}

/// Split text into lines on `\r\n`, `\n` or `\r` (empty lines may appear
/// in place of `\r\n`; every consumer skips empty lines). Strips a leading BOM.
pub(crate) fn split_lines(text: &str) -> impl Iterator<Item = &str> {
    text.strip_prefix('\u{feff}')
        .unwrap_or(text)
        .split(['\n', '\r'])
}

/// True for JS `/^-?\d/`.
pub(crate) fn starts_with_number(line: &str) -> bool {
    let b = line.as_bytes();
    match b.first() {
        Some(b'-') => b.get(1).is_some_and(|c| c.is_ascii_digit()),
        Some(c) => c.is_ascii_digit(),
        None => false,
    }
}

pub fn parse_ini(text: &str) -> Vec<Section> {
    parse_ini_opt(text, true)
}

pub fn parse_ini_no_lines(text: &str) -> Vec<Section> {
    parse_ini_opt(text, false)
}

pub fn parse_ini_opt(text: &str, keep_lines: bool) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::with_capacity(text.len() / 256 + 4);
    for raw in split_lines(text) {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let name = match rest.find(']') {
                Some(e) => &rest[..e],
                None => rest,
            }
            .trim();
            out.push(Section {
                name: name.to_string(),
                lname: name.to_ascii_lowercase(),
                entries: Vec::with_capacity(8),
                lines: Vec::new(),
            });
            continue;
        }
        if out.is_empty() {
            out.push(Section::default());
        }
        let cur = out.last_mut().unwrap();
        if let Some(eq) = line.find('=') {
            if eq > 0 && !starts_with_number(line) {
                cur.entries.push((
                    line[..eq].trim().to_ascii_lowercase(),
                    line[eq + 1..].trim().to_string(),
                ));
            }
        }
        if keep_lines {
            cur.lines.push(line.to_string());
        }
    }
    out
}

/// First section whose name equals `name` (case-insensitive).
pub fn find_section<'a>(secs: &'a [Section], name: &str) -> Option<&'a Section> {
    secs.iter().find(|s| s.lname.eq_ignore_ascii_case(name))
}

/// Trim and remove surrounding double quotes (text up to the closing quote).
pub fn unquote(v: &str) -> String {
    unquote_str(v).to_string()
}

pub fn unquote_str(v: &str) -> &str {
    let v = v.trim();
    if let Some(rest) = v.strip_prefix('"') {
        match rest.find('"') {
            Some(e) => &rest[..e],
            None => rest,
        }
    } else {
        v
    }
}

/// Comma separated numbers (JS `parseFloat` semantics per item). Missing
/// value -> `def`; unparsable items -> `def[i]` or 0.
pub fn nums(v: Option<&str>, def: &[f32]) -> Vec<f32> {
    let Some(v) = v else { return def.to_vec() };
    if v.trim().is_empty() {
        return def.to_vec();
    }
    v.split(',')
        .enumerate()
        .map(|(i, p)| match parse_float(p) {
            Some(n) if n.is_finite() => n as f32,
            _ => def.get(i).copied().unwrap_or(0.0),
        })
        .collect()
}

/// Split on top-level commas (respecting parens/brackets/quotes); items trimmed.
pub fn split_top(v: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut depth, mut in_q, mut start) = (0i32, false, 0usize);
    for (i, c) in v.bytes().enumerate() {
        if c == b'"' {
            in_q = !in_q;
        } else if in_q {
            continue;
        } else if c == b'(' || c == b'[' {
            depth += 1;
        } else if c == b')' || c == b']' {
            depth = (depth - 1).max(0);
        } else if c == b',' && depth == 0 {
            out.push(v[start..i].trim().to_string());
            start = i + 1;
        }
    }
    out.push(v[start..].trim().to_string());
    out
}
