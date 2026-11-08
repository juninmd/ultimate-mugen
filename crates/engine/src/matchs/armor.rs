use super::*;

impl Match {
    pub(super) fn hit_override(&self, d: usize, hd: &HitDefData) -> Option<(i32, bool)> {
        self.fighters[d].hit_overrides.iter().flatten().find(|o| o.time != 0 && !o.keep && Self::attr_match(hd, &o.attr)).map(|o| (o.state_no, o.force_air))
    }

    pub(super) fn absorb_armored_hit(&mut self, a: usize, d: usize, hd: &HitDefData, at: (f32, f32), proj: Option<(i32, f32)>) -> bool {
        let Some(slot) = self.fighters[d].hit_overrides.iter().position(|o| {
            o.as_ref().is_some_and(|o| o.keep && o.time != 0 && o.hits > 0 && Self::attr_match(hd, &o.attr) && (o.threshold <= 0.0 || hd.damage <= o.threshold))
        }) else { return false };
        let chip = self.fighters[d].hit_overrides[slot].as_ref().map_or(0.0, |o| o.chip);
        let attack_mul = self.idx(self.fighters[a].root).map(|r| self.fighters[r].attack_mul).unwrap_or(1.0);
        let dmg = (hd.damage * attack_mul * chip / 100.0).round().max(1.0);
        let gt = self.game_time;
        let attacker_id = self.fighters[a].id;
        {
            let fd = &mut self.fighters[d];
            fd.life = (fd.life - dmg).max(fd.life.min(1.0));
            fd.received_damage += dmg;
            fd.received_hits += 1;
            fd.hit_shake = hd.pause_p2.min(6);
            fd.last_hit_by = Some(attacker_id);
            if let Some(o) = fd.hit_overrides[slot].as_mut() {
                o.hits -= 1;
                if o.hits <= 0 { fd.hit_overrides[slot] = None; }
            }
        }
        if let Some((pid, _)) = proj { self.fighters[a].proj_contacts.push(('h', pid, gt)); }
        else { let fa = &mut self.fighters[a]; fa.move_contact = 1; fa.move_hit = 1; fa.hit_pause = hd.pause_p1.min(8); }
        self.add_power(a, hd.get_power * 0.5);
        self.add_power(d, hd.give_power);
        self.spawn_spark(a, hd.guard_spark_no, hd.guard_spark_own, at.0, at.1, true, false, hd.spark_scale);
        if let Some(s) = hd.guard_sound { self.sound(a, s.0, s.1, s.2, Some("guard")); }
        let (aside, dside) = (self.fighters[a].side, self.fighters[d].side);
        self.events.push(Event::Hit { attacker_side: aside, defender_side: dside, damage: dmg, guarded: true, heavy: false, attacker: a, defender: d });
        true
    }
}
