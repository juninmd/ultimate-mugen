use ume_engine::fighter::V2;
use ume_engine::matchs::Match;

pub struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
}

impl Fnv {
    pub fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.0 ^= x as u64;
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }
    pub fn u32(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn i32(&mut self, v: i32) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn i64(&mut self, v: i64) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn f32(&mut self, v: f32) {
        self.u32(v.to_bits());
    }
    pub fn f64(&mut self, v: f64) {
        self.bytes(&v.to_bits().to_le_bytes());
    }
    pub fn v2(&mut self, v: V2) {
        self.f32(v.x);
        self.f32(v.y);
    }
    pub fn finish(&self) -> u64 {
        self.0
    }
}

pub fn checksum(m: &Match) -> u128 {
    let mut h = Fnv::default();
    h.i64(m.game_time);
    for v in [m.round_state, m.round_no, m.round_time, m.timer, m.wins[0], m.wins[1], m.winner_side, m.slow_mo, m.shake_time] {
        h.i32(v);
    }
    h.u32(m.rng);
    h.u32(m.next_id);
    h.u32(m.global_asserts);
    h.u32(m.match_over as u32);
    h.v2(m.camera);
    h.f32(m.zoom);
    h.i32(m.pause.as_ref().map_or(-1, |p| p.time));
    for t in &m.texts {
        h.u32(t.owner);
        h.i32(t.id);
        h.i32(t.time);
        h.v2(t.pos);
        h.bytes(t.text.as_bytes());
    }
    for f in &m.fighters {
        h.u32(f.id);
        h.u32(f.destroyed as u32);
        h.v2(f.pos);
        h.v2(f.vel);
        h.f32(f.facing);
        for v in [f.state_no, f.prev_state_no, f.state_time, f.anim_no, f.anim_real, f.elem as i32, f.elem_time, f.anim_elapsed] {
            h.i32(v);
        }
        for v in [f.hit_pause, f.hit_shake, f.hit_over_time, f.slide_time, f.fall_time, f.juggle_used, f.move_contact, f.move_hit, f.hit_count, f.uniq_hit_count, f.combo_count] {
            h.i32(v);
        }
        h.bytes(&[f.state_type as u8, f.move_type as u8, f.physics as u8, f.ctrl as u8, f.alive as u8, f.hitdef_active as u8]);
        h.f32(f.life);
        h.u32(f.ikemen.standby as u32);
        h.f32(f.power);
        for v in f.vars {
            h.i32(v);
        }
        for v in f.fvars {
            h.f64(v);
        }
        for v in f.sysvars {
            h.i32(v);
        }
        h.u32(f.asserts);
        h.i64(f.history.tick);
    }
    for e in &m.explods {
        h.u32(e.owner);
        h.i32(e.id);
        h.v2(e.pos);
        h.i32(e.time);
        h.i32(e.cur.anim);
    }
    for p in &m.projectiles {
        h.u32(p.owner);
        h.i32(p.id);
        h.v2(p.pos);
        h.i32(p.hits);
        h.i32(p.time);
    }
    h.finish() as u128
}
