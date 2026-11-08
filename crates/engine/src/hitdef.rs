//! HitDef parameter parsing.
use crate::chardef::Controller;
use crate::fighter::{HitDefData, PalFx, Shake, V2};
use crate::matchs::Match;

mod types;
use types::{animtype, groundtype};

pub fn attr_bits(state: &str, kinds: &[String]) -> i32 {
    let mut bits = 0;
    for c in state.chars() {
        bits |= match c {
            'S' => 1,
            'C' => 2,
            'A' => 4,
            'L' => 8,
            _ => 0,
        };
    }
    for k in kinds {
        bits |= match k.as_str() {
            "NA" => 64,
            "NT" => 128,
            "NP" => 256,
            "SA" => 512,
            "ST" => 1024,
            "SP" => 2048,
            "HA" => 4096,
            "HT" => 8192,
            "HP" => 16384,
            _ => 0,
        };
    }
    bits
}
pub fn attr_kind_mask(want: &str) -> i32 {
    let w = want.as_bytes();
    if w.len() != 2 {
        return 0;
    }
    let columns: &[i32] = match w[0].to_ascii_uppercase() {
        b'N' => &[0],
        b'S' => &[3],
        b'H' => &[6],
        b'A' => &[0, 3, 6],
        _ => return 0,
    };
    let row = match w[1].to_ascii_uppercase() {
        b'A' => 0,
        b'T' => 1,
        b'P' => 2,
        _ => return 0,
    };
    columns
        .iter()
        .fold(0, |mask, c| mask | (1 << (6 + c + row)))
}
pub fn state_mask(state: &str) -> i32 {
    state.bytes().fold(0, |bits, c| {
        bits | match c.to_ascii_uppercase() {
            b'S' => 1,
            b'C' => 2,
            b'A' => 4,
            b'L' => 8,
            _ => 0,
        }
    })
}
pub fn guardflag_bits(flags: &str) -> i32 {
    flags.bytes().fold(0, |bits, c| {
        bits | match c.to_ascii_uppercase() {
            b'H' => 1,
            b'L' => 2,
            b'A' => 4,
            b'M' => 3,
            _ => 0,
        }
    })
}
pub fn parse_sound_ref(raw: Option<&str>) -> Option<(i32, i32, bool)> {
    let s = raw?.trim();
    let (own, rest) = if let Some(r) = s.strip_prefix(['s', 'S']) {
        (true, r)
    } else if let Some(r) = s.strip_prefix(['f', 'F']) {
        (false, r)
    } else {
        (false, s)
    };
    let mut it = rest.split(',').map(|x| x.trim().parse::<i32>());
    let g = it.next()?.ok()?;
    let n = it.next().and_then(|x| x.ok()).unwrap_or(0);
    Some((g, n, own))
}

pub fn build(m: &Match, i: usize, sc: &Controller) -> HitDefData {
    let f = &m.fighters[i];
    let l = f.lscl;
    let num = |k: &str, d: f64, idx: usize| m.num(i, sc, k, d, idx) as f32;
    let has = |k: &str, idx: usize| sc.has(k, idx);
    let raw = |k: &str| sc.raw(k).map(|s| s.trim().to_string());
    let attr = raw("attr")
        .unwrap_or("S,NA".into())
        .to_ascii_uppercase()
        .replace(char::is_whitespace, "");
    let mut parts = attr.split(',');
    let attr_state = parts.next().unwrap_or("S").to_string();
    let attr_kind: Vec<String> = parts.map(|s| s.to_string()).collect();
    let damage = num("damage", 0.0, 0);
    let (pause_p1, pause_p2) = (
        num("pausetime", 0.0, 0) as i32,
        num("pausetime", 0.0, 1) as i32,
    );
    let at = raw("animtype").and_then(|s| animtype(&s)).unwrap_or(0);
    let gt = raw("ground.type").and_then(|s| groundtype(&s)).unwrap_or(1);
    let spark = |r: Option<String>, d: i32| -> (i32, bool) {
        match r {
            None => (d, false),
            Some(s) => {
                let own = s.starts_with(['s', 'S']);
                (
                    s.trim_start_matches(['s', 'S', 'f', 'F'])
                        .trim()
                        .parse()
                        .unwrap_or(d),
                    own,
                )
            }
        }
    };
    let (spark_no, spark_own) = spark(raw("sparkno"), f.c("data.sparkno", 2.0) as i32);
    let (guard_spark_no, guard_spark_own) =
        spark(raw("guard.sparkno"), f.c("data.guard.sparkno", 40.0) as i32);
    let gvx = num("ground.velocity", 0.0, 0) * l;
    let gvy = num("ground.velocity", 0.0, 1) * l;
    let fall = num("fall", 0.0, 0) != 0.0;
    let ght = num("ground.hittime", 0.0, 0) as i32;
    let guard_vel = if has("guard.velocity", 0) {
        num("guard.velocity", 0.0, 0) * l
    } else {
        gvx
    };
    let ground_corner_push = if has("ground.cornerpush.veloff", 0) {
        num("ground.cornerpush.veloff", 0.0, 0) * l
    } else if attr_state.contains('A') && !attr_state.contains('S') && !attr_state.contains('C') {
        0.0
    } else {
        1.3 * guard_vel
    };
    let parse_shake = |time_k: &str, freq_k: &str, ampl_k: &str, phase_k: &str, dir_k: &str, time_default: f64| -> Shake {
        let freq = num(freq_k, 60.0, 0).clamp(0.0, 180.0);
        Shake {
            time: num(time_k, time_default, 0) as i32,
            freq,
            ampl: num(ampl_k, -4.0, 0) * l,
            phase: num(
                phase_k,
                if freq >= 90.0 { 90.0 } else { 0.0 },
                0,
            ),
            dir: (num(dir_k, 0.0, 0) as u8).min(2),
        }
    };
    let opt = |k: &str, idx: usize| {
        if has(k, idx) {
            Some(num(k, 0.0, idx) * l)
        } else {
            None
        }
    };
    let target_palfx = if has("palfx.time", 0) || has("palfx.add", 0) || has("palfx.mul", 0) {
        Some(PalFx::new(
            num("palfx.time", 0.0, 0) as i32,
            [
                num("palfx.add", 0.0, 0),
                num("palfx.add", 0.0, 1),
                num("palfx.add", 0.0, 2),
            ],
            [
                num("palfx.mul", 256.0, 0),
                num("palfx.mul", 256.0, 1),
                num("palfx.mul", 256.0, 2),
            ],
        ))
    } else {
        None
    };
    let gst = num("ground.slidetime", 0.0, 0) as i32;
    let hyper = attr_kind.iter().any(|k| k.starts_with('H'));
    let life_mul = |name: &str| {
        f.def.c(
            &format!("{}.{name}", if hyper { "super" } else { "default" }),
            0.0,
        ) as f32
    };
    HitDefData {
        attr_state: attr_state.clone(),
        attr_kind: attr_kind.clone(),
        hitflag: raw("hitflag").unwrap_or("MAF".into()).to_ascii_uppercase(),
        guardflag: raw("guardflag").unwrap_or_default().to_ascii_uppercase(),
        animtype: at,
        air_animtype: raw("air.animtype").and_then(|s| animtype(&s)).unwrap_or(at),
        fall_animtype: raw("fall.animtype")
            .and_then(|s| animtype(&s))
            .unwrap_or(if at >= 3 { at } else { 4 }),
        priority: (num("priority", 4.0, 0) as i32).clamp(1, 7),
        damage,
        guard_damage: num("damage", 0.0, 1),
        pause_p1,
        pause_p2,
        guard_pause_p1: if has("guard.pausetime", 0) {
            num("guard.pausetime", 0.0, 0) as i32
        } else {
            pause_p1
        },
        guard_pause_p2: if has("guard.pausetime", 0) {
            num("guard.pausetime", 0.0, 1) as i32
        } else {
            pause_p2
        },
        spark_no,
        spark_own,
        guard_spark_no,
        guard_spark_own,
        spark_x: num("sparkxy", 0.0, 0) * l,
        spark_y: num("sparkxy", 0.0, 1) * l,
        hit_sound: parse_sound_ref(raw("hitsound").as_deref()).or(Some((5, 0, false))),
        guard_sound: parse_sound_ref(raw("guardsound").as_deref()).or(Some((6, 0, false))),
        ground_type: gt,
        air_type: raw("air.type").and_then(|s| groundtype(&s)).unwrap_or(gt),
        ground_slide_time: gst,
        guard_slide_time: if has("guard.slidetime", 0) {
            num("guard.slidetime", 0.0, 0) as i32
        } else {
            gst
        },
        ground_hit_time: ght,
        guard_hit_time: if has("guard.hittime", 0) {
            num("guard.hittime", 0.0, 0) as i32
        } else {
            ght
        },
        air_hit_time: num("air.hittime", 20.0, 0) as i32,
        guard_ctrl_time: if has("guard.ctrltime", 0) {
            num("guard.ctrltime", 0.0, 0) as i32
        } else if has("guard.slidetime", 0) {
            num("guard.slidetime", 0.0, 0) as i32
        } else {
            gst
        },
        guard_dist: num("guard.dist", f.c("size.attack.dist", 160.0) as f64, 0),
        yaccel: num("yaccel", f.c("movement.yaccel", 0.44) as f64, 0) * l,
        ground_vel: [gvx, gvy],
        guard_vel,
        air_vel: if has("air.velocity", 0) {
            [
                num("air.velocity", 0.0, 0) * l,
                num("air.velocity", 0.0, 1) * l,
            ]
        } else {
            [gvx, gvy]
        },
        air_guard_vel: if has("airguard.velocity", 0) {
            [
                num("airguard.velocity", 0.0, 0) * l,
                num("airguard.velocity", 0.0, 1) * l,
            ]
        } else {
            [guard_vel * 1.5, gvy / 2.0]
        },
        p1sprpriority: num("p1sprpriority", 1.0, 0) as i32,
        p2sprpriority: num("p2sprpriority", 0.0, 0) as i32,
        p1facing: num("p1facing", 0.0, 0) as i32,
        p2facing: num("p2facing", 0.0, 0) as i32,
        p1stateno: if has("p1stateno", 0) {
            num("p1stateno", -1.0, 0) as i32
        } else {
            -1
        },
        p2stateno: if has("p2stateno", 0) {
            num("p2stateno", -1.0, 0) as i32
        } else {
            -1
        },
        p2getp1state: num("p2getp1state", 1.0, 0) != 0.0,
        forcestand: num("forcestand", if gvy != 0.0 { 1.0 } else { 0.0 }, 0) != 0.0,
        fall,
        air_fall: if has("air.fall", 0) {
            num("air.fall", 0.0, 0) != 0.0
        } else {
            fall
        },
        fall_xvel: if has("fall.xvelocity", 0) {
            Some(num("fall.xvelocity", 0.0, 0) * l)
        } else {
            None
        },
        fall_yvel: num("fall.yvelocity", -4.5, 0) * l,
        fall_recover: num("fall.recover", 1.0, 0) != 0.0,
        fall_recover_time: num("fall.recovertime", 4.0, 0) as i32,
        fall_damage: num("fall.damage", 0.0, 0),
        fall_kill: num("fall.kill", 1.0, 0) != 0.0,
        kill: num("kill", 1.0, 0) != 0.0,
        guard_kill: num("guard.kill", 1.0, 0) != 0.0,
        get_power: if has("getpower", 0) {
            num("getpower", 0.0, 0)
        } else {
            damage
        },
        give_power: if has("givepower", 0) {
            num("givepower", 0.0, 0)
        } else {
            damage
        },
        guard_get_power: if has("getpower", 1) {
            num("getpower", 0.0, 1)
        } else {
            damage / 2.0
        },
        guard_give_power: if has("givepower", 1) {
            num("givepower", 0.0, 1)
        } else {
            damage / 2.0
        },
        id: num("id", 0.0, 0) as i32,
        chain_id: Some(num("chainid", -1.0, 0) as i32).filter(|v| *v >= 0),
        envshake_time: num("envshake.time", 0.0, 0) as i32,
        num_hits: num("numhits", 1.0, 0) as i32,
        juggle: f
            .st_def
            .states
            .get(&f.state_no)
            .and_then(|sd| sd.param("juggle"))
            .and_then(|p| p.exprs.first())
            .map(|e| m.ev(i, e) as i32)
            .unwrap_or(0),
        no_chain_id: [0, 1].map(|k| Some(num("nochainid", -1.0, k) as i32).filter(|v| *v >= 0)),
        hit_once: num(
            "hitonce",
            if attr_kind.iter().any(|k| k.ends_with('T')) {
                1.0
            } else {
                0.0
            },
            0,
        ) != 0.0,
        snap: if has("snap", 0) {
            Some(V2 {
                x: num("snap", 0.0, 0) * l,
                y: num("snap", 0.0, 1) * l,
            })
        } else {
            None
        },
        min_dist: [opt("mindist", 0), opt("mindist", 1)],
        max_dist: [opt("maxdist", 0), opt("maxdist", 1)],
        affect_team: raw("affectteam")
            .and_then(|s| s.chars().next())
            .map(|c| c.to_ascii_uppercase())
            .unwrap_or('E'),
        down_vel: if has("down.velocity", 0) {
            [
                num("down.velocity", 0.0, 0) * l,
                num("down.velocity", 0.0, 1) * l,
            ]
        } else {
            [gvx, gvy]
        },
        down_hit_time: if has("down.hittime", 0) {
            num("down.hittime", 0.0, 0) as i32
        } else {
            ght
        },
        down_bounce: num("down.bounce", 0.0, 0) != 0.0,
        fall_env_shake: parse_shake("fall.envshake.time", "fall.envshake.freq", "fall.envshake.ampl", "fall.envshake.phase", "fall.envshake.dir", 0.0),
        env_shake: parse_shake("envshake.time", "envshake.freq", "envshake.ampl", "envshake.phase", "envshake.dir", 0.0),
        ground_corner_push,
        air_corner_push: if has("air.cornerpush.veloff", 0) {
            num("air.cornerpush.veloff", 0.0, 0) * l
        } else {
            ground_corner_push
        },
        guard_corner_push: if has("guard.cornerpush.veloff", 0) {
            num("guard.cornerpush.veloff", 0.0, 0) * l
        } else {
            ground_corner_push
        },
        air_guard_corner_push: if has("airguard.cornerpush.veloff", 0) {
            num("airguard.cornerpush.veloff", 0.0, 0) * l
        } else {
            ground_corner_push
        },
        target_palfx,
        dizzy_points: num(
            "dizzypoints",
            (life_mul("lifetodizzypointsmul") * damage) as f64,
            0,
        ),
        guard_points: num(
            "guardpoints",
            (life_mul("lifetoguardpointsmul") * damage) as f64,
            0,
        ),
        red_life: [
            num("redlife", (life_mul("lifetoredlifemul") * damage) as f64, 0),
            num(
                "redlife",
                (life_mul("lifetoredlifemul") * num("damage", 0.0, 1)) as f64,
                1,
            ),
        ],
        score: [num("score", 0.0, 0), num("score", 0.0, 1)],
        down_recover: num("down.recover", 1.0, 0) != 0.0,
        miss_on_override: num("missonoverride", 0.0, 0) != 0.0,
        unhittable_time: num("unhittabletime", -1.0, 0) as i32,
        spark_scale: num("sparkscale", 1.0, 0).max(0.01),
        xaccel: num("xaccel", 0.0, 0) * l,
        attack_depth: num("attack.depth", 0.0, 0),
        p2clsncheck: num("p2clsncheck", 1.0, 0) != 0.0,
        p2clsnrequire: num("p2clsnrequire", -1.0, 0) as i32,
        airjuggle: if f.has_assert(crate::fighter::asf::NOJUGGLECHECK) {
            i32::MAX / 2
        } else {
            f.def.c("data.airjuggle", 15.0) as i32
        },
    }
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
