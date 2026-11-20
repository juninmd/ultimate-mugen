use super::Match;

pub const ARENA_HALF_WIDTH: f32 = 426.0;
pub const ARENA_PLATFORMS: [(f32, f32, f32); 3] = [
    (-310.0, -150.0, -50.0),
    (-78.0, 78.0, -96.0),
    (150.0, 310.0, -50.0),
];

pub(super) fn platform_floor(x: f32, previous_y: f32, vertical_speed: f32) -> f32 {
    if vertical_speed < 0.0 {
        return 0.0;
    }
    ARENA_PLATFORMS
        .into_iter()
        .filter(|&(left, right, top)| x >= left && x <= right && previous_y <= top)
        .map(|(_, _, top)| top)
        .min_by(f32::total_cmp)
        .unwrap_or(0.0)
}

impl Match {
    pub fn enable_power_arena(&mut self) {
        self.power_arena = true;
        self.stage.bound_left = -ARENA_HALF_WIDTH;
        self.stage.bound_right = ARENA_HALF_WIDTH;
        self.zoom_max = (ARENA_HALF_WIDTH * 2.0 + 48.0) / super::BASE_W;
    }

    pub(super) fn power_arena_ring_outs(&mut self) {
        if !self.power_arena || self.round_state != 2 {
            return;
        }
        for fighter in &mut self.fighters {
            if !fighter.is_helper && fighter.alive && fighter.pos.x.abs() > ARENA_HALF_WIDTH {
                fighter.life = 0.0;
                fighter.alive = false;
            }
        }
    }

    pub(super) fn track_power_arena(&mut self, left: f32, right: f32) {
        let span = (right - left).max(0.0);
        let overview = ((span - 160.0) / (ARENA_HALF_WIDTH * 1.4)).clamp(0.0, 1.0);
        let target = 1.0 + (self.zoom_max - 1.0) * overview;
        self.zoom += (target - self.zoom) * 0.15;
        self.apply_zoom();
        let margin = (ARENA_HALF_WIDTH + 24.0 - self.view_w / 2.0).max(0.0);
        let center = ((left + right) * 0.5).clamp(-margin, margin);
        self.camera.x += (center - self.camera.x) * 0.22;
    }
}
