use crate::chardef::Controller;
use crate::fighter::asf;
use crate::matchs::Match;

impl Match {
    pub fn destroy_self(&mut self, i: usize, sc: &Controller) {
        if !self.fighters[i].is_helper || self.fighters[i].has_assert(asf::NODESTROYSELF) { return; }
        let recursive = self.num(i, sc, "recursive", 0.0, 0) != 0.0;
        let remove_explods = self.num(i, sc, "removeexplods", 0.0, 0) != 0.0;
        let mut doomed = vec![self.fighters[i].id];
        if recursive {
            let mut grew = true;
            while grew {
                grew = false;
                for f in self.fighters.iter().filter(|f| f.is_helper) {
                    if f.parent.is_some_and(|p| doomed.contains(&p)) && !doomed.contains(&f.id) {
                        doomed.push(f.id);
                        grew = true;
                    }
                }
            }
        }
        for id in doomed.into_iter().rev() {
            if remove_explods { self.explods.retain(|e| e.owner != id); }
            if let Some(idx) = self.idx(id) { self.destroy_helper(idx); }
        }
    }
}
