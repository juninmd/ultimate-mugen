use crate::storage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) const FILE: &str = "player-card.json";
pub(crate) const SEEKING_COUNT: u8 = 4;
pub(crate) const SETS_KEPT: usize = 10;

pub(crate) fn matches_word(n: u32) -> &'static str {
    use crate::i18n::tr3;
    match (n == 1, crate::i18n::get()) {
        (true, crate::i18n::Lang::En) => "match",
        (false, crate::i18n::Lang::En) => "matches",
        _ => tr3("partidas", "matches", "partidas"),
    }
}
const XP_WIN: u32 = 10;
const XP_LOSS: u32 = 3;

pub(crate) fn seeking_label(i: u8) -> &'static str {
    use crate::i18n::tr3;
    match i % SEEKING_COUNT {
        0 => tr3("Revanche", "Rematch", "Revancha"),
        1 => tr3("Treino", "Training", "Entrenamiento"),
        2 => tr3("Partidas casuais", "Casual matches", "Partidas casuales"),
        _ => tr3("Torneio", "Tournament", "Torneo"),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Card {
    pub name: String,
    pub flag: String,
    pub title: String,
    pub seeking: u8,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            name: String::new(),
            flag: "BR".into(),
            title: String::new(),
            seeking: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SetRec {
    pub won: bool,
    pub fighter: String,
    pub rival: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Record {
    pub card: Card,
    pub matches: u32,
    pub wins: u32,
    pub streak: u32,
    pub best_streak: u32,
    pub xp: u32,
    pub roster: BTreeMap<String, u32>,
    pub rivals: BTreeMap<String, [u32; 2]>,
    pub sets: Vec<SetRec>,
}

impl Record {
    pub(crate) fn parse(text: &str) -> Self {
        serde_json::from_str(text).unwrap_or_default()
    }

    pub(crate) fn load() -> Self {
        storage::read(FILE)
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }

    pub(crate) fn save(&self) {
        if let Ok(text) = serde_json::to_string(self) {
            storage::write(FILE, &text);
        }
    }

    pub(crate) fn apply_set(&mut self, won: bool, fighter: &str, rival: &str) {
        self.matches += 1;
        self.xp += if won { XP_WIN } else { XP_LOSS };
        if won {
            self.wins += 1;
            self.streak += 1;
            self.best_streak = self.best_streak.max(self.streak);
        } else {
            self.streak = 0;
        }
        *self.roster.entry(fighter.to_string()).or_default() += 1;
        self.rivals.entry(rival.to_string()).or_default()[usize::from(!won)] += 1;
        self.sets.insert(
            0,
            SetRec {
                won,
                fighter: fighter.to_string(),
                rival: rival.to_string(),
            },
        );
        self.sets.truncate(SETS_KEPT);
    }

    pub(crate) fn record_set(&mut self, won: bool, fighter: &str, rival: &str) {
        self.apply_set(won, fighter, rival);
        self.save();
    }

    pub(crate) fn win_rate(&self) -> f32 {
        if self.matches == 0 {
            0.0
        } else {
            self.wins as f32 / self.matches as f32
        }
    }

    pub(crate) fn most_played(&self) -> Option<&str> {
        self.roster
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
            .map(|(id, _)| id.as_str())
    }

    pub(crate) fn top_roster(&self, n: usize) -> Vec<(&str, u32)> {
        let mut list: Vec<(&str, u32)> = self.roster.iter().map(|(id, c)| (id.as_str(), *c)).collect();
        list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        list.truncate(n);
        list
    }
}
