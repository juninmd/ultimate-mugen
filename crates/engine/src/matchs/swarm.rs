use super::power_arena::{tile_local, ARENA_HALF_WIDTH, ARENA_PLATFORMS};
use super::{Match, V2};
use crate::fighter::GetHitVars;

pub const SWARM_MAX_TOTAL: u32 = 100;
const RESPAWN_DELAY: u32 = 90;
const SPAWN_GRACE: i32 = 40;
const SAFE_DISTANCE: f32 = 220.0;
const TEAM_SPACING: f32 = 44.0;
const FIGHTERS_PER_TILE: f32 = 6.0;
const MAX_TILES: u32 = 7;
const ROUND_TICKS: i32 = 20 * 60 * 60;

#[derive(Clone, Debug)]
pub struct Swarm {
    pub total: u32,
    pub spawned: u32,
    pub kills: u32,
    slots: Vec<Slot>,
}

#[derive(Clone, Debug)]
struct Slot {
    fighter: usize,
    counted: bool,
    cooldown: u32,
}

impl Swarm {
    pub fn remaining(&self) -> u32 {
        self.total - self.kills
    }
}

pub fn arena_tiles(fighters: usize) -> u32 {
    let wanted = (fighters as f32 / FIGHTERS_PER_TILE).ceil().max(1.0) as u32;
    (wanted | 1).min(MAX_TILES)
}

impl Match {
    pub fn enable_swarm(&mut self, total: u32) {
        let foes: Vec<usize> = self.root_indices().into_iter().filter(|&i| self.fighters[i].side == 1).collect();
        let players: Vec<usize> = self.root_indices().into_iter().filter(|&i| self.fighters[i].side == 0).collect();
        if foes.is_empty() || players.is_empty() {
            return;
        }
        let total = total.clamp(foes.len() as u32, SWARM_MAX_TOTAL);
        let tiles = arena_tiles(players.len() + foes.len());
        self.arena_half = ARENA_HALF_WIDTH * tiles as f32;
        self.stage.bound_left = -self.arena_half;
        self.stage.bound_right = self.arena_half;
        self.timer = ROUND_TICKS;
        self.round_ticks = ROUND_TICKS;
        self.swarm = Some(Swarm {
            total,
            spawned: foes.len() as u32,
            kills: 0,
            slots: foes.iter().map(|&fighter| Slot { fighter, counted: false, cooldown: 0 }).collect(),
        });
        let crowd = tiles > 1 || foes.len() > 1 || players.len() > 1;
        for (k, &i) in players.iter().enumerate() {
            let step = (k as f32 / 2.0).ceil() * TEAM_SPACING;
            let x = if k % 2 == 0 { -step } else { step };
            self.fighters[i].pos.x = if crowd { x } else { -112.0 };
            self.fighters[i].facing = 1.0;
        }
        if crowd {
            for &i in &foes {
                let (x, y) = self.spawn_point();
                self.place_foe(i, x, y);
            }
        }
        self.update_camera();
    }

    pub fn swarm_pending(&self) -> bool {
        self.swarm.as_ref().is_some_and(|s| s.spawned < s.total)
    }

    pub(super) fn swarm_focus(&self) -> Option<(f32, f32)> {
        self.swarm.as_ref()?;
        let spread = |only_alive: bool| {
            self.roots()
                .filter(|f| f.side == 0 && !f.destroyed && (f.alive || !only_alive))
                .fold(None, |acc: Option<(f32, f32)>, f| {
                    Some(acc.map_or((f.pos.x, f.pos.x), |(lo, hi)| (lo.min(f.pos.x), hi.max(f.pos.x))))
                })
        };
        spread(true).or_else(|| spread(false))
    }

    pub(super) fn swarm_step(&mut self) {
        if self.swarm.is_none() || self.round_state < 2 {
            return;
        }
        let live = self.round_state == 2;
        let mut respawn = [usize::MAX; 8];
        let mut due = 0;
        if let Some(swarm) = self.swarm.as_mut() {
            for slot in swarm.slots.iter_mut() {
                let down = !self.fighters[slot.fighter].alive || self.fighters[slot.fighter].life <= 0.0;
                if !down {
                    continue;
                }
                if !slot.counted {
                    slot.counted = true;
                    slot.cooldown = RESPAWN_DELAY;
                    swarm.kills += 1;
                } else if !live {
                    continue;
                } else if slot.cooldown > 1 {
                    slot.cooldown -= 1;
                } else if swarm.spawned < swarm.total {
                    swarm.spawned += 1;
                    slot.counted = false;
                    if due < respawn.len() {
                        respawn[due] = slot.fighter;
                        due += 1;
                    }
                }
            }
        }
        for &i in &respawn[..due] {
            let (x, y) = self.spawn_point();
            self.place_foe(i, x, y);
        }
    }

    fn spawn_point(&mut self) -> (f32, f32) {
        let half = self.arena_half - 70.0;
        let mut x = 0.0;
        for _ in 0..8 {
            x = (self.random() * 2.0 - 1.0) * half;
            if self.roots().filter(|f| f.side == 0 && f.alive).all(|f| (f.pos.x - x).abs() >= SAFE_DISTANCE) {
                break;
            }
        }
        let local = tile_local(x);
        let fits = |&&(l, r, _): &&(f32, f32, f32)| local >= l && local <= r;
        let choices = 1 + ARENA_PLATFORMS.iter().filter(fits).count();
        let pick = ((self.random() * choices as f32) as usize).min(choices - 1);
        let top = if pick == 0 { 0.0 } else { ARENA_PLATFORMS.iter().filter(fits).nth(pick - 1).map_or(0.0, |p| p.2) };
        (x, top)
    }

    fn place_foe(&mut self, i: usize, x: f32, y: f32) {
        self.elixir_release(i);
        let toward = self.roots().filter(|f| f.side == 0 && f.alive).map(|f| f.pos.x).min_by(|a, b| (a - x).abs().total_cmp(&(b - x).abs())).unwrap_or(0.0);
        let in_play = self.round_state == 2;
        {
            let f = &mut self.fighters[i];
            f.destroyed = false;
            f.pos = V2 { x, y };
            f.vel = V2::default();
            f.ground_level = y;
            f.facing = if toward >= x { 1.0 } else { -1.0 };
            f.life = f.life_max;
            f.alive = true;
            f.revert_form();
            f.reset_passive_for_round();
            f.reset_potential();
            f.hit_pause = 0;
            f.hit_shake = 0;
            f.targets.clear();
            f.bind = None;
            f.hitdef_active = false;
            f.asserts = 0;
            f.angle_draw = false;
            f.draw_scale = [1.0, 1.0];
            f.sprite_transform = None;
            f.move_type = 'I';
            f.state_type = 'S';
            f.unhittable = SPAWN_GRACE;
            f.st_def = f.def.clone();
            f.ghv = GetHitVars::default();
            f.sysvars = [0; 5];
            f.ctrl = in_play;
        }
        crate::ikemen::life::reset_for_round(self, i);
        let def = self.fighters[i].def.clone();
        self.change_state(i, 0, def);
    }
}
