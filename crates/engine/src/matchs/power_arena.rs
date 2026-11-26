use super::{Fighter, Match, V2, asf};

pub const ARENA_HALF_WIDTH: f32 = 554.0;
pub const ENGINE_STATE_ARENA_EXIT: i32 = 19903;
const ARENA_EXIT_DEPTH: f32 = 184.0;
pub const ARENA_P1_SPAWN_X: f32 = -140.0;
pub const ARENA_P2_SPAWN_X: f32 = 220.0;
pub(super) const ARENA_ROOF_Y: f32 = -414.0;
const CAMERA_HEADROOM: f32 = 134.0;
const CAMERA_FLOOR_MARGIN: f32 = 34.0;
pub const ARENA_PLATFORMS: [(f32, f32, f32); 40] = [
    (-508.0, -368.0, -50.0),
    (-263.0, -121.0, -50.0),
    (-83.0, 47.0, -50.0),
    (157.0, 299.0, -50.0),
    (332.0, 472.0, -50.0),

    (-467.0, -329.0, -102.0),
    (-304.0, -160.0, -102.0),
    (-40.0, 84.0, -102.0),
    (116.0, 260.0, -102.0),
    (373.0, 511.0, -102.0),

    (-509.0, -357.0, -154.0),
    (-264.0, -130.0, -154.0),
    (-82.0, 56.0, -154.0),
    (156.0, 290.0, -154.0),
    (331.0, 483.0, -154.0),

    (-460.0, -328.0, -206.0),
    (-300.0, -172.0, -206.0),
    (-46.0, 98.0, -206.0),
    (120.0, 248.0, -206.0),
    (380.0, 512.0, -206.0),

    (-513.0, -375.0, -258.0),
    (-259.0, -113.0, -258.0),
    (-92.0, 44.0, -258.0),
    (161.0, 307.0, -258.0),
    (327.0, 465.0, -258.0),

    (-480.0, -336.0, -310.0),
    (-284.0, -160.0, -310.0),
    (-59.0, 83.0, -310.0),
    (136.0, 260.0, -310.0),
    (360.0, 504.0, -310.0),

    (-496.0, -364.0, -362.0),
    (-270.0, -130.0, -362.0),
    (-73.0, 53.0, -362.0),
    (150.0, 290.0, -362.0),
    (344.0, 476.0, -362.0),

    (-468.0, -340.0, ARENA_ROOF_Y),
    (-294.0, -158.0, ARENA_ROOF_Y),
    (-45.0, 77.0, ARENA_ROOF_Y),
    (126.0, 262.0, ARENA_ROOF_Y),
    (372.0, 500.0, ARENA_ROOF_Y),
];

pub(crate) fn tile_local(x: f32) -> f32 {
    (x + ARENA_HALF_WIDTH).rem_euclid(ARENA_HALF_WIDTH * 2.0) - ARENA_HALF_WIDTH
}

pub(super) fn platform_floor(x: f32, previous_y: f32, vertical_speed: f32, half: f32) -> f32 {
    if x.abs() > half {
        return ARENA_EXIT_DEPTH;
    }
    if vertical_speed < 0.0 {
        return 0.0;
    }
    let local = tile_local(x);
    ARENA_PLATFORMS
        .into_iter()
        .filter(|&(left, right, top)| local >= left && local <= right && previous_y <= top)
        .map(|(_, _, top)| top)
        .min_by(f32::total_cmp)
        .unwrap_or(0.0)
}

impl Match {
    pub fn enable_power_arena(&mut self) {
        self.power_arena = true;
        self.stage.bound_left = -self.arena_half;
        self.stage.bound_right = self.arena_half;
        self.zoom_max = (ARENA_HALF_WIDTH * 2.0 + 48.0) / super::BASE_W;
        let spawn_offsets = [ARENA_P1_SPAWN_X - self.stage.p1x, ARENA_P2_SPAWN_X - self.stage.p2x];
        self.stage.p1x = ARENA_P1_SPAWN_X;
        self.stage.p2x = ARENA_P2_SPAWN_X;
        for fighter in self.fighters.iter_mut().filter(|fighter| !fighter.is_helper) {
            fighter.pos.x += spawn_offsets[fighter.side.min(1)];
        }
        self.place_elixir();
        self.update_camera();
    }

    pub(super) fn arena_exit_complete(&self, index: usize) -> bool {
        self.power_arena && arena_exit_complete(&self.fighters[index], self.arena_half)
    }

    pub(super) fn arena_exit_pending(&self) -> bool {
        self.power_arena && self.roots().any(|f| f.state_no == ENGINE_STATE_ARENA_EXIT && f.alive)
    }

    pub(super) fn power_arena_ring_outs(&mut self) {
        if !self.power_arena || self.round_state != 2 {
            return;
        }
        let half = self.arena_half;
        for i in 0..self.fighters.len() {
            let f = &mut self.fighters[i];
            if f.is_helper || !f.alive || f.life <= 0.0 {
                continue;
            }
            if f.state_no == ENGINE_STATE_ARENA_EXIT {
                if arena_exit_complete(f, half) {
                    f.life = 0.0;
                    f.alive = false;
                    f.asserts |= asf::INVISIBLE;
                }
                continue;
            }
            if f.pos.x.abs() <= half || f.pos.y < 6.0 {
                continue;
            }
            self.change_engine_state(i, ENGINE_STATE_ARENA_EXIT);
            let f = &mut self.fighters[i];
            f.state_giver = None;
            f.bind = None;
            f.ctrl = false;
            f.asserts = 0;
            f.sprite_transform = None;
            f.hitdef_active = false;
            f.state_type = 'A';
            f.move_type = 'I';
            f.hit_pause = 0;
            f.hit_shake = 0;
            f.unhittable = 120;
            f.vel = V2 { x: (f.vel.x.abs() * 0.7).clamp(2.5, 5.0), y: f.vel.y.clamp(1.0, 3.2) };
            if let Some(animation) = [5050, 5030, 41, 40].into_iter().find(|anim| f.def.anims.contains_key(anim)) {
                f.change_anim_own(animation, 1);
            }
        }
    }

    pub(super) fn track_power_arena(&mut self, left: f32, right: f32) {
        let (left, right) = self.swarm_focus().unwrap_or((left, right));
        let followed = |m: &Match, side: usize| m.swarm.is_none() || side == 0;
        let span = (right - left).max(0.0);
        let overview = ((span - 160.0) / (ARENA_HALF_WIDTH * 1.4)).clamp(0.0, 1.0);
        let target = 1.0 + (self.zoom_max - 1.0) * overview;
        let (highest, lowest) = self.roots()
            .filter(|fighter| !fighter.destroyed && !fighter.ikemen.standby && fighter.move_camera[1] && followed(self, fighter.side))
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(high, low), fighter| {
                let y = if fighter.state_no == ENGINE_STATE_ARENA_EXIT { fighter.pos.y.min(90.0) } else { fighter.pos.y };
                (high.min(y), low.max(y))
            });
        let vertical_zoom = (lowest - highest + CAMERA_HEADROOM + CAMERA_FLOOR_MARGIN) / super::BASE_H;
        let needed = vertical_zoom.max((span + 112.0) / super::BASE_W);
        let target = target.max(needed);
        self.zoom = (self.zoom + (target - self.zoom) * 0.15).max(needed);
        self.apply_zoom();
        let extent = (self.arena_half + 24.0).max(left.abs().max(right.abs()) + 74.0);
        let margin = (extent - self.view_w / 2.0).max(0.0);
        let center = ((left + right) * 0.5).clamp(-margin, margin);
        self.camera.x += (center - self.camera.x) * 0.22;
        let camera_left = right - self.view_w / 2.0 + 56.0;
        let camera_right = left + self.view_w / 2.0 - 56.0;
        self.camera.x = self.camera.x.clamp(camera_left, camera_right.max(camera_left));
        let top = (highest - CAMERA_HEADROOM).min(ARENA_ROOF_Y * overview - 18.0) + self.stage.zoffset;
        let bottom = lowest + self.stage.zoffset - self.view_h + CAMERA_FLOOR_MARGIN;
        let target_y = 0.0_f32.clamp(bottom, top.max(bottom));
        self.camera.y += (target_y - self.camera.y) * 0.22;
        self.camera.y = self.camera.y.clamp(bottom, top.max(bottom));
    }
}

fn arena_exit_complete(f: &Fighter, half: f32) -> bool {
    f.state_no == ENGINE_STATE_ARENA_EXIT
        && f.pos.x.abs() > half + f.front_width().max(f.back_width())
        && f.pos.y >= ARENA_EXIT_DEPTH
}

pub(super) fn step_arena_exit(f: &mut Fighter, half: f32) {
    f.player_push = false;
    if arena_exit_complete(f, half) {
        f.asserts |= asf::INVISIBLE;
        return;
    }
    let outward = f.pos.x.signum();
    f.pos.x += outward * f.vel.x;
    f.pos.y += f.vel.y;
    f.vel.y = (f.vel.y + 0.45).min(14.0);
    let progress = ((f.pos.y - 6.0) / (ARENA_EXIT_DEPTH - 6.0)).clamp(0.0, 1.0);
    f.angle_draw = true;
    f.angle = -outward * f.facing * (20.0 + progress * 300.0);
    let scale = 1.0 - progress * 0.48;
    f.draw_scale = [scale; 2];
}
