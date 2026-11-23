use super::StoryRun;
use crate::i18n::tr3;

const S_MIN_SCORE: i64 = 30_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rank {
    S,
    A,
    B,
    C,
}

impl Rank {
    pub fn letter(self) -> &'static str {
        match self {
            Rank::S => "S",
            Rank::A => "A",
            Rank::B => "B",
            Rank::C => "C",
        }
    }
}

pub fn of(score: i64, credits_left: u8, credits_start: u8) -> Rank {
    let spent = credits_start.saturating_sub(credits_left);
    match spent {
        0 if score >= S_MIN_SCORE => Rank::S,
        0 | 1 => Rank::A,
        2 => Rank::B,
        _ => Rank::C,
    }
}

pub fn of_run(run: &StoryRun) -> Rank {
    of(run.score, run.credits, run.script().campaign.credits)
}

pub fn line(rank: Rank) -> String {
    format!("{} {}", tr3("NOTA", "RANK", "RANGO"), rank.letter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_continue_and_a_big_score_is_s() {
        assert_eq!(of(40_000, 3, 3), Rank::S);
        assert_eq!(of(10_000, 3, 3), Rank::A);
    }

    #[test]
    fn every_continue_used_lowers_the_rank() {
        assert_eq!(of(99_999, 2, 3), Rank::A);
        assert_eq!(of(99_999, 1, 3), Rank::B);
        assert_eq!(of(99_999, 0, 3), Rank::C);
    }

    #[test]
    fn more_credits_than_the_start_never_underflows() {
        assert_eq!(of(0, 5, 3), Rank::A);
    }
}
