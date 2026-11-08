use crate::fighter::V2;
use crate::matchs::{Event, Match};

pub const DEFAULT_COOLDOWN: i32 = 180;
pub const DEFAULT_CANCEL_WINDOW: i32 = 30;
const KO_SWAP_DELAY: i32 = 50;

impl Match {
    pub fn tag_members(&self, side: usize) -> Vec<usize> {
        (0..self.fighters.len())
            .filter(|&i| {
                let f = &self.fighters[i];
                !f.is_helper && !f.destroyed && f.side.min(1) == side.min(1)
            })
            .collect()
    }

    pub fn tag_active(&self, side: usize) -> Option<usize> {
        self.tag_members(side)
            .into_iter()
            .find(|&i| !self.fighters[i].ikemen.standby)
    }

    pub fn tag_next_alive(&self, from: usize) -> Option<usize> {
        let members = self.tag_members(self.fighters[from].side);
        let at = members.iter().position(|&i| i == from)?;
        (1..members.len())
            .map(|k| members[(at + k) % members.len()])
            .find(|&i| self.fighters[i].life > 0.0)
    }

    pub fn tag_cancel_open(&self, from: usize) -> bool {
        let f = &self.fighters[from];
        f.ctrl
            || (f.move_type == 'A'
                && f.move_contact > 0
                && f.move_contact <= self.tag_cancel_window)
    }

    pub fn tag_ready(&self, from: usize) -> bool {
        let f = &self.fighters[from];
        self.team_tag
            && self.round_state == 2
            && self.pause.is_none()
            && f.alive
            && !f.ikemen.standby
            && self.tag_cooldown[f.side.min(1)] <= 0
            && self.tag_next_alive(from).is_some()
            && self.tag_cancel_open(from)
    }

    pub fn tag_swap(&mut self, side: usize, forced: bool) -> bool {
        let Some(from) = self.tag_active(side) else {
            return false;
        };
        let Some(to) = self.tag_next_alive(from) else {
            return false;
        };
        if !forced && !self.tag_ready(from) {
            return false;
        }
        self.tag_switch(from, to);
        true
    }

    pub fn tag_switch(&mut self, from: usize, to: usize) {
        if from == to || self.fighters[from].side != self.fighters[to].side {
            return;
        }
        let side = self.fighters[from].side.min(1);
        let (x, facing) = (self.fighters[from].pos.x, self.fighters[from].facing);
        {
            let f = &mut self.fighters[from];
            f.ikemen.standby = true;
            f.hitdef_active = false;
            f.ctrl = false;
            f.vel = V2::default();
            f.targets.clear();
            f.bind = None;
        }
        {
            let f = &mut self.fighters[to];
            f.ikemen.standby = false;
            f.pos = V2 { x, y: 0.0 };
            f.vel = V2::default();
            f.facing = facing;
            f.alive = true;
            f.hit_pause = 0;
            f.hit_shake = 0;
        }
        let d = self.fighters[to].def.clone();
        self.change_state(to, 0, d);
        self.fighters[to].ctrl = true;
        self.tag_cooldown[side] = self.tag_cooldown_max;
        self.tag_ko_timer[side] = 0;
        let fighter = self.fighters[to].id;
        self.events.push(Event::Tag { side, fighter });
    }

    pub fn tag_by_controller(&mut self, i: usize, out: bool) {
        let side = self.fighters[i].side;
        if out {
            if self.fighters[i].ikemen.standby {
                return;
            }
            if let Some(to) = self.tag_next_alive(i) {
                self.tag_switch(i, to);
            }
        } else if self.fighters[i].ikemen.standby && self.fighters[i].life > 0.0 {
            if let Some(from) = self.tag_active(side) {
                self.tag_switch(from, i);
            }
        }
    }

    pub(crate) fn reset_tag_round(&mut self) {
        self.tag_cooldown = [0; 2];
        self.tag_ko_timer = [0; 2];
        for side in 0..2 {
            let members = self.tag_members(side);
            let Some(&lead) = members.first() else {
                continue;
            };
            let (pos, facing) = (self.fighters[lead].pos, self.fighters[lead].facing);
            for (n, &i) in members.iter().enumerate() {
                let f = &mut self.fighters[i];
                f.ikemen.standby = n != 0;
                f.pos = pos;
                f.facing = facing;
            }
        }
    }

    pub(crate) fn tag_step(&mut self, requests: &[usize]) {
        for &fi in requests {
            let side = self.fighters[fi].side;
            if self.tag_active(side) == Some(fi) {
                self.tag_swap(side, false);
            }
        }
        if self.round_state == 2 && self.pause.is_none() {
            for c in self.tag_cooldown.iter_mut() {
                *c = (*c - 1).max(0);
            }
        }
        for side in 0..2 {
            let Some(active) = self.tag_active(side) else {
                continue;
            };
            if self.round_state == 2
                && self.fighters[active].life <= 0.0
                && self.tag_next_alive(active).is_some()
            {
                self.tag_ko_timer[side] += 1;
                if self.tag_ko_timer[side] >= KO_SWAP_DELAY {
                    self.tag_swap(side, true);
                }
            } else {
                self.tag_ko_timer[side] = 0;
            }
            let (pos, facing) = (self.fighters[active].pos, self.fighters[active].facing);
            for i in self.tag_members(side) {
                if i == active {
                    continue;
                }
                let f = &mut self.fighters[i];
                f.pos = V2 { x: pos.x, y: 0.0 };
                f.vel = V2::default();
                f.facing = facing;
            }
        }
    }
}
