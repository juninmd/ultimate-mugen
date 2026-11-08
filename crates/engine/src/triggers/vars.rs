use super::{b, Ctx};

impl<'a> Ctx<'a> {
    pub(super) fn ghv(&self, k: &str) -> f64 {
        let f = self.f(); let g = &f.ghv; let l = f.lscl as f64;
        match k {
            "type" => g.kind as f64, "animtype" => g.animtype as f64, "airtype" => g.airtype as f64, "groundtype" => g.groundtype as f64,
            "damage" => g.damage as f64, "hitcount" => g.hitcount as f64, "guardcount" => g.guardcount as f64, "hitshaketime" => f.hit_shake as f64,
            "hittime" => g.hittime as f64, "slidetime" => g.slidetime as f64, "ctrltime" => g.ctrltime as f64,
            "xvel" => g.xvel as f64 / l, "yvel" => g.yvel as f64 / l, "yaccel" => g.yaccel as f64 / l, "xaccel" => g.xaccel as f64 / l, "chainid" => g.hit_id as f64,
            "guarded" => g.guarded as i32 as f64, "fall" => g.fall as i32 as f64, "fall.damage" => g.fall_damage as f64,
            "fall.xvel" => g.fall_xvel as f64 / l, "fall.yvel" => g.fall_yvel as f64 / l, "fall.recover" => g.fall_recover as i32 as f64,
            "fall.time" => f.fall_time as f64, "fall.recovertime" => g.fall_recover_time as f64, "fall.kill" => g.fall_kill as i32 as f64,
            "fall.envshake.time" => g.fall_env_shake.time as f64, "fall.envshake.freq" => g.fall_env_shake.freq as f64,
            "fall.envshake.ampl" => (g.fall_env_shake.ampl / f.lscl) as f64, "fall.envshake.phase" => g.fall_env_shake.phase as f64,
            "kill" => g.kill as i32 as f64, "id" | "playerid" => g.attacker_id as f64, "playerno" => g.attacker_no as f64, "down.recover" => g.down_recover as i32 as f64,
            "recovertime" => if g.fall && g.fall_recover { (g.fall_recover_time - f.fall_time).max(0) as f64 } else { 0.0 },
            "priority" => g.priority as f64, "facing" => g.facing as f64, "power" => g.power as f64, "isbound" => b(f.bind.is_some()),
            "attr" => g.attr as f64, "guardflag" => g.guardflag as f64,
            "hitdamage" => g.hit_damage as f64, "guarddamage" => g.guard_damage as f64, "hitpower" => g.hit_power as f64, "guardpower" => g.guard_power as f64,
            "dizzypoints" => g.dizzy_points as f64, "guardpoints" => g.guard_points as f64, "redlife" => g.red_life as f64, "score" => g.score as f64,
            "teamside" => g.team_side as f64, "projid" => g.proj_id as f64, "p1stateno" => g.p1stateno as f64, "p2stateno" => g.p2stateno as f64,
            _ => 0.0,
        }
    }
    pub(super) fn hitdef_var(&self, k: &str) -> f64 {
        let f = self.f(); let l = f.lscl as f64;
        let Some(hd) = f.hitdef.as_ref() else { return 0.0 };
        let (stem, second) = match k.rsplit_once('.') { Some((s, "y")) | Some((s, "p2")) => (s, true), Some((s, "x")) | Some((s, "p1")) => (s, false), _ => (k, false) };
        let pick = |pair: [f64; 2]| pair[second as usize];
        match stem {
            "p1stateno" => hd.p1stateno as f64, "p2stateno" => hd.p2stateno as f64, "priority" => hd.priority as f64,
            "sparkx" => hd.spark_x as f64 / l, "sparky" => hd.spark_y as f64 / l, "sparkno" => hd.spark_no as f64, "guard.sparkno" => hd.guard_spark_no as f64,
            "pausetime" => pick([hd.pause_p1 as f64, hd.pause_p2 as f64]), "guard.pausetime" => pick([hd.guard_pause_p1 as f64, hd.guard_pause_p2 as f64]),
            "ground.velocity" => pick([hd.ground_vel[0] as f64 / l, hd.ground_vel[1] as f64 / l]), "air.velocity" => pick([hd.air_vel[0] as f64 / l, hd.air_vel[1] as f64 / l]),
            "down.velocity" => pick([hd.down_vel[0] as f64 / l, hd.down_vel[1] as f64 / l]), "guard.velocity" => hd.guard_vel as f64 / l,
            "airguard.velocity" => pick([hd.air_guard_vel[0] as f64 / l, hd.air_guard_vel[1] as f64 / l]),
            "attr" => crate::hitdef::attr_bits(&hd.attr_state, &hd.attr_kind) as f64, "guardflag" => crate::hitdef::guardflag_bits(&hd.guardflag) as f64,
            "id" => hd.id as f64, "chainid" => hd.chain_id.unwrap_or(-1) as f64, "damage" => hd.damage as f64, "guard.damage" => hd.guard_damage as f64,
            "hittime" | "ground.hittime" => hd.ground_hit_time as f64, "air.hittime" => hd.air_hit_time as f64, "guard.hittime" => hd.guard_hit_time as f64,
            "numhits" => hd.num_hits as f64, "fall" => hd.fall as i32 as f64, "yaccel" => hd.yaccel as f64 / l,
            _ => 0.0,
        }
    }
    pub(super) fn move_hit_var(&self, k: &str) -> f64 {
        let v = &self.f().move_hit_var; let now = self.m.game_time;
        match k {
            "id" => v.target_id as f64, "playerno" => v.target_no as f64, "sparkx" => v.spark_x as f64, "sparky" => v.spark_y as f64,
            "cornerpush" => v.corner_push as f64, "overridden" => b(v.overridden),
            "frame" => b(v.hit_time > 0 && now - v.hit_time <= 1),
            _ => 0.0,
        }
    }
    pub(super) fn proj_time(&self, kind: char, id: i32) -> i64 {
        let f = self.f();
        let mut best = -1i64;
        for &(k, pid, t) in &f.proj_contacts {
            if (kind == 'c' && k == 'x') || (kind != 'c' && k != kind) { continue; }
            if id != 0 && pid != id { continue; }
            let dt = self.m.game_time - t;
            if best < 0 || dt < best { best = dt; }
        }
        best
    }}