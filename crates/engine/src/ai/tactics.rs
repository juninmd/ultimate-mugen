use super::Ai;
use crate::commands::CommandDef;
use crate::fighter::Fighter;
use crate::input::k;
use crate::matchs::Match;

const MINIMUM_LEVEL: i32 = 3;
const SUPER_COST: f32 = 1000.0;
const ANTI_AIR_RANGE: f32 = 105.0;
const WHIFF_MIN_STATE_TIME: i32 = 6;
const CONFIRM_COOLDOWN: i32 = 24;
const REACTION_COOLDOWN: i32 = 24;
const FINISHER_LIFE_RATIO: f32 = 0.4;
const DESPERATE_LIFE_RATIO: f32 = 0.5;

pub(super) fn is_anti_air_motion(c: &CommandDef) -> bool {
    let raw = c.raw.trim_start_matches('~');
    raw.starts_with("F, D, DF") || raw.starts_with("F,D,DF")
}

fn cost_of(f: &Fighter, c: &CommandDef) -> f32 {
    f.def
        .movelist
        .iter()
        .find(|mv| mv.command == c.name)
        .map_or(0.0, |mv| mv.cost)
}

pub(super) fn affordable(f: &Fighter, e: &Fighter, c: &CommandDef) -> bool {
    let cost = cost_of(f, c);
    if cost > f.power {
        return false;
    }
    let combo_open = e.move_type == 'H';
    if cost >= SUPER_COST {
        return combo_open
            || f.life < f.life_max * DESPERATE_LIFE_RATIO
            || e.life < e.life_max * FINISHER_LIFE_RATIO;
    }
    cost == 0.0 || combo_open || f.power >= cost + SUPER_COST
}

impl Ai {
    pub(super) fn append_seq(
        queue: &mut std::collections::VecDeque<u32>,
        c: &CommandDef,
        facing: f32,
    ) {
        queue.clear();
        for s in &c.steps {
            let d = Self::abs(s.dir, facing);
            if s.release {
                queue.push_back(d | s.btns);
                queue.push_back(d | s.btns);
                for _ in 0..s.charge {
                    queue.push_back(d | s.btns);
                }
                continue;
            }
            if s.btns != 0 {
                let pd = queue.back().map(|x| x & 15).unwrap_or(0);
                queue.push_back(if d != 0 { d } else { pd } | s.btns);
                queue.push_back(0);
            } else {
                queue.push_back(d);
                queue.push_back(d);
            }
        }
    }

    pub(super) fn idle(&mut self, dist: f32, fwd: u32, back: u32) -> u32 {
        if dist > 80.0 {
            self.hold = fwd;
            self.hold_ticks = 6 + (self.r() * 10.0) as i32;
            return fwd;
        }
        if dist < 35.0 && self.r() < 0.1 {
            self.hold = back;
            self.hold_ticks = 8;
            return back;
        }
        if self.r() < 0.03 {
            self.hold = k::D;
            self.hold_ticks = 10;
            return k::D;
        }
        0
    }

    fn choose(
        &mut self,
        m: &Match,
        me: usize,
        ei: usize,
        pool: &[usize],
        keep: impl Fn(&CommandDef) -> bool,
    ) -> Option<usize> {
        let (f, e) = (&m.fighters[me], &m.fighters[ei]);
        let usable: Vec<usize> = pool
            .iter()
            .copied()
            .filter(|&i| keep(&f.def.commands[i]) && affordable(f, e, &f.def.commands[i]))
            .collect();
        if usable.is_empty() {
            return None;
        }
        let pick = (self.r() * usable.len() as f32) as usize % usable.len();
        Some(usable[pick])
    }

    pub(super) fn pick_attack(
        &mut self,
        m: &Match,
        me: usize,
        ei: usize,
        ranged: bool,
    ) -> Option<usize> {
        let pool = if ranged {
            self.ranged_attack.clone()
        } else {
            self.attack.clone()
        };
        self.choose(m, me, ei, &pool, |_| true)
    }

    pub(super) fn reaction(
        &mut self,
        m: &Match,
        me: usize,
        ei: usize,
        near: f32,
        skill: f32,
    ) -> Option<usize> {
        if self.level < MINIMUM_LEVEL || m.round_state != 2 {
            return None;
        }
        let (f, e) = (&m.fighters[me], &m.fighters[ei]);
        let dist = (e.pos.x - f.pos.x).abs();
        let chance = 0.35 + skill * 0.6;
        let roll = self.r();
        let pool = self.attack.clone();
        if f.move_type == 'A' && f.move_hit > 0 && !f.ctrl {
            let finisher_pool: Vec<usize> = pool
                .iter()
                .copied()
                .filter(|&i| f.def.commands[i].steps.len() > 1)
                .collect();
            let picked = if roll < chance {
                self.choose(m, me, ei, &finisher_pool, |_| true)
            } else {
                None
            };
            if picked.is_some() {
                self.cooldown = CONFIRM_COOLDOWN;
            }
            return picked;
        }
        if !f.ctrl || f.state_type == 'A' || roll >= chance {
            return None;
        }
        let picked = if e.state_type == 'A'
            && e.vel.y > 0.0
            && dist < ANTI_AIR_RANGE * (f.def.local_w / 320.0).clamp(0.5, 2.0)
        {
            self.choose(m, me, ei, &pool, is_anti_air_motion)
        } else if e.move_type == 'A'
            && e.move_contact == 0
            && e.state_time > WHIFF_MIN_STATE_TIME
            && dist > near * 0.8
            && dist < near * 2.0
        {
            let ranged = self.ranged_attack.clone();
            self.choose(m, me, ei, &ranged, |_| true)
        } else {
            None
        };
        if picked.is_some() {
            self.cooldown = REACTION_COOLDOWN;
        }
        picked
    }
}

#[cfg(test)]
mod tests;
