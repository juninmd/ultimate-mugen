use crate::chardef::Controller;
use crate::fighter::{HitDefData, Shake};
use crate::hitdef::build;
use crate::matchs::Match;

const HIT_VAR_KEYS: &[&str] = &[
    "hittype", "animtype", "airtype", "groundtype", "damage", "hitcount", "guardcount", "hitshaketime", "hittime", "slidetime", "ctrltime",
    "xvel", "yvel", "yaccel", "chainid", "guarded", "fall", "fall.damage", "fall.xvel", "fall.yvel", "fall.recover", "fall.time",
    "fall.recovertime", "fall.kill", "kill", "id", "playerno",
];

pub fn get_hit_var_set(m: &mut Match, i: usize, sc: &Controller) {
    let sets: Vec<(&str, f64)> = HIT_VAR_KEYS.iter().filter(|k| sc.has(k, 0)).map(|k| (*k, m.num(i, sc, k, 0.0, 0))).collect();
    let f = &mut m.fighters[i];
    let l = f.lscl;
    for (k, v) in sets {
        let g = &mut f.ghv;
        let flag = v != 0.0;
        match k {
            "hittype" => g.kind = v as i32,
            "animtype" => g.animtype = v as i32,
            "airtype" => g.airtype = v as i32,
            "groundtype" => g.groundtype = v as i32,
            "damage" => g.damage = v as f32,
            "hitcount" => g.hitcount = v as i32,
            "guardcount" => g.guardcount = v as i32,
            "hitshaketime" => f.hit_shake = v.max(0.0) as i32,
            "hittime" => g.hittime = v as i32,
            "slidetime" => g.slidetime = v as i32,
            "ctrltime" => g.ctrltime = v as i32,
            "xvel" => g.xvel = v as f32 * l,
            "yvel" => g.yvel = v as f32 * l,
            "yaccel" => g.yaccel = v as f32 * l,
            "chainid" => g.hit_id = v as i32,
            "guarded" => g.guarded = flag,
            "fall" => g.fall = flag,
            "fall.damage" => g.fall_damage = v as f32,
            "fall.xvel" => g.fall_xvel = v as f32 * l,
            "fall.yvel" => g.fall_yvel = v as f32 * l,
            "fall.recover" => g.fall_recover = flag,
            "fall.time" => f.fall_time = v as i32,
            "fall.recovertime" => g.fall_recover_time = v as i32,
            "fall.kill" => g.fall_kill = flag,
            "kill" => g.kill = flag,
            "id" => g.attacker_id = v.max(0.0) as u32,
            "playerno" => g.attacker_no = v.max(0.0) as usize,
            _ => {}
        }
    }
}

fn shake_key(prefix: &str, key: &str) -> bool {
    key.strip_prefix(prefix).is_some_and(|rest| matches!(rest, ".time" | ".freq" | ".ampl" | ".phase"))
}

fn copy_shake(dst: &mut Shake, src: &Shake, key: &str, prefix: &str) {
    if !shake_key(prefix, key) {
        return;
    }
    match &key[prefix.len()..] {
        ".time" => dst.time = src.time,
        ".freq" => dst.freq = src.freq,
        ".ampl" => dst.ampl = src.ampl,
        _ => dst.phase = src.phase,
    }
}

pub fn merge_hitdef(dst: &mut HitDefData, src: &HitDefData, sc: &Controller) {
    for (key, _) in &sc.params {
        let k = key.as_str();
        copy_shake(&mut dst.env_shake, &src.env_shake, k, "envshake");
        copy_shake(&mut dst.fall_env_shake, &src.fall_env_shake, k, "fall.envshake");
        match k {
            "attr" => { dst.attr_state = src.attr_state.clone(); dst.attr_kind = src.attr_kind.clone(); }
            "hitflag" => dst.hitflag = src.hitflag.clone(),
            "guardflag" => dst.guardflag = src.guardflag.clone(),
            "animtype" => dst.animtype = src.animtype,
            "air.animtype" => dst.air_animtype = src.air_animtype,
            "fall.animtype" => dst.fall_animtype = src.fall_animtype,
            "priority" => dst.priority = src.priority,
            "damage" => { dst.damage = src.damage; dst.guard_damage = src.guard_damage; }
            "pausetime" => { dst.pause_p1 = src.pause_p1; dst.pause_p2 = src.pause_p2; }
            "guard.pausetime" => { dst.guard_pause_p1 = src.guard_pause_p1; dst.guard_pause_p2 = src.guard_pause_p2; }
            "sparkno" => { dst.spark_no = src.spark_no; dst.spark_own = src.spark_own; }
            "guard.sparkno" => { dst.guard_spark_no = src.guard_spark_no; dst.guard_spark_own = src.guard_spark_own; }
            "sparkxy" => { dst.spark_x = src.spark_x; dst.spark_y = src.spark_y; }
            "hitsound" => dst.hit_sound = src.hit_sound,
            "guardsound" => dst.guard_sound = src.guard_sound,
            "ground.type" => dst.ground_type = src.ground_type,
            "air.type" => dst.air_type = src.air_type,
            "ground.slidetime" => dst.ground_slide_time = src.ground_slide_time,
            "guard.slidetime" => dst.guard_slide_time = src.guard_slide_time,
            "ground.hittime" => dst.ground_hit_time = src.ground_hit_time,
            "guard.hittime" => dst.guard_hit_time = src.guard_hit_time,
            "air.hittime" => dst.air_hit_time = src.air_hit_time,
            "guard.ctrltime" => dst.guard_ctrl_time = src.guard_ctrl_time,
            "guard.dist" => dst.guard_dist = src.guard_dist,
            "yaccel" => dst.yaccel = src.yaccel,
            "ground.velocity" => dst.ground_vel = src.ground_vel,
            "guard.velocity" => dst.guard_vel = src.guard_vel,
            "air.velocity" => dst.air_vel = src.air_vel,
            "airguard.velocity" => dst.air_guard_vel = src.air_guard_vel,
            "down.velocity" => dst.down_vel = src.down_vel,
            "down.hittime" => dst.down_hit_time = src.down_hit_time,
            "down.bounce" => dst.down_bounce = src.down_bounce,
            "p1sprpriority" => dst.p1sprpriority = src.p1sprpriority,
            "p2sprpriority" => dst.p2sprpriority = src.p2sprpriority,
            "p1facing" => dst.p1facing = src.p1facing,
            "p2facing" => dst.p2facing = src.p2facing,
            "p1stateno" => dst.p1stateno = src.p1stateno,
            "p2stateno" => dst.p2stateno = src.p2stateno,
            "p2getp1state" => dst.p2getp1state = src.p2getp1state,
            "forcestand" => dst.forcestand = src.forcestand,
            "fall" => dst.fall = src.fall,
            "air.fall" => dst.air_fall = src.air_fall,
            "fall.xvelocity" => dst.fall_xvel = src.fall_xvel,
            "fall.yvelocity" => dst.fall_yvel = src.fall_yvel,
            "fall.recover" => dst.fall_recover = src.fall_recover,
            "fall.recovertime" => dst.fall_recover_time = src.fall_recover_time,
            "fall.damage" => dst.fall_damage = src.fall_damage,
            "fall.kill" => dst.fall_kill = src.fall_kill,
            "kill" => dst.kill = src.kill,
            "guard.kill" => dst.guard_kill = src.guard_kill,
            "getpower" => { dst.get_power = src.get_power; dst.guard_get_power = src.guard_get_power; }
            "givepower" => { dst.give_power = src.give_power; dst.guard_give_power = src.guard_give_power; }
            "id" => dst.id = src.id,
            "chainid" => dst.chain_id = src.chain_id,
            "nochainid" => dst.no_chain_id = src.no_chain_id,
            "numhits" => dst.num_hits = src.num_hits,
            "hitonce" => dst.hit_once = src.hit_once,
            "snap" => dst.snap = src.snap,
            "mindist" => dst.min_dist = src.min_dist,
            "maxdist" => dst.max_dist = src.max_dist,
            "affectteam" => dst.affect_team = src.affect_team,
            "ground.cornerpush.veloff" => dst.ground_corner_push = src.ground_corner_push,
            "air.cornerpush.veloff" => dst.air_corner_push = src.air_corner_push,
            "guard.cornerpush.veloff" => dst.guard_corner_push = src.guard_corner_push,
            "airguard.cornerpush.veloff" => dst.air_guard_corner_push = src.air_guard_corner_push,
            "palfx.time" | "palfx.add" | "palfx.mul" => dst.target_palfx = src.target_palfx.clone(),
            "sparkscale" => { dst.spark_scale = src.spark_scale; if !sc.has("guard.sparkscale", 0) { dst.guard_spark_scale = src.guard_spark_scale; } }
            "guard.sparkscale" => dst.guard_spark_scale = src.guard_spark_scale,
            _ => {}
        }
    }
}

pub fn modify_hitdef(m: &mut Match, i: usize, sc: &Controller) {
    if m.fighters[i].hitdef.is_none() {
        return;
    }
    let fresh = build(m, i, sc);
    if let Some(hd) = m.fighters[i].hitdef.as_mut() {
        merge_hitdef(hd, &fresh, sc);
    }
}

pub fn modify_reversaldef(m: &mut Match, i: usize, sc: &Controller) {
    if m.fighters[i].reversal.is_none() {
        return;
    }
    let fresh = build(m, i, sc);
    let new_attr = sc.raw("reversal.attr").map(str::to_string);
    if let Some((attr, hd)) = m.fighters[i].reversal.as_mut() {
        merge_hitdef(hd, &fresh, sc);
        if let Some(a) = new_attr {
            *attr = a;
        }
    }
}
