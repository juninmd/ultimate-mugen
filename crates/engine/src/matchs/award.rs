use super::Match;

impl Match {
    pub fn award_round(&mut self) {
        if self.winner_side >= 0 {
            self.wins[self.winner_side as usize] += 1;
        } else if self.winner_side == -2 {
            let cap = (self.rounds_to_win - 1).max(0);
            for w in self.wins.iter_mut() {
                *w = (*w + 1).min(cap);
            }
        }
    }
}
