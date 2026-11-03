//! HitDef parameter parsing.
use crate::chardef::Controller;
use crate::fighter::HitDefData;
use crate::matchs::Match;

fn animtype(s: &str) -> Option<i32> {
    Some(match s.trim().to_ascii_lowercase().as_str() { "light" => 0, "medium" | "med" => 1, "hard" | "heavy" => 2, "back" => 3, "up" => 4, "diagup" => 5, _ => return None })
}
fn groundtype(s: &str) -> Option<i32> {
    Some(match s.trim().to_ascii_lowercase().as_str() { "none" => 0, "high" => 1, "low" => 2, "trip" => 3, _ => return None })
}
pub fn parse_sound_ref(raw: Option<&str>) -> Option<(i32, i32, bool)> {
    let s = raw?.trim();
    let (own, rest) = if let Some(r) = s.strip_prefix(['s', 'S']) { (true, r) } else if let Some(r) = s.strip_prefix(['f', 'F']) { (false, r) } else { (false, s) };
    let mut it = rest.split(',').map(|x| x.trim().parse::<i32>());
    let g = it.next()?.ok()?;
    let n = it.next().and_then(|x| x.ok()).unwrap_or(0);
    Some((g, n, own))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animtype_and_groundtype() {
        assert_eq!(animtype("light"), Some(0));
        assert_eq!(animtype("MEDIUM"), Some(1));
        assert_eq!(animtype("med"), Some(1));
        assert_eq!(animtype("hard"), Some(2));
        assert_eq!(animtype("heavy"), Some(2));
        assert_eq!(animtype("back"), Some(3));
        assert_eq!(animtype("up"), Some(4));
        assert_eq!(animtype("diagup"), Some(5));
        assert_eq!(animtype("unknown"), None);

        assert_eq!(groundtype("none"), Some(0));
        assert_eq!(groundtype("high"), Some(1));
        assert_eq!(groundtype("LOW"), Some(2));
        assert_eq!(groundtype("trip"), Some(3));
        assert_eq!(groundtype("air"), None);
    }

    #[test]
    fn test_parse_sound_ref() {
        assert_eq!(parse_sound_ref(None), None);
        assert_eq!(parse_sound_ref(Some("")), None);
        assert_eq!(parse_sound_ref(Some("5, 0")), Some((5, 0, false)));
        assert_eq!(parse_sound_ref(Some("s5, 2")), Some((5, 2, true)));
        assert_eq!(parse_sound_ref(Some("S10, 3")), Some((10, 3, true)));
        assert_eq!(parse_sound_ref(Some("f6, 1")), Some((6, 1, false)));
        assert_eq!(parse_sound_ref(Some("F8, 4")), Some((8, 4, false)));
        assert_eq!(parse_sound_ref(Some("7")), Some((7, 0, false)));
        assert_eq!(parse_sound_ref(Some("invalid")), None);
    }
}

pub fn build(m: &Match, i: usize, sc: &Controller) -> HitDefData {
    let f = &m.fighters[i];
    let l = f.lscl;
    let num = |k: &str, d: f64, idx: usize| m.num(i, sc, k, d, idx) as f32;
    let has = |k: &str, idx: usize| sc.has(k, idx);
    let raw = |k: &str| sc.raw(k).map(|s| s.trim().to_string());
    let attr = raw("attr").unwrap_or("S,NA".into()).to_ascii_uppercase().replace(char::is_whitespace, "");
    let mut parts = attr.split(',');
    let attr_state = parts.next().unwrap_or("S").to_string();
    let attr_kind: Vec<String> = parts.map(|s| s.to_string()).collect();
    let damage = num("damage", 0.0, 0);
    let (pause_p1, pause_p2) = (num("pausetime", 0.0, 0) as i32, num("pausetime", 0.0, 1) as i32);
    let at = raw("animtype").and_then(|s| animtype(&s)).unwrap_or(0);
    let gt = raw("ground.type").and_then(|s| groundtype(&s)).unwrap_or(1);
    let spark = |r: Option<String>, d: i32| -> (i32, bool) {
        match r { None => (d, false), Some(s) => { let own = s.starts_with(['s', 'S']); (s.trim_start_matches(['s', 'S', 'f', 'F']).trim().parse().unwrap_or(d), own) } }
    };
    let (spark_no, spark_own) = spark(raw("sparkno"), f.c("data.sparkno", 2.0) as i32);
    let (guard_spark_no, guard_spark_own) = spark(raw("guard.sparkno"), f.c("data.guard.sparkno", 40.0) as i32);
    let gvx = num("ground.velocity", 0.0, 0) * l; let gvy = num("ground.velocity", 0.0, 1) * l;
    let fall = num("fall", 0.0, 0) != 0.0;
    let ght = num("ground.hittime", 0.0, 0) as i32;
    let gst = num("ground.slidetime", 0.0, 0) as i32;
    HitDefData {
        attr_state: attr_state.clone(), attr_kind: attr_kind.clone(),
        hitflag: raw("hitflag").unwrap_or("MAF".into()).to_ascii_uppercase(),
        guardflag: raw("guardflag").unwrap_or_default().to_ascii_uppercase(),
        animtype: at,
        air_animtype: raw("air.animtype").and_then(|s| animtype(&s)).unwrap_or(at),
        fall_animtype: raw("fall.animtype").and_then(|s| animtype(&s)).unwrap_or(if at >= 3 { at } else { 4 }),
        priority: num("priority", 4.0, 0) as i32,
        damage, guard_damage: num("damage", 0.0, 1),
        pause_p1, pause_p2,
        guard_pause_p1: if has("guard.pausetime", 0) { num("guard.pausetime", 0.0, 0) as i32 } else { pause_p1 },
        guard_pause_p2: if has("guard.pausetime", 0) { num("guard.pausetime", 0.0, 1) as i32 } else { pause_p2 },
        spark_no, spark_own, guard_spark_no, guard_spark_own,
        spark_x: num("sparkxy", 0.0, 0) * l, spark_y: num("sparkxy", 0.0, 1) * l,
        hit_sound: parse_sound_ref(raw("hitsound").as_deref()).or(Some((5, 0, false))),
        guard_sound: parse_sound_ref(raw("guardsound").as_deref()).or(Some((6, 0, false))),
        ground_type: gt, air_type: raw("air.type").and_then(|s| groundtype(&s)).unwrap_or(gt),
        ground_slide_time: gst, guard_slide_time: if has("guard.slidetime", 0) { num("guard.slidetime", 0.0, 0) as i32 } else { gst },
        ground_hit_time: ght, guard_hit_time: if has("guard.hittime", 0) { num("guard.hittime", 0.0, 0) as i32 } else { ght },
        air_hit_time: num("air.hittime", 20.0, 0) as i32,
        guard_ctrl_time: if has("guard.ctrltime", 0) { num("guard.ctrltime", 0.0, 0) as i32 } else if has("guard.slidetime", 0) { num("guard.slidetime", 0.0, 0) as i32 } else { gst },
        guard_dist: num("guard.dist", f.c("size.attack.dist", 160.0) as f64, 0),
        yaccel: num("yaccel", f.c("movement.yaccel", 0.44) as f64, 0) * l,
        ground_vel: [gvx, gvy],
        guard_vel: if has("guard.velocity", 0) { num("guard.velocity", 0.0, 0) * l } else { gvx },
        air_vel: if has("air.velocity", 0) { [num("air.velocity", 0.0, 0) * l, num("air.velocity", 0.0, 1) * l] } else { [gvx, gvy] },
        air_guard_vel: if has("airguard.velocity", 0) { [num("airguard.velocity", 0.0, 0) * l, num("airguard.velocity", 0.0, 1) * l] } else { [gvx * 1.5, gvy / 2.0] },
        p1sprpriority: num("p1sprpriority", 1.0, 0) as i32, p2sprpriority: num("p2sprpriority", 0.0, 0) as i32,
        p1facing: num("p1facing", 0.0, 0) as i32, p2facing: num("p2facing", 0.0, 0) as i32,
        p1stateno: if has("p1stateno", 0) { num("p1stateno", -1.0, 0) as i32 } else { -1 },
        p2stateno: if has("p2stateno", 0) { num("p2stateno", -1.0, 0) as i32 } else { -1 },
        p2getp1state: num("p2getp1state", 1.0, 0) != 0.0,
        forcestand: num("forcestand", if gvy != 0.0 { 1.0 } else { 0.0 }, 0) != 0.0,
        fall, air_fall: if has("air.fall", 0) { num("air.fall", 0.0, 0) != 0.0 } else { fall },
        fall_xvel: if has("fall.xvelocity", 0) { Some(num("fall.xvelocity", 0.0, 0) * l) } else { None },
        fall_yvel: num("fall.yvelocity", -4.5, 0) * l,
        fall_recover: num("fall.recover", 1.0, 0) != 0.0, fall_recover_time: num("fall.recovertime", 4.0, 0) as i32,
        fall_damage: num("fall.damage", 0.0, 0), fall_kill: num("fall.kill", 1.0, 0) != 0.0,
        kill: num("kill", 1.0, 0) != 0.0, guard_kill: num("guard.kill", 1.0, 0) != 0.0,
        get_power: if has("getpower", 0) { num("getpower", 0.0, 0) } else { damage },
        give_power: if has("givepower", 0) { num("givepower", 0.0, 0) } else { damage },
        guard_get_power: if has("getpower", 1) { num("getpower", 0.0, 1) } else { damage / 2.0 },
        guard_give_power: if has("givepower", 1) { num("givepower", 0.0, 1) } else { damage / 2.0 },
        id: num("id", 0.0, 0) as i32, chain_id: num("chainid", -1.0, 0) as i32,
        envshake_time: num("envshake.time", 0.0, 0) as i32, num_hits: num("numhits", 1.0, 0) as i32,
        juggle: f.st_def.states.get(&f.state_no).and_then(|sd| sd.param("juggle")).and_then(|p| p.exprs.first()).map(|e| m.ev(i, e) as i32).unwrap_or(0),
        airjuggle: if f.has_assert(crate::fighter::asf::NOJUGGLE) { i32::MAX / 2 } else { f.def.c("data.airjuggle", 15.0) as i32 },
    }
}
