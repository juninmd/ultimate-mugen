use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use ume_engine::diagnostics::{
    controller_supported, redirection_supported, string_trigger_supported, trigger_supported,
};
use ume_engine::expr::{parse, Node};

#[derive(Debug, Default)]
struct Lists {
    controllers: BTreeSet<String>,
    controllers_missing: BTreeSet<String>,
    triggers: BTreeSet<String>,
    triggers_missing: BTreeSet<String>,
    redirections: BTreeSet<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn backticked(line: &str) -> Option<Vec<String>> {
    let mut names = vec![];
    let mut rest = line;
    let mut residue = String::new();
    while let Some(start) = rest.find('`') {
        residue.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('`')?;
        names.push(after[..end].to_string());
        rest = &after[end + 1..];
    }
    residue.push_str(rest);
    let label_free = match residue.find(':') {
        Some(i) if !names.is_empty() => residue[i + 1..].to_string(),
        _ => residue,
    };
    let only_separators = label_free
        .chars()
        .all(|c| c.is_whitespace() || c == ',' || c == '.');
    (only_separators && !names.is_empty()).then_some(names)
}

fn is_missing_label(line: &str) -> bool {
    let l = line.trim_start().to_ascii_lowercase();
    l.starts_with("missing") || l.starts_with("faltam")
}

fn base_name(token: &str) -> String {
    token
        .split('(')
        .next()
        .unwrap_or(token)
        .trim()
        .to_ascii_lowercase()
}

fn parse_lists(text: &str) -> Lists {
    let mut lists = Lists::default();
    let mut heading = String::new();
    for line in text.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            heading = h.to_ascii_lowercase();
            continue;
        }
        let Some(names) = backticked(line) else {
            continue;
        };
        let missing = is_missing_label(line);
        let names = names
            .iter()
            .map(|n| base_name(n))
            .filter(|n| n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        if heading.contains("controller") {
            if missing {
                lists.controllers_missing.extend(names)
            } else {
                lists.controllers.extend(names)
            }
        } else if heading.contains("trigger") {
            if missing {
                lists.triggers_missing.extend(names)
            } else {
                lists.triggers.extend(names)
            }
        } else if heading.contains("redirec") {
            lists.redirections.extend(names)
        }
    }
    lists
}

fn trigger_handled(name: &str) -> bool {
    let is_special = matches!(parse(&format!("{name} = 1")), Ok(Node::Special { .. }));
    trigger_supported(name)
        || string_trigger_supported(name)
        || is_special
        || matches!(name, "ifelse" | "cond")
}

fn wiki_files() -> [(&'static str, &'static str); 2] {
    [
        ("docs/wiki/CNS-Scripts.md", "en"),
        ("docs/wiki/pt-BR/Scripts-CNS.md", "pt-BR"),
    ]
}

#[test]
fn every_listed_controller_is_implemented() {
    for (path, lang) in wiki_files() {
        let lists = parse_lists(&read(path));
        assert!(
            lists.controllers.len() > 50,
            "{lang}: controller list not found"
        );
        let unsupported: Vec<_> = lists
            .controllers
            .iter()
            .filter(|n| !controller_supported(n))
            .collect();
        assert!(
            unsupported.is_empty(),
            "{path} lists controllers the engine does not handle: {unsupported:?}"
        );
        let implemented: Vec<_> = lists
            .controllers_missing
            .iter()
            .filter(|n| controller_supported(n))
            .collect();
        assert!(
            implemented.is_empty(),
            "{path} marks implemented controllers as missing: {implemented:?}"
        );
    }
}

#[test]
fn every_listed_trigger_is_implemented() {
    for (path, lang) in wiki_files() {
        let lists = parse_lists(&read(path));
        assert!(lists.triggers.len() > 100, "{lang}: trigger list not found");
        let unsupported: Vec<_> = lists
            .triggers
            .iter()
            .filter(|n| !trigger_handled(n))
            .collect();
        assert!(
            unsupported.is_empty(),
            "{path} lists triggers the engine does not handle: {unsupported:?}"
        );
        let implemented: Vec<_> = lists
            .triggers_missing
            .iter()
            .filter(|n| trigger_handled(n))
            .collect();
        assert!(
            implemented.is_empty(),
            "{path} marks implemented triggers as missing: {implemented:?}"
        );
    }
}

#[test]
fn every_listed_redirection_is_implemented() {
    for (path, _) in wiki_files() {
        let lists = parse_lists(&read(path));
        assert!(
            !lists.redirections.is_empty(),
            "{path}: redirection list not found"
        );
        let unsupported: Vec<_> = lists
            .redirections
            .iter()
            .filter(|n| !redirection_supported(n))
            .collect();
        assert!(
            unsupported.is_empty(),
            "{path} lists redirections the engine does not handle: {unsupported:?}"
        );
    }
}

#[test]
fn english_and_portuguese_lists_match() {
    let en = parse_lists(&read("docs/wiki/CNS-Scripts.md"));
    let pt = parse_lists(&read("docs/wiki/pt-BR/Scripts-CNS.md"));
    assert_eq!(en.controllers, pt.controllers);
    assert_eq!(en.controllers_missing, pt.controllers_missing);
    assert_eq!(en.triggers, pt.triggers);
    assert_eq!(en.triggers_missing, pt.triggers_missing);
    assert_eq!(en.redirections, pt.redirections);
}

fn source_literals() -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut pending = vec![repo_root().join("crates/engine/src")];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs")
                || path.file_name().is_some_and(|n| n == "tests.rs")
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            let mut at = 0;
            while let Some(open) = text[at..].find('"').map(|p| p + at) {
                let Some(close) = text[open + 1..].find('"').map(|p| p + open + 1) else {
                    break;
                };
                let piece = &text[open + 1..close];
                let is_name = !piece.is_empty()
                    && piece.len() < 40
                    && piece
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_');
                if is_name {
                    found.insert(piece.to_string());
                    at = close + 1;
                } else {
                    at = open + 1;
                }
            }
        }
    }
    found
}

#[test]
fn every_implemented_controller_and_trigger_is_documented() {
    let lists = parse_lists(&read("docs/wiki/CNS-Scripts.md"));
    let literals = source_literals();
    let undocumented_controllers: Vec<_> = literals
        .iter()
        .filter(|n| controller_supported(n) && !lists.controllers.contains(*n))
        .collect();
    assert!(undocumented_controllers.is_empty(), "implemented controllers missing from docs/wiki/CNS-Scripts.md (and its pt-BR mirror): {undocumented_controllers:?}");
    let undocumented_triggers: Vec<_> = literals
        .iter()
        .filter(|n| {
            trigger_handled(n)
                && !lists.triggers.contains(*n)
                && !INTERNAL_TRIGGERS.contains(&n.as_str())
        })
        .collect();
    assert!(undocumented_triggers.is_empty(), "implemented triggers missing from docs/wiki/CNS-Scripts.md (and its pt-BR mirror): {undocumented_triggers:?}");
}

const INTERNAL_TRIGGERS: &[&str] = &["__str"];

#[test]
#[ignore]
fn print_current_lists() {
    let lists = parse_lists(&read("docs/wiki/CNS-Scripts.md"));
    let literals = source_literals();
    let known: BTreeSet<String> = literals
        .iter()
        .cloned()
        .chain(lists.controllers.iter().cloned())
        .chain(lists.controllers_missing.iter().cloned())
        .collect();
    let controllers: Vec<_> = known.iter().filter(|n| controller_supported(n)).collect();
    let known_triggers: BTreeSet<String> = literals
        .iter()
        .cloned()
        .chain(lists.triggers.iter().cloned())
        .chain(lists.triggers_missing.iter().cloned())
        .collect();
    let triggers: Vec<_> = known_triggers
        .iter()
        .filter(|n| trigger_handled(n))
        .collect();
    let missing_triggers: Vec<_> = lists
        .triggers
        .iter()
        .chain(lists.triggers_missing.iter())
        .filter(|n| !trigger_handled(n))
        .collect();
    println!("controllers: {controllers:?}");
    println!("triggers: {triggers:?}");
    println!("missing triggers: {missing_triggers:?}");
}
