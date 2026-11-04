use crate::expr::{eval, Host, Node};
use crate::fighter::Fighter;
use crate::matchs::Match;

pub struct Ctx<'a> { pub m: &'a Match, pub i: usize }

const HIT_VARS: &[&str] = &[
    "type", "animtype", "airtype", "groundtype", "damage", "hitcount", "guardcount", "hitshaketime", "hittime", "slidetime", "ctrltime",
    "recovertime", "xvel", "yvel", "yaccel", "xaccel", "chainid", "guarded", "fall", "fall.damage", "fall.xvel", "fall.yvel", "fall.recover",
    "fall.time", "fall.recovertime", "fall.kill", "fall.envshake.time", "fall.envshake.freq", "fall.envshake.ampl", "fall.envshake.phase",
    "kill", "id", "playerid", "playerno", "down.recover", "priority", "facing", "power",
    "isbound", "attr", "guardflag", "hitdamage", "guarddamage", "hitpower", "guardpower", "dizzypoints", "guardpoints", "redlife",
    "score", "teamside", "projid", "p1stateno", "p2stateno",
];

pub fn note_hit_var(name: &str) {
    static REPORTED: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> = std::sync::OnceLock::new();
    if HIT_VARS.contains(&name) { return; }
    let first = REPORTED.get_or_init(Default::default).lock().map(|mut set| set.insert(name.to_string())).unwrap_or(false);
    if first { eprintln!("gethitvar: unknown parameter '{name}' evaluates to 0"); }
}

impl<'a> Ctx<'a> {
    fn f(&self) -> &Fighter { &self.m.fighters[self.i] }
    fn enemy_index(&self) -> Option<usize> { self.m.enemy(self.i, 0) }
    fn enemy(&self) -> Option<&Fighter> { self.enemy_index().map(|e| &self.m.fighters[e]) }
    fn to_local(&self, world: f32) -> f64 { world as f64 / self.f().lscl as f64 }
    fn local_delta(&self, other: &Fighter, comp: Option<char>) -> f64 {
        let f = self.f();
        self.to_local(if comp == Some('y') { other.pos.y - f.pos.y } else { (other.pos.x - f.pos.x) * f.facing })
    }
    fn screen_edge(&self, right: bool) -> f32 {
        let m = self.m;
        if right { m.camera.x + m.view_w / 2.0 } else { m.camera.x - m.view_w / 2.0 }
    }
    fn edge_dist(&self, front: bool) -> f32 {
        let f = self.f();
        if (f.facing > 0.0) == front { self.screen_edge(true) - f.pos.x } else { f.pos.x - self.screen_edge(false) }
    }
    fn const_val(&self, k: &str) -> f64 {
        if let Some((_, v)) = self.f().size_overrides.iter().find(|(n, _)| n == k) { return *v as f64; }
        let c = &self.f().def.constants;
        if let Some(v) = c.get(k) { return *v; }
        if k.starts_with("velocity.") && !k.ends_with(".x") && !k.ends_with(".y") { return *c.get(&format!("{k}.x")).unwrap_or(&0.0); }
        if k == "data.defense" { return *c.get("data.defence").unwrap_or(&100.0); }
        if k.ends_with(".x") { if let Some(v) = c.get(&k[..k.len() - 2]) { return *v; } }
        0.0
    }
    fn ghv(&self, k: &str) -> f64 {
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
    fn hitdef_var(&self, k: &str) -> f64 {
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
    fn move_hit_var(&self, k: &str) -> f64 {
        let v = &self.f().move_hit_var; let now = self.m.game_time;
        match k {
            "id" => v.target_id as f64, "playerno" => v.target_no as f64, "sparkx" => v.spark_x as f64, "sparky" => v.spark_y as f64,
            "cornerpush" => v.corner_push as f64, "overridden" => b(v.overridden),
            "frame" => b(v.hit_time > 0 && now - v.hit_time <= 1),
            _ => 0.0,
        }
    }
    fn proj_time(&self, kind: char, id: i32) -> i64 {
        let f = self.f();
        let mut best = -1i64;
        for &(k, pid, t) in &f.proj_contacts {
            if (kind == 'c' && k == 'x') || (kind != 'c' && k != kind) { continue; }
            if id != 0 && pid != id { continue; }
            let dt = self.m.game_time - t;
            if best < 0 || dt < best { best = dt; }
        }
        best
    }
    fn a(&self, args: &[Node], i: usize) -> f64 { args.get(i).map(|n| eval(n, self)).unwrap_or(0.0) }
}

fn cmp(x: f64, op: &str, y: f64) -> bool {
    match op { "=" => x == y, "!=" => x != y, "<" => x < y, "<=" => x <= y, ">" => x > y, ">=" => x >= y, _ => false }
}
fn b(x: bool) -> f64 { x as i32 as f64 }
fn attr_kind_match(want: &str, have: &[String]) -> bool {
    let w = want.as_bytes();
    if w.len() != 2 { return false; }
    let (w0, w1) = (w[0].to_ascii_uppercase(), w[1].to_ascii_uppercase());
    have.iter().any(|k| { let k = k.as_bytes(); k.len() == 2 && (w0 == b'A' || w0 == k[0].to_ascii_uppercase()) && w1 == k[1].to_ascii_uppercase() })
}

impl<'a> Host for Ctx<'a> {
    fn redirect(&self, kind: &str, arg: Option<f64>) -> Option<Self> {
        let m = self.m; let f = self.f();
        let i = match kind {
            "parent" => f.parent.and_then(|p| m.idx(p))?,
            "root" => m.idx(f.root)?,
            "helper" => { let id = arg.map(|a| a as i32).unwrap_or(-1); m.fighters.iter().position(|h| h.is_helper && h.root == f.root && !h.destroyed && (id < 0 || h.helper_id == id))? }
            "target" => { let id = arg.map(|a| a as i32).unwrap_or(-1); let t = f.targets.iter().find(|t| id < 0 || t.1 == id)?; m.idx(t.0)? }
            "enemy" | "enemynear" | "p2" => m.enemy(self.i, arg.unwrap_or(0.0) as usize)?,
            "partner" => m.partner(if f.is_helper { m.idx(f.root)? } else { self.i }, arg.map(|a| a.max(0.0) as usize).unwrap_or(0))?,
            "playerid" => m.idx(arg? as u32)?,
            "stateowner" => self.i,
            _ => { crate::diagnostics::note_unsupported(crate::diagnostics::Category::Redirection, kind); return None }
        };
        Some(Ctx { m, i })
    }
    fn cmpstr(&self, name: &str, neg: bool, values: &[String]) -> bool {
        let v0 = values.first().map(|s| s.as_str()).unwrap_or("");
        let c0 = v0.chars().next().unwrap_or(' ').to_ascii_uppercase();
        let f = self.f();
        let r = match name {
            "teammode" => crate::matchs::TeamMode::parse(v0).is_some_and(|t| t == self.m.team_mode()),
            "command" => self.m.has_command(self.i, values.first().map(|s| s.as_str()).unwrap_or("")),
            "statetype" => f.state_type == c0,
            "movetype" => f.move_type == c0,
            "physics" => f.physics == c0,
            "p2statetype" => self.enemy().is_some_and(|e| e.state_type == c0),
            "p2movetype" => self.enemy().is_some_and(|e| e.move_type == c0),
            "name" | "p1name" => f.def.name.eq_ignore_ascii_case(v0),
            "p2name" => self.enemy().is_some_and(|e| e.def.name.eq_ignore_ascii_case(v0)),
            "p3name" | "p4name" | "p5name" | "p6name" | "p7name" | "p8name" => {
                let slot = (name.as_bytes()[1] - b'0') as usize;
                self.m.roots().find(|r| r.player_no == slot).is_some_and(|r| r.def.name.eq_ignore_ascii_case(v0))
            }
            "p2command" => self.enemy_index().is_some_and(|e| self.m.has_command(e, v0)),
            "stagevar" => match (crate::stagevar::value(self.m, v0), values.get(1)) {
                (Some(crate::stagevar::StageValue::Text(t)), Some(want)) => t.eq_ignore_ascii_case(want),
                _ => false,
            },
            "authorname" => f.def.author.eq_ignore_ascii_case(v0),
            "displayname" => f.def.display_name.eq_ignore_ascii_case(v0),
            "selfcommand" => self.m.has_command(self.i, v0),
            "hitdefattr" => {
                let Some(hd) = f.hitdef.as_ref() else { return neg };
                if f.move_type != 'A' { return neg; }
                if v0.is_empty() || !v0.chars().all(|c| "SCAsca".contains(c)) { return neg; }
                let ok_state = hd.attr_state.chars().any(|c| v0.chars().any(|v| v.eq_ignore_ascii_case(&c)));
                ok_state && (values.len() == 1 || values[1..].iter().any(|v| attr_kind_match(v, &hd.attr_kind)))
            }
            "hitbyattr" => {
                let attr = f.ghv.attr;
                if attr == 0 { return neg; }
                let ok_state = crate::hitdef::state_mask(&v0.to_ascii_uppercase()) & attr != 0;
                ok_state && (values.len() == 1 || values[1..].iter().any(|v| crate::hitdef::attr_kind_mask(&v.to_ascii_uppercase()) & attr != 0))
            }
            "prevstatetype" => f.prev_state_type == c0,
            "prevmovetype" => f.prev_move_type == c0,
            _ => { crate::diagnostics::note_unsupported(crate::diagnostics::Category::StringTrigger, name); false }
        };
        r != neg
    }
    fn special(&self, name: &str, op: &str, a: f64, op2: Option<&str>, bb: Option<f64>) -> bool {
        let neg = op == "!=";
        let f = self.f();
        if name == "animelem" {
            return match bb {
                None => f.anim_elem_started(a as i32) != neg,
                Some(t) => match f.anim_elem_time(a as i32) { Some(dt) => cmp(dt as f64, op2.unwrap_or("="), t), None => false },
            };
        }
        if name == "timemod" {
            let m = a as i32;
            return m != 0 && ((f.state_time % m) as f64 == bb.unwrap_or(0.0)) != neg;
        }
        let (kind, rest) = if let Some(r) = name.strip_prefix("projhit") { ('h', r) } else if let Some(r) = name.strip_prefix("projcontact") { ('c', r) } else if let Some(r) = name.strip_prefix("projguarded") { ('g', r) } else { return false };
        let id = rest.parse::<i32>().unwrap_or(0);
        let t = self.proj_time(kind, id);
        let happened = t >= 0;
        let want = a != 0.0;
        match bb { None => (happened == want) != neg, Some(v) => happened == want && cmp(t as f64, op2.unwrap_or("="), v) }
    }
    fn assign(&self, name: &str, idx: i32, v: f64) -> f64 {
        self.m.assigns.borrow_mut().push((self.i, name.to_string(), idx, v));
        if name == "var" || name == "sysvar" { (v as i32) as f64 } else { v }
    }
    fn trigger(&self, name: &str, args: &[Node], comp: Option<char>, raw: Option<&str>) -> f64 {
        let f = self.f(); let m = self.m; let l = f.lscl as f64;
        let a0 = || self.a(args, 0);
        let edist = |front: bool| self.edge_dist(front) as f64;
        match name {
            "__str" => 0.0,
            "time" | "statetime" => f.state_time as f64,
            "animtime" => f.anim_time_trig() as f64,
            "anim" => f.anim_no as f64,
            "animelemno" => f.anim_elem_no_at(a0() as i32) as f64,
            "animelemtime" => f.anim_elem_time(a0() as i32).unwrap_or(-1) as f64,
            "animexist" | "selfanimexist" => b(f.def.anims.contains_key(&(a0() as i32))),
            "stateno" => f.state_no as f64,
            "prevstateno" => f.prev_state_no as f64,
            "ctrl" => b(f.ctrl && !f.ikemen.dizzy && !f.ikemen.guard_break),
            "alive" => b(f.alive),
            "life" => f.life as f64, "lifemax" => f.life_max as f64,
            "power" => m.idx(f.root).map(|r| m.fighters[r].power).unwrap_or(f.power) as f64,
            "powermax" => f.power_max as f64,
            "facing" => f.facing as f64,
            "var" => f.vars.get(a0() as usize).copied().unwrap_or(0) as f64,
            "fvar" => f.fvars.get(a0() as usize).copied().unwrap_or(0.0),
            "sysvar" => f.sysvars.get(a0() as usize).copied().unwrap_or(0) as f64,
            "sysfvar" => f.sysfvars.get(a0() as usize).copied().unwrap_or(0.0),
            "vel" => (match comp { Some('y') => f.vel.y, Some('z') => 0.0, _ => f.vel.x }) as f64 / l,
            "pos" => (match comp { Some('y') => f.air_height(), Some('z') => 0.0, _ => f.pos.x - m.camera.x }) as f64 / l,
            "screenpos" => self.to_local(match comp { Some('y') => f.pos.y - m.camera.y + m.ground_y(), _ => f.pos.x - m.camera.x + m.view_w / 2.0 }),
            "camerapos" => (if comp == Some('y') { m.camera.y } else { m.camera.x }) as f64,
            "camerazoom" => m.camera_scale() as f64,
            "zoomvar" => m.zoom_var(raw.unwrap_or("")),
            "p2dist" | "p1dist" => self.enemy().map(|e| self.local_delta(e, comp)).unwrap_or(0.0),
            "p2bodydist" => self.enemy().map(|e| {
                if comp == Some('y') { return self.local_delta(e, comp); }
                let dx = (e.pos.x - f.pos.x) * f.facing;
                let theirs = if (e.pos.x - f.pos.x) * e.facing < 0.0 { e.front_width() } else { e.back_width() };
                self.to_local(dx - f.front_width() - theirs)
            }).unwrap_or(0.0),
            "rootdist" => m.idx(f.root).map(|r| self.local_delta(&m.fighters[r], comp)).unwrap_or(0.0),
            "parentdist" => f.parent.and_then(|p| m.idx(p)).map(|r| self.local_delta(&m.fighters[r], comp)).unwrap_or(0.0),
            "frontedgedist" => edist(true) / l,
            "backedgedist" => edist(false) / l,
            "frontedgebodydist" => (edist(true) - f.front_width() as f64) / l,
            "backedgebodydist" => (edist(false) - f.back_width() as f64) / l,
            "p2life" => self.enemy().map(|e| e.life as f64).unwrap_or(0.0),
            "p2stateno" => self.enemy().map(|e| e.state_no as f64).unwrap_or(0.0),
            "p2ctrl" => b(self.enemy().is_some_and(|e| e.ctrl)),
            "p2power" => self.enemy().map(|e| e.power as f64).unwrap_or(0.0),
            "hitshakeover" => b(f.hit_shake <= 0),
            "hitover" => b(f.hit_over_time <= 0),
            "hitfall" => b(f.ghv.fall),
            "canrecover" => b(f.ghv.fall_recover && f.fall_time >= f.ghv.fall_recover_time),
            "movecontact" => f.move_contact as f64, "movehit" => f.move_hit as f64, "moveguarded" => f.move_guarded as f64, "movereversed" => f.move_reversed as f64,
            "hitcount" => f.hit_count as f64,
            "uniqhitcount" => f.uniq_hit_count as f64,
            "hitpausetime" => f.hit_pause as f64,
            "inguarddist" => b(m.in_guard_dist(self.i)),
            "gethitvar" => self.ghv(raw.unwrap_or("")),
            "hitdefvar" => self.hitdef_var(raw.unwrap_or("")),
            "movehitvar" => self.move_hit_var(raw.unwrap_or("")),
            "movecountered" => f.move_countered as f64,
            "hitoverridden" => b(f.hit_overridden_at > 0 && m.game_time - f.hit_overridden_at <= 1),
            "prevanim" => f.prev_anim as f64,
            "selfstatenoexist" => b(f.def.states.contains_key(&(a0() as i32))),
            "inrunstate" => b(matches!(f.state_no, 100 | 105)),
            "incustomstate" => b(!std::sync::Arc::ptr_eq(&f.st_def, &f.def)),
            "incustomanim" => b(!std::sync::Arc::ptr_eq(&f.anim_def, &f.def)),
            "jugglepoints" => {
                let budget = f.last_hit_by.and_then(|id| m.idx(id)).filter(|_| f.move_type == 'H').map(|k| m.fighters[k].def.c("data.airjuggle", 15.0)).unwrap_or_else(|| f.def.c("data.airjuggle", 15.0));
                (budget - f.juggle_used as f64).max(0.0)
            }
            "hitvel" => (if comp == Some('y') { f.ghv.yvel } else { f.ghv.xvel }) as f64 / l,
            "parentexist" => b(f.parent.and_then(|p| m.idx(p)).is_some_and(|k| !m.fighters[k].destroyed)),
            "airjumpcount" => f.air_jumps as f64,
            "const" => self.const_val(raw.unwrap_or("")),
            "const240p" => a0() * f.def.local_w as f64 / 320.0,
            "const480p" => a0() * f.def.local_w as f64 / 640.0,
            "const720p" => a0() * f.def.local_w as f64 / 1280.0,
            "random" => { let x = (m.game_time as u64).wrapping_mul(2654435761).wrapping_add(f.id as u64 * 40503 + f.state_time as u64 * 977 + m.rng as u64); ((x >> 7) % 1000) as f64 }
            "randomrange" => { let (lo, hi) = (a0(), self.a(args, 1)); let x = (m.game_time as u64).wrapping_mul(2654435761).wrapping_add(m.rng as u64); lo + ((x >> 7) % ((hi - lo + 1.0).max(1.0) as u64)) as f64 }
            "roundstate" => m.round_state as f64,
            "roundno" => m.round_no as f64,
            "roundsexisted" => (m.round_no - 1) as f64,
            "matchover" => b(m.match_over),
            "matchno" => 1.0,
            "gametime" => m.game_time as f64,
            "tickspersecond" => 60.0,
            "ailevel" => m.idx(f.root).filter(|&r| !m.fighters[r].has_assert(crate::flags::asf::NOAILEVEL)).map(|r| m.fighters[r].ai_level).unwrap_or(0) as f64,
            "ishelper" => b(f.is_helper && (args.is_empty() || f.helper_id == a0() as i32)),
            "ishometeam" => b(f.side == 0),
            "teamside" => (f.side + 1) as f64,
            "teammode" => m.team_mode().number() as f64,
            "teamsize" => m.team_size_of(f.side) as f64,
            "teamleader" => m.team_leader_no(f.side) as f64,
            "numplayer" => m.num_players() as f64,
            "playerno" => m.slot_no(self.i) as f64,
            "memberno" => m.member_no(self.i) as f64,
            "playerindexexist" => { let n = a0(); b(n >= 0.0 && m.fighters.get(n as usize).is_some_and(|h| !h.destroyed)) }
            "lastplayerid" => m.next_id.saturating_sub(1) as f64,
            "introstate" => match m.round_state { 0 => 1.0, 1 => 2.0, _ => 0.0 },
            "outrostate" => match m.round_state { 3 => 1.0, 4 => 2.0, _ => 0.0 },
            "id" => f.id as f64,
            "playeridexist" => b(m.idx(a0() as u32).is_some()),
            "numhelper" => { let id = if args.is_empty() { -1 } else { a0() as i32 }; m.fighters.iter().filter(|h| h.is_helper && h.root == f.root && !h.destroyed && (id < 0 || h.helper_id == id)).count() as f64 }
            "numexplod" => { let id = if args.is_empty() { -1 } else { a0() as i32 }; m.explods.iter().filter(|e| e.owner == f.id && (id < 0 || e.id == id)).count() as f64 }
            "numproj" => m.projectiles.iter().filter(|p| p.tally == f.projectile_tally_id() && !p.remove).count() as f64,
            "numprojid" => { let id = a0() as i32; m.projectiles.iter().filter(|p| p.tally == f.projectile_tally_id() && !p.remove && p.id == id).count() as f64 }
            "numtarget" => { let id = if args.is_empty() { -1 } else { a0() as i32 }; f.targets.iter().filter(|t| id < 0 || t.1 == id).count() as f64 }
            "numenemy" => m.roots().filter(|r| r.side != f.side).count() as f64,
            "numpartner" => m.roots().filter(|r| r.side == f.side && r.id != f.id).count() as f64,
            "win" => b(m.winner_side == f.side as i32),
            "lose" => b(m.winner_side >= 0 && m.winner_side != f.side as i32),
            "winko" => b(m.winner_side == f.side as i32 && m.won_by_ko),
            "wintime" => b(m.winner_side == f.side as i32 && !m.won_by_ko),
            "winperfect" => b(m.winner_side == f.side as i32 && m.roots().filter(|r| r.side == f.side).all(|r| r.life >= r.life_max)),
            "loseko" => b(m.winner_side >= 0 && m.winner_side != f.side as i32 && m.won_by_ko),
            "losetime" => b(m.winner_side >= 0 && m.winner_side != f.side as i32 && !m.won_by_ko),
            "attack" | "attackmul" => f.attack_mul as f64,
            "defence" | "defencemul" => f.defence_mul as f64,
            "receiveddamage" => f.received_damage as f64,
            "receivedhits" => f.received_hits as f64,
            "combocount" => f.combo_count as f64,
            "score" => f.score,
            "scoretotal" => f.score_total,
            "round" => m.round_no as f64,
            "roundtime" => m.round_time as f64,
            "timeremaining" => ((m.timer.max(0) + 59) / 60) as f64,
            "timeelapsed" => (m.round_time / 60) as f64,
            "timetotal" => (m.round_ticks / 60) as f64,
            "fighttime" => (m.round_time / 60) as f64,
            "roundswon" => m.wins[f.side.min(1)] as f64,
            "firstattack" => b(m.first_attack == Some(f.side)),
            "decisiveround" => b(m.wins[0].max(m.wins[1]) == m.rounds_to_win - 1),
            "projcanceltime" => self.proj_time('x', a0() as i32) as f64,
            "mugenversion" => f.def.mugen_version as f64,
            "drawgame" => b(m.winner_side == -2),
            "palno" => f.pal_no as f64,
            "majorversion" => 1.0,
            "projcontact" | "projhit" | "projguarded" => b(self.proj_time(match name { "projhit" => 'h', "projguarded" => 'g', _ => 'c' }, 0) >= 0),
            "projcontacttime" => self.proj_time('c', a0() as i32) as f64,
            "projhittime" => self.proj_time('h', a0() as i32) as f64,
            "projguardedtime" => self.proj_time('g', a0() as i32) as f64,
            "leftedge" => self.screen_edge(false) as f64 / l,
            "rightedge" => self.screen_edge(true) as f64 / l,
            "frontedge" => self.screen_edge(f.facing > 0.0) as f64 / l,
            "backedge" => self.screen_edge(f.facing <= 0.0) as f64 / l,
            "topedge" => (m.camera.y - m.ground_y()) as f64,
            "bottomedge" => (m.camera.y + m.view_h - m.ground_y()) as f64,
            "stagevar" => match crate::stagevar::value(m, raw.unwrap_or("")) { Some(crate::stagevar::StageValue::Num(v)) => v, _ => 0.0 },
            "gameheight" => m.view_h as f64, "gamewidth" => m.view_w as f64,
            "abs" => a0().abs(), "floor" => a0().floor(), "ceil" => a0().ceil(), "sin" => a0().sin(), "cos" => a0().cos(), "tan" => a0().tan(),
            "asin" => a0().asin(), "acos" => a0().acos(), "atan" => a0().atan(), "exp" => a0().exp(), "ln" => a0().ln(), "sqrt" => a0().sqrt(),
            "log" => self.a(args, 1).ln() / a0().ln(),
            "min" => a0().min(self.a(args, 1)), "max" => a0().max(self.a(args, 1)),
            "clamp" => a0().max(self.a(args, 1)).min(self.a(args, 2)),
            "sign" => a0().signum(), "e" => std::f64::consts::E, "pi" => std::f64::consts::PI,
            _ => crate::scene_triggers::trigger(self, name, args, comp, raw).or_else(|| crate::ikemen::introspect::trigger(self, name, args, raw)).or_else(|| crate::ikemen::trigger(self, name, args)).unwrap_or_else(|| { crate::diagnostics::note_unsupported(crate::diagnostics::Category::Trigger, name); 0.0 }),
        }
    }
}
