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
        let v = if v == 0.0 { 0.0 } else { v };
        self.u32(v.to_bits());
    }
    pub fn f64(&mut self, v: f64) {
        let v = if v == 0.0 { 0.0 } else { v };
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
    for v in [m.tag_cooldown[0], m.tag_cooldown[1], m.tag_ko_timer[0], m.tag_ko_timer[1], m.team_member[0] as i32, m.team_member[1] as i32, m.round_ticks] {
        h.i32(v);
    }
    h.i32(m.first_attack.map_or(-1, |s| s as i32));
    h.i32(m.intro_ticks);
    h.i32(m.rounds_to_win);
    h.u32(m.corner_pushes);
    h.u32(m.won_by_ko as u32); h.u32(m.power_arena as u32); h.u32(m.elixir.as_ref().map_or(0, |e| 1 + e.taken() as u32)); h.u32(m.swarm.as_ref().map_or(0, |s| s.kills ^ (s.spawned << 8)));
    h.i32(m.finish_window);
    h.u32(m.rng);
    h.u32(m.rand_state.get());
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
        for v in [f.hit_pause, f.hit_shake, f.hit_over_time, f.slide_time, f.fall_time, f.juggle_ledger.iter().map(|e| e.1).sum::<i32>(), f.move_contact, f.move_hit, f.move_guarded, f.move_reversed, f.hit_count, f.uniq_hit_count, f.combo_count] {
            h.i32(v);
        }
        for v in [f.pause_move_time, f.super_move_time, f.air_jumps, f.form_time] {
            h.i32(v);
        }
        h.u32(f.form as u32);
        h.i32(f.passive.stacks);
        h.i32(f.passive.idle_ticks);
        h.i32(f.passive.fight_ticks);
        h.i32(f.rush.overdrive);
        h.i32(f.rush.cooldown);
        h.i32(f.rush.cancels);
        h.i32(f.rush.guard_hold);
        h.i32(f.rush.perfect);
        for v in [f.ghv.kind, f.ghv.hittime, f.ghv.slidetime, f.ghv.ctrltime] {
            h.i32(v);
        }
        h.f32(f.ghv.xvel);
        h.f32(f.ghv.yvel);
        h.f32(f.ghv.damage);
        h.u32(f.ghv.fall as u32);
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
        for v in f.sysfvars {
            h.f64(v);
        }
        h.u32(f.asserts as u32);
        h.u32((f.asserts >> 32) as u32);
        h.i64(f.history.tick);
        h.i64(set_digest(f.cmd.pause_hold_entries()) as i64);
        h.i64(set_digest(f.cmd.active_names().map(|n| (n, 0))) as i64);
        h.u32(f.hitdef_targets.len() as u32);
        for t in &f.hitdef_targets {
            h.u32(*t);
        }
        h.u32(f.targets.len() as u32);
        for (id, v) in &f.targets {
            h.u32(*id);
            h.i32(*v);
        }
        for slot in f.hit_by.iter().chain(f.not_hit_by.iter()) {
            match slot {
                Some((name, v)) => {
                    h.u32(1);
                    h.bytes(name.as_bytes());
                    h.i32(*v);
                }
                None => h.u32(0),
            }
        }
        for (id, v) in &f.persist {
            h.u32(*id);
            h.i32(*v);
        }
        for v in [f.combo_timer, f.combo_count, f.received_hits, f.air_dashes, f.unhittable, f.no_ctrl_ticks, f.move_countered, f.run_state_depth as i32] {
            h.i32(v);
        }
        for v in [f.combo_damage, f.received_damage, f.ground_level, f.angle, f.super_defence_mul] {
            h.f32(v);
        }
        h.f64(f.score);
        h.f64(f.score_total);
        h.i64(f.hit_overridden_at);
        h.u32(f.last_hit_by.unwrap_or(u32::MAX));
        h.u32(f.combo_target.unwrap_or(u32::MAX));
        h.i32(f.victory_quote);
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
    for b in &m.objects.breakables {
        h.f32(b.life);
        h.i32(b.rehit);
        h.u32(b.broken as u32);
    }
    for p in &m.objects.pickups {
        h.v2(p.pos);
        h.i32(p.age);
    }
    h.i32(m.training as i32);
    if let Some((t, c)) = m.flash {
        h.i32(t);
        for v in c {
            h.f32(v);
        }
    }
    h.finish() as u128
}

fn set_digest<'a>(items: impl Iterator<Item = (&'a str, i64)>) -> u64 {
    items.fold(0u64, |acc, (k, v)| {
        let mut e = Fnv::default();
        e.bytes(k.as_bytes());
        e.i64(v);
        acc.wrapping_add(e.finish())
    })
}
