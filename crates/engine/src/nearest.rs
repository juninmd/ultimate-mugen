use crate::matchs::Match;

impl Match {
    pub fn nearest_foe(&self, i: usize) -> Option<usize> {
        let me = self.fighters.get(i)?;
        let distance = |j: usize| {
            let f = &self.fighters[j];
            let (dx, dy) = (f.pos.x - me.pos.x, f.pos.y - me.pos.y);
            dx * dx + dy * dy
        };
        (0..self.fighters.len())
            .filter(|&j| {
                let f = &self.fighters[j];
                !f.is_helper && f.alive && !f.destroyed && !f.ikemen.standby && f.side != me.side
            })
            .min_by(|&a, &b| distance(a).total_cmp(&distance(b)))
            .or_else(|| self.enemy(i, 0))
    }
}
