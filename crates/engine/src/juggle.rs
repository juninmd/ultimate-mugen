use crate::fighter::Fighter;
use crate::matchs::Match;

impl Fighter {
    pub fn juggle_spent(&self, source: u32) -> i32 {
        self.juggle_ledger.iter().find(|(s, _)| *s == source).map_or(0, |(_, p)| *p)
    }
    pub fn add_juggle(&mut self, source: u32, points: i32) {
        match self.juggle_ledger.iter_mut().find(|(s, _)| *s == source) {
            Some(entry) => entry.1 += points,
            None => self.juggle_ledger.push((source, points)),
        }
    }
}

impl Match {
    /// The fighter whose juggle ledger an attack charges: helpers with `inheritjuggle` 1/2 use their parent/root.
    pub fn juggle_source(&self, a: usize) -> u32 {
        let f = &self.fighters[a];
        if !f.is_helper { return f.id; }
        match f.inherit_juggle {
            1 => f.parent.unwrap_or(f.id),
            2 => f.root,
            _ => f.id,
        }
    }
}
