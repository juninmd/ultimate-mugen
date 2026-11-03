//! CPU opponent: works with any character (native or MUGEN) by replaying command definitions.
use crate::commands::CommandDef;
use crate::input::{k, InputSource};
use crate::matchs::Match;

pub struct Ai { pub level: i32, queue: std::collections::VecDeque<u32>, hold: u32, hold_ticks: i32, cooldown: i32, rng: u32, attack: Vec<CommandDef>, init: bool }

impl Ai {
    pub fn new(level: i32, seed: u32) -> Self { Ai { level, queue: Default::default(), hold: 0, hold_ticks: 0, cooldown: 0, rng: seed | 1, attack: vec![], init: false } }
    /// Forget cached per-character data (when a CPU pilot is handed another fighter, e.g. control switching in team battles).
    pub fn reset(&mut self) { self.init = false; self.queue.clear(); self.hold = 0; self.hold_ticks = 0; self.cooldown = 0; self.attack.clear(); }
    fn r(&mut self) -> f32 { let mut x = self.rng; x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rng = x; x as f32 / 4294967296.0 }
    fn abs(rel: u32, facing: f32) -> u32 {
        let mut b = rel & 3;
        let (back, fwd) = (rel & 4 != 0, rel & 8 != 0);
        if facing >= 0.0 { if back { b |= k::L } if fwd { b |= k::R } } else { if back { b |= k::R } if fwd { b |= k::L } }
        b
    }
    fn seq(c: &CommandDef, facing: f32) -> Vec<u32> {
        let mut out = vec![];
        for s in &c.steps {
            let d = Self::abs(s.dir, facing);
            if s.release {
                out.push(d | s.btns); out.push(d | s.btns);
                for _ in 0..s.charge { out.push(d | s.btns); }
                continue;
            }
            if s.btns != 0 { let pd = out.last().map(|x| x & 15).unwrap_or(0); out.push(if d != 0 { d } else { pd } | s.btns); out.push(0); }
            else { out.push(d); out.push(d); }
        }
        out
    }
}

impl InputSource for Ai {
    fn poll(&mut self, m: &Match, me: usize) -> u32 {
        let f = &m.fighters[me];
        if !self.init {
            self.init = true;
            self.attack = f.def.commands.iter().filter(|c| f.def.ai_commands.contains(&c.name) && c.steps.iter().any(|s| s.btns != 0)).cloned().collect();
        }
        if let Some(b) = self.queue.pop_front() { return b; }
        if m.round_state != 2 { return 0; }
        let Some(ei) = m.enemy(me, 0) else { return 0 };
        let e = &m.fighters[ei];
        if self.hold_ticks > 0 { self.hold_ticks -= 1; return self.hold; }
        let dist = (e.pos.x - f.pos.x).abs();
        let fwd = if f.facing > 0.0 { k::R } else { k::L };
        let back = if f.facing > 0.0 { k::L } else { k::R };
        let skill = self.level as f32 / 8.0;
        let r = self.r();
        let threat = e.move_type == 'A' || m.projectiles.iter().any(|p| p.side == e.side);
        if threat && dist < 140.0 && r < 0.25 + skill * 0.6 && f.move_type != 'H' {
            self.hold = back | if e.state_type == 'C' || self.r() < 0.3 { k::D } else { 0 };
            self.hold_ticks = 8 + (self.r() * 10.0) as i32;
            return self.hold;
        }
        if self.cooldown > 0 { self.cooldown -= 1; return self.idle(dist, fwd, back); }
        if !f.ctrl { return 0; }
        if !self.attack.is_empty() && (dist < 70.0 || (dist > 110.0 && self.r() < 0.08 + skill * 0.06)) && self.r() < 0.2 + skill * 0.5 {
            let pool: Vec<usize> = (0..self.attack.len()).filter(|&i| dist < 70.0 || self.attack[i].steps.len() > 1).collect();
            if !pool.is_empty() {
                let pick = (self.r() * pool.len() as f32) as usize % pool.len();
                let c = &self.attack[pool[pick]];
                self.queue = Self::seq(c, f.facing).into();
                self.cooldown = (20.0 + (1.0 - skill) * 40.0 + self.r() * 20.0) as i32;
                return self.queue.pop_front().unwrap_or(0);
            }
        }
        if dist > 60.0 && self.r() < 0.02 + skill * 0.02 { self.queue = vec![k::U | fwd, k::U | fwd, fwd].into(); self.cooldown = 30; return self.queue.pop_front().unwrap(); }
        self.idle(dist, fwd, back)
    }
}
impl Ai {
    fn idle(&mut self, dist: f32, fwd: u32, back: u32) -> u32 {
        if dist > 80.0 { self.hold = fwd; self.hold_ticks = 6 + (self.r() * 10.0) as i32; return fwd; }
        if dist < 35.0 && self.r() < 0.1 { self.hold = back; self.hold_ticks = 8; return back; }
        if self.r() < 0.03 { self.hold = k::D; self.hold_ticks = 10; return k::D; }
        0
    }
}
