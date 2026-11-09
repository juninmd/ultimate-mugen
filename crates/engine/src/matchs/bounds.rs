use super::Match;

impl Match {
    pub(super) fn bound_fighters(&mut self) {
        let left = self.camera.x - self.view_w / 2.0; let right = self.camera.x + self.view_w / 2.0;
        let (bl, br, vw) = (self.stage.bound_left, self.stage.bound_right, self.view_w);
        let mut bounced = None;
        for i in 0..self.fighters.len() {
            let f = &mut self.fighters[i];
            if f.destroyed || (f.is_helper && !f.helper_player) { continue; }
            if !f.screen_bound { f.screen_bound = true; continue; }
            let lw = if f.facing > 0.0 { f.back_width() } else { f.front_width() };
            let rw = if f.facing > 0.0 { f.front_width() } else { f.back_width() };
            let minx = left.max(bl - vw / 2.0) + lw; let maxx = right.min(br + vw / 2.0) - rw;
            let hit_wall_left = f.pos.x <= minx && f.vel.x <= 0.0;
            let hit_wall_right = f.pos.x >= maxx && f.vel.x >= 0.0;
            f.pos.x = f.pos.x.clamp(minx, maxx.max(minx));
            if (hit_wall_left || hit_wall_right) && f.ghv.wall_bounce && !f.ghv.wall_bounced && (f.move_type == 'H' || f.ghv.fall) {
                f.ghv.wall_bounced = true;
                f.vel.x = if hit_wall_left { 5.2 * f.lscl } else { -5.2 * f.lscl };
                f.vel.y = -6.5 * f.lscl;
                f.state_type = 'A';
                bounced = Some(i);
            }
        }
        if let Some(i) = bounced {
            self.shake_with_dir(10, 12.0, 4.5, 0.0, 0);
            self.sound(i, 5, 0, false, Some("hitheavy"));
        }
    }
}
