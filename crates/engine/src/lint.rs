use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    DuplicateParameter,
    InvalidValue,
    NegativeStatePersistent,
    StateNumberMismatch,
    GarbageLine,
    DuplicateAction,
    EmptyAction,
    DuplicateCommandKey,
}

impl Rule {
    pub fn label(self) -> &'static str {
        match self {
            Rule::DuplicateParameter => "duplicate parameter",
            Rule::InvalidValue => "invalid value",
            Rule::NegativeStatePersistent => "persistent in negative state",
            Rule::StateNumberMismatch => "state number mismatch",
            Rule::GarbageLine => "garbage line",
            Rule::DuplicateAction => "duplicate action",
            Rule::EmptyAction => "empty action",
            Rule::DuplicateCommandKey => "duplicate command key",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub rule: Rule,
    pub file: String,
    pub line: usize,
    pub message: String,
}

const TRANS_FULL: &[&str] = &["default", "none", "add", "add1", "addalpha", "sub"];
const TRANS_AFTERIMAGE: &[&str] = &["none", "add", "add1", "sub"];
const POSTYPE: &[&str] = &["p1", "p2", "front", "back", "left", "right", "none"];
const SPACE: &[&str] = &["stage", "screen"];

fn enum_values(kind: &str, key: &str) -> Option<&'static [&'static str]> {
    Some(match (kind, key) {
        ("trans", "trans") | ("explod", "trans") | ("modifyexplod", "trans") => TRANS_FULL,
        ("afterimage", "trans") | ("projectile", "afterimage.trans") => TRANS_AFTERIMAGE,
        ("explod", "postype") | ("modifyexplod", "postype") | ("helper", "postype") | ("projectile", "postype") => POSTYPE,
        ("explod", "space") | ("modifyexplod", "space") => SPACE,
        _ => return None,
    })
}

pub fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            ';' if !quoted => return line[..i].trim(),
            _ => {}
        }
    }
    line.trim()
}

pub fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let (mut depth, mut quoted) = (0i32, false);
    for (i, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            '=' if !quoted && depth == 0 => return Some((line[..i].trim(), line[i + 1..].trim())),
            _ => {}
        }
    }
    None
}

fn header_inner(line: &str) -> Option<&str> {
    let s = strip_comment(line);
    (s.len() >= 2 && s.starts_with('[') && s.ends_with(']')).then(|| s[1..s.len() - 1].trim())
}

fn leading_int(s: &str) -> Option<i32> {
    let s = s.trim_start();
    let end = s.char_indices().find(|&(i, c)| !(c.is_ascii_digit() || (i == 0 && (c == '-' || c == '+')))).map_or(s.len(), |(i, _)| i);
    s[..end].parse().ok()
}

fn is_trigger_key(k: &str) -> bool {
    k == "triggerall" || k.strip_prefix("trigger").is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn is_var_target(k: &str) -> bool {
    k == "v" || k == "fv" || ["var(", "fvar(", "sysvar(", "sysfvar("].iter().any(|p| k.starts_with(p))
}

#[derive(Default)]
struct Block {
    header: String,
    kind: String,
    params: Vec<(usize, String, String)>,
}

pub fn lint_cns(file: &str, text: &str) -> Vec<Issue> {
    let mut out = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    let mut statedef: Option<i32> = None;
    let mut push = |rule: Rule, line: usize, message: String| out.push(Issue { rule, file: file.to_string(), line, message });
    for (i, raw) in text.lines().enumerate() {
        let n = i + 1;
        let s = strip_comment(raw);
        if s.is_empty() { continue; }
        if let Some(inner) = header_inner(raw) {
            let lower = inner.to_ascii_lowercase();
            if let Some(rest) = lower.strip_prefix("statedef") {
                statedef = leading_int(rest);
            } else if let Some(rest) = lower.strip_prefix("state") {
                if let (Some(sd), Some(no)) = (statedef, leading_int(rest.trim_start_matches(|c: char| c.is_whitespace()))) {
                    if sd != no { push(Rule::StateNumberMismatch, n, format!("[{inner}] inside Statedef {sd}")); }
                }
            }
            blocks.push(Block { header: inner.to_string(), kind: lower, ..Block::default() });
            continue;
        }
        match split_key_value(s) {
            None => push(Rule::GarbageLine, n, s.to_string()),
            Some((k, v)) => if let Some(b) = blocks.last_mut() { b.params.push((n, k.to_ascii_lowercase(), v.to_string())); },
        }
    }
    let mut statedef: Option<i32> = None;
    for b in &mut blocks {
        let is_statedef = b.kind.starts_with("statedef");
        if is_statedef { statedef = leading_int(&b.kind["statedef".len()..]); }
        if !b.kind.starts_with("state") { continue; }
        let ctrl = b.params.iter().find(|p| p.1 == "type").map(|p| p.2.to_ascii_lowercase()).unwrap_or_default();
        b.kind = if is_statedef { "statedef".into() } else { ctrl };
        let mut seen: HashMap<String, usize> = HashMap::new();
        let var_family = matches!(b.kind.as_str(), "varset" | "varadd" | "parentvarset" | "parentvaradd");
        for (n, k, v) in &b.params {
            if !is_trigger_key(k) {
                let id = if var_family && is_var_target(k) { "\0var".to_string() } else { k.clone() };
                if let Some(first) = seen.get(&id) {
                    out.push(Issue { rule: Rule::DuplicateParameter, file: file.to_string(), line: *n, message: format!("{k} in [{}] repeats line {first}", b.header) });
                } else {
                    seen.insert(id, *n);
                }
            }
            if let Some(allowed) = enum_values(&b.kind, k) {
                if !allowed.contains(&v.to_ascii_lowercase().as_str()) {
                    out.push(Issue { rule: Rule::InvalidValue, file: file.to_string(), line: *n, message: format!("{k} = {v} in [{}]", b.header) });
                }
            }
            if k == "persistent" && !is_statedef && statedef.is_some_and(|s| s < 0) {
                out.push(Issue { rule: Rule::NegativeStatePersistent, file: file.to_string(), line: *n, message: format!("[{}] in state {}", b.header, statedef.unwrap_or(0)) });
            }
        }
    }
    out.sort_by_key(|i| i.line);
    out
}

pub fn lint_cmd(file: &str, text: &str) -> Vec<Issue> {
    let mut out: Vec<Issue> = lint_cns(file, text).into_iter().filter(|i| i.rule != Rule::GarbageLine).collect();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut section = String::new();
    for (i, raw) in text.lines().enumerate() {
        let s = strip_comment(raw);
        if let Some(inner) = header_inner(raw) { section = inner.to_ascii_lowercase(); seen.clear(); continue; }
        if !(section.starts_with("command") || section == "defaults" || section == "remap") { continue; }
        match split_key_value(s) {
            None if !s.is_empty() => out.push(Issue { rule: Rule::GarbageLine, file: file.to_string(), line: i + 1, message: s.to_string() }),
            Some((k, _)) => {
                let k = k.to_ascii_lowercase();
                if let Some(first) = seen.insert(k.clone(), i + 1) {
                    out.push(Issue { rule: Rule::DuplicateCommandKey, file: file.to_string(), line: i + 1, message: format!("{k} in [{section}] repeats line {first}") });
                }
            }
            None => {}
        }
    }
    out.sort_by_key(|i| i.line);
    out
}

pub fn lint_air(file: &str, text: &str) -> Vec<Issue> {
    let mut out = Vec::new();
    let mut seen: HashMap<i32, usize> = HashMap::new();
    let mut current: Option<(i32, usize, bool)> = None;
    let finish = |cur: Option<(i32, usize, bool)>, out: &mut Vec<Issue>| {
        if let Some((no, line, has_frames)) = cur {
            if !has_frames { out.push(Issue { rule: Rule::EmptyAction, file: file.to_string(), line, message: format!("action {no} has no frames") }); }
        }
    };
    for (i, raw) in text.lines().enumerate() {
        let body = raw.split(';').next().unwrap_or("").trim();
        if body.is_empty() { continue; }
        if let Some(inner) = header_inner(raw) {
            finish(current.take(), &mut out);
            let lower = inner.to_ascii_lowercase();
            if let Some(no) = lower.strip_prefix("begin action").and_then(leading_int) {
                if let Some(first) = seen.insert(no, i + 1) {
                    out.push(Issue { rule: Rule::DuplicateAction, file: file.to_string(), line: i + 1, message: format!("action {no} repeats line {first}") });
                }
                current = Some((no, i + 1, false));
            }
            continue;
        }
        if let Some(c) = current.as_mut() {
            if body.starts_with(|ch: char| ch.is_ascii_digit() || ch == '-') { c.2 = true; }
        }
    }
    finish(current.take(), &mut out);
    out
}

pub fn lint_file(file: &str, text: &str) -> Vec<Issue> {
    let lower = file.to_ascii_lowercase();
    if lower.ends_with(".air") { lint_air(file, text) }
    else if lower.ends_with(".cmd") { lint_cmd(file, text) }
    else if lower.ends_with(".cns") { lint_cns(file, text) }
    else { Vec::new() }
}
