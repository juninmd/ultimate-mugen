//! UI localization: keyed catalogs (`en` default, `pt-BR`, `es`) plus inline `tr`/`tr3` helpers.
//! Lookup order is current locale -> English -> the key itself; a missing key is logged once.
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

mod catalog;
mod es_catalog;
mod es_phrases;
pub mod system;
#[cfg(test)]
mod tests;

use catalog::{EN, PT};
use es_catalog::ES;
use es_phrases::ES_PHRASES;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Pt,
    En,
    Es,
}

pub const LANGS: [(Lang, &str); 3] = [
    (Lang::En, "English"),
    (Lang::Pt, "Português (Brasil)"),
    (Lang::Es, "Español"),
];

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Pt => "pt",
            Lang::Es => "es",
        }
    }
    pub fn from_code(code: &str) -> Lang {
        match code {
            "en" => Lang::En,
            "es" => Lang::Es,
            _ => Lang::Pt,
        }
    }
    pub fn next(self, step: i32) -> Lang {
        let at = LANGS.iter().position(|(l, _)| *l == self).unwrap_or(0) as i32;
        LANGS[(at + step).rem_euclid(LANGS.len() as i32) as usize].0
    }
    fn slot(self) -> u8 {
        match self {
            Lang::Pt => 0,
            Lang::En => 1,
            Lang::Es => 2,
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(1);
static MISSING: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());

pub fn set(l: Lang) {
    CURRENT.store(l.slot(), Ordering::Relaxed);
}
pub fn get() -> Lang {
    match CURRENT.load(Ordering::Relaxed) {
        0 => Lang::Pt,
        2 => Lang::Es,
        _ => Lang::En,
    }
}
pub fn default_lang() -> Lang {
    Lang::En
}

fn spanish_of(en: &str) -> Option<&'static str> {
    ES_PHRASES.iter().find(|(k, _)| *k == en).map(|(_, v)| *v)
}

pub fn tr<'a>(pt: &'a str, en: &'a str) -> &'a str {
    match get() {
        Lang::En => en,
        Lang::Pt => pt,
        Lang::Es => spanish_of(en).unwrap_or(en),
    }
}

pub fn tr3<'a>(pt: &'a str, en: &'a str, es: &'a str) -> &'a str {
    match get() {
        Lang::En => en,
        Lang::Pt => pt,
        Lang::Es => es,
    }
}

pub fn fill(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (k, v) in pairs {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

fn find(table: &'static [(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

pub fn t(key: &'static str) -> &'static str {
    let table = match get() {
        Lang::En => EN,
        Lang::Pt => PT,
        Lang::Es => ES,
    };
    if let Some(v) = find(table, key).or_else(|| find(EN, key)) {
        return v;
    }
    if let Some(v) = crate::story::text::lookup(key) {
        return v;
    }
    let mut missing = MISSING.lock().unwrap_or_else(|e| e.into_inner());
    if missing.insert(key) {
        eprintln!("i18n: missing key '{key}'");
    }
    key
}

pub fn tf(key: &'static str, pairs: &[(&str, &str)]) -> String {
    let mut out = t(key).to_string();
    for (k, v) in pairs {
        out = out.replace(k, v);
    }
    out
}

pub fn keys_without_english() -> Vec<&'static str> {
    PT.iter()
        .map(|(k, _)| *k)
        .filter(|k| !EN.iter().any(|(e, _)| e == k))
        .collect()
}

pub fn missing_logged() -> Vec<&'static str> {
    MISSING
        .lock()
        .map(|m| m.iter().copied().collect())
        .unwrap_or_default()
}
