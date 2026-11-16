//! Local player card: identity, record, roster and achievements, saved beside the config.
mod achievements;
mod rank;
mod record;
#[cfg(test)]
mod tests;

pub(crate) use achievements::{achievements, Achievement};
pub(crate) use rank::{rank, rank_name};
pub(crate) use record::{matches_word, seeking_label, Card, Record, SEEKING_COUNT};
