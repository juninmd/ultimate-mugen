use super::Record;
use crate::i18n::tr3;

#[derive(Clone, Copy)]
pub(crate) struct Achievement {
    pub title: &'static str,
    pub goal: &'static str,
    pub value: u32,
    pub tiers: [u32; 3],
}

impl Achievement {
    pub(crate) fn level(&self) -> usize {
        self.tiers.iter().filter(|&&t| self.value >= t).count()
    }

    pub(crate) fn next_tier(&self) -> Option<u32> {
        self.tiers.iter().copied().find(|&t| self.value < t)
    }
}

pub(crate) fn achievements(r: &Record) -> [Achievement; 5] {
    [
        Achievement {
            title: tr3("Primeira vitória", "First win", "Primera victoria"),
            goal: tr3("Vença partidas", "Win matches", "Gana partidas"),
            value: r.wins,
            tiers: [1, 10, 50],
        },
        Achievement {
            title: tr3("Veterano", "Veteran", "Veterano"),
            goal: tr3("Dispute partidas", "Play matches", "Juega partidas"),
            value: r.matches,
            tiers: [5, 25, 100],
        },
        Achievement {
            title: tr3("Sequência", "Streak", "Racha"),
            goal: tr3("Vitórias seguidas", "Wins in a row", "Victorias seguidas"),
            value: r.best_streak,
            tiers: [3, 5, 10],
        },
        Achievement {
            title: tr3("Explorador", "Explorer", "Explorador"),
            goal: tr3("Lutadores usados", "Fighters used", "Luchadores usados"),
            value: r.roster.len() as u32,
            tiers: [3, 6, 12],
        },
        Achievement {
            title: tr3("Rival", "Rival", "Rival"),
            goal: tr3("Adversários diferentes", "Different rivals", "Rivales distintos"),
            value: r.rivals.len() as u32,
            tiers: [1, 3, 6],
        },
    ]
}
