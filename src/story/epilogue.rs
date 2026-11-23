use super::epilogue_store::table;
use super::stages::STAGES;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use ume_native::LocalText;

#[path = "epilogue_tale.rs"]
mod epilogue_tale;

pub(super) const BUNDLED_TEXT: &str = include_str!("../../assets/story/epilogues.json");
pub const FILE: &str = "story-epilogues.json";
pub const FALLBACK: &str = "*";
pub const ROOFTOP_BEATS: usize = 2;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Epilogue {
    #[serde(default)]
    pub scene: String,
    #[serde(default)]
    pub title: LocalText,
    #[serde(default)]
    pub boss: LocalText,
    #[serde(default)]
    pub reply: LocalText,
    #[serde(default)]
    pub pages: Vec<LocalText>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    Boss,
    Hero,
    Narrator,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Beat {
    pub speaker: Speaker,
    pub text: String,
    pub home: bool,
}

pub type Table = BTreeMap<String, Epilogue>;

impl Epilogue {
    pub fn beats(&self, lang: &str) -> Vec<Beat> {
        let spoken = [(Speaker::Boss, &self.boss), (Speaker::Hero, &self.reply)];
        let rooftop = spoken.into_iter().map(|(speaker, text)| Beat {
            speaker,
            text: text.pick(lang).to_string(),
            home: false,
        });
        let home = self.pages.iter().map(|text| Beat {
            speaker: Speaker::Narrator,
            text: text.pick(lang).to_string(),
            home: true,
        });
        rooftop
            .chain(home)
            .filter(|beat| !beat.text.is_empty())
            .collect()
    }

    pub fn page_count(&self, lang: &str) -> usize {
        self.beats(lang).len() + 1
    }
}

pub fn parse(text: &str) -> Option<Table> {
    serde_json::from_str(text).ok()
}

pub fn merged(custom: Option<Table>) -> Table {
    let mut table = parse(BUNDLED_TEXT).unwrap_or_default();
    table.extend(custom.unwrap_or_default());
    table
}

pub fn pick<'a>(table: &'a Table, fighter_id: &str) -> Option<&'a Epilogue> {
    let id = fighter_id.strip_prefix("umf:").unwrap_or(fighter_id);
    table.get(id).or_else(|| table.get(FALLBACK))
}

pub fn of(fighter_id: &str) -> Option<&'static Epilogue> {
    pick(table(), fighter_id)
}

pub fn of_with(fighter_id: &str, mercy: Option<bool>) -> Option<&'static Epilogue> {
    if mercy == Some(true) {
        let id = fighter_id.strip_prefix("umf:").unwrap_or(fighter_id);
        let table = table();
        if let Some(script) = table
            .get(&format!("{id}:mercy"))
            .or_else(|| table.get(&format!("{FALLBACK}:mercy")))
        {
            return Some(script);
        }
    }
    of(fighter_id)
}

pub fn final_boss(player_id: &str) -> &'static str {
    let (boss, alt_boss): (&'static str, &'static str) = match super::script::active().stages.last() {
        Some(last) => (&last.boss, &last.alt_boss),
        None => {
            let last = &STAGES[STAGES.len() - 1];
            (last.boss, last.alt_boss)
        }
    };
    if boss == player_id { alt_boss } else { boss }
}

pub fn pick_named<'a>(table: &'a Table, fighter_id: &str, name: &str) -> Option<Cow<'a, Epilogue>> {
    let id = fighter_id.strip_prefix("umf:").unwrap_or(fighter_id);
    match table.get(id) {
        Some(own) => Some(Cow::Borrowed(own)),
        None => epilogue_tale::tale_for(table, id, name)
            .map(Cow::Owned)
            .or_else(|| table.get(FALLBACK).map(Cow::Borrowed)),
    }
}

pub fn of_named(fighter_id: &str, name: &str, mercy: Option<bool>) -> Option<Cow<'static, Epilogue>> {
    if mercy == Some(true) {
        return of_with(fighter_id, mercy).map(Cow::Borrowed);
    }
    pick_named(table(), fighter_id, name)
}

#[cfg(test)]
#[path = "epilogue_tests.rs"]
mod tests;
