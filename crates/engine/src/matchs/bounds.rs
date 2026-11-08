use super::Match;

impl Match {
    pub(super) fn bound_fighters(&mut self) {
        let left = self.camera.x - self.view_w / 2.0; let right = self.camera.x + self.view_w / 2.0;
        let (bl, br, vw) = (self.stage.bound_left, self.stage.bound_right, self.view_w);
        for f in self.fighters.iter_mut() {
            if f.destroyed || (f.is_helper && !f.helper_player) { continue; }
            if !f.screen_bound { f.screen_bound = true; continue; }
            let lw = if f.facing > 0.0 { f.back_width() } else { f.front_width() };
            let rw = if f.facing > 0.0 { f.front_width() } else { f.back_width() };
            let minx = left.max(bl - vw / 2.0) + lw; let maxx = right.min(br + vw / 2.0) - rw;
            f.pos.x = f.pos.x.clamp(minx, maxx.max(minx));
        }
    }
}
