use crate::commands::CommandDef;
use crate::input::{k, InputSource};
use crate::matchs::Match;

mod charge;

pub struct Ai {
    pub level: i32,
    queue: std::collections::VecDeque<u32>,
    hold: u32,
    hold_ticks: i32,
    cooldown: i32,
    rng: u32,
    attack: Vec<usize>,
    ranged_attack: Vec<usize>,
    init: bool,
    last_form: u8,
}

impl Ai {
    pub fn new(level: i32, seed: u32) -> Self {
        Ai {
            level,
            queue: Default::default(),
            hold: 0,
            hold_ticks: 0,
            cooldown: 0,
            rng: seed | 1,
            attack: vec![],
            ranged_attack: vec![],
            init: false,
            last_form: u8::MAX,
        }
    }
    pub fn reset(&mut self) {
        self.init = false;
        self.last_form = u8::MAX;
        self.queue.clear();
        self.hold = 0;
        self.hold_ticks = 0;
        self.cooldown = 0;
        self.attack.clear();
        self.ranged_attack.clear();
    }
    fn r(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x as f32 / 4294967296.0
    }
    #[inline]
    fn abs(rel: u32, facing: f32) -> u32 {
        let bf = rel & 12;
        if facing >= 0.0 {
            rel & 15
        } else {
            (rel & 3) | ((bf & 4) << 1) | ((bf & 8) >> 1)
        }
    }
    fn append_seq(queue: &mut std::collections::VecDeque<u32>, c: &CommandDef, facing: f32) {
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
}

impl InputSource for Ai {
    fn poll(&mut self, m: &Match, me: usize) -> u32 {
        if m.is_lane_scrolling() {
            return 0;
        }
        self.poll_fight(m, me) | crate::lane::ai_lane_bits(m, me)
    }
}

impl Ai {
    fn poll_fight(&mut self, m: &Match, me: usize) -> u32 {
        let f = &m.fighters[me];
        if !self.init || self.last_form != f.form {
            self.init = true;
            self.last_form = f.form;
            self.attack.clear();
            self.ranged_attack.clear();
            let mask = 1u32 << (f.form as u32).min(31);
            for (idx, c) in f.def.commands.iter().enumerate() {
                if matches!(c.name.as_str(), "fly_toggle" | "speed_toggle" | "teleport_cmd" | "charge_cmd") {
                    continue;
                }
                let form_ok = f
                    .def
                    .movelist
                    .iter()
                    .find(|mv| mv.command == c.name)
                    .is_none_or(|mv| (mv.forms & mask) != 0);
                if form_ok && f.def.ai_commands.contains(&c.name) && c.steps.iter().any(|s| s.btns != 0) {
                    self.attack.push(idx);
                    if c.steps.len() > 1 {
                        self.ranged_attack.push(idx);
                    }
                }
            }
        }
        if let Some(b) = self.queue.pop_front() {
            return b;
        }
        if m.round_state != 2 {
            return 0;
        }
        let Some(ei) = m.enemy(me, 0) else { return 0 };
        let e = &m.fighters[ei];
        let foe_wins = m.wins[e.side.min(1)];
        let max_form = f.base_def.forms.len().max(1) as u8;
        let want_form = if f.def.boss && foe_wins >= 1 {
            max_form
        } else if f.def.boss && max_form >= 2 && f.life < f.life_max * 0.35 {
            2
        } else if f.life < f.life_max * if f.def.boss { 0.65 } else { 0.45 } {
            1
        } else {
            0
        };
        if f.form < want_form
            && f.ctrl
            && f.next_form_cost().is_some_and(|c| f.power >= c)
            && self.level >= 3
            && f.def.anims.contains_key(&196)
        {
            return k::T;
        }
        if charge::wants_to_charge(m, me, ei, self.level) {
            return k::S;
        }
        if self.hold_ticks > 0 {
            self.hold_ticks -= 1;
            return self.hold;
        }
        let dist = (e.pos.x - f.pos.x).abs();
        let fwd = if f.facing > 0.0 { k::R } else { k::L };
        let back = if f.facing > 0.0 { k::L } else { k::R };
        let boss = f.def.boss;
        let final_boss = boss && f.form >= 2;
        let skill = if final_boss {
            1.35
        } else if boss {
            1.15
        } else {
            self.level as f32 / 8.0
        };
        let r = self.r();
        let threat = e.move_type == 'A' || m.projectiles.iter().any(|p| p.side == e.side);
        if threat
            && dist < 140.0
            && r < (0.25 + skill * 0.6).min(if final_boss { 0.68 } else if boss { 0.55 } else { 1.0 })
            && f.move_type != 'H'
        {
            self.hold = back | if e.state_type == 'C' || self.r() < 0.3 { k::D } else { 0 };
            self.hold_ticks = 8 + (self.r() * 10.0) as i32;
            return self.hold;
        }
        let near = if final_boss { 120.0 } else if boss { 105.0 } else { 70.0 };
        let scale = (f.def.local_w / 320.0).clamp(0.5, 2.0);
        let near = near * scale;
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return self.idle(dist, fwd, back);
        }
        if !f.ctrl {
            return 0;
        }
        if !self.attack.is_empty()
            && (dist < near
                || (dist > 110.0 * scale && self.r() < 0.08 + skill * if final_boss { 0.28 } else if boss { 0.2 } else { 0.06 }))
            && self.r() < 0.2 + skill * 0.5
        {
            let n = if dist < near { self.attack.len() } else { self.ranged_attack.len() };
            if n > 0 {
                let r = self.r();
                let pick = (r * n as f32) as usize % n;
                let cmd_idx = if dist < near { self.attack[pick] } else { self.ranged_attack[pick] };
                let c = &f.def.commands[cmd_idx];
                Self::append_seq(&mut self.queue, c, f.facing);
                self.cooldown = if final_boss {
                    (4.0 + self.r() * 8.0) as i32
                } else if boss {
                    (8.0 + self.r() * 12.0) as i32
                } else {
                    (20.0 + (1.0 - skill) * 40.0 + self.r() * 20.0) as i32
                };
                return self.queue.pop_front().unwrap_or(0);
            }
        }
        if dist > 60.0 && self.r() < 0.02 + skill * 0.02 {
            self.queue.clear();
            self.queue.push_back(k::U | fwd);
            self.queue.push_back(k::U | fwd);
            self.queue.push_back(fwd);
            self.cooldown = 30;
            return self.queue.pop_front().unwrap();
        }
        self.idle(dist, fwd, back)
    }
}

impl Ai {
    fn idle(&mut self, dist: f32, fwd: u32, back: u32) -> u32 {
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
}
