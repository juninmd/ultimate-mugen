//! State controller implementations (~90 MUGEN sctrls).
use crate::chardef::Controller;
use crate::fighter::{asf, AfterImage, Bind, BindKind, HitOverride, PalFx};
use crate::hitdef::{build, parse_sound_ref};
use crate::matchs::{Event, Match};
use std::sync::Arc;

fn set_vars(m: &mut Match, i: usize, sc: &Controller, add: bool, target: usize) {
    if sc.has("v", 0) || sc.has("fv", 0) {
        let isf = sc.has("fv", 0);
        let idx = m.num(i, sc, if isf { "fv" } else { "v" }, 0.0, 0) as usize;
        let v = m.num(i, sc, "value", 0.0, 0);
        let t = &mut m.fighters[target];
        if isf { if idx < 40 { t.fvars[idx] = if add { t.fvars[idx] + v } else { v }; } }
        else if idx < 60 { t.vars[idx] = if add { t.vars[idx] + v as i32 } else { v as i32 }; }
        return;
    }
    let keys: Vec<String> = sc.params.iter().map(|(k, _)| k.clone()).collect();
    for k in keys {
        let (sys, fl, rest) = if let Some(r) = k.strip_prefix("sysfvar(") { (true, true, r) } else if let Some(r) = k.strip_prefix("sysvar(") { (true, false, r) }
            else if let Some(r) = k.strip_prefix("fvar(") { (false, true, r) } else if let Some(r) = k.strip_prefix("var(") { (false, false, r) } else { continue };
        let Ok(idx) = rest.trim_end_matches(')').trim().parse::<usize>() else { continue };
        let v = m.num(i, sc, &k, 0.0, 0);
        let t = &mut m.fighters[target];
        match (sys, fl) {
            (false, false) if idx < 60 => t.vars[idx] = if add { t.vars[idx] + v as i32 } else { v as i32 },
            (false, true) if idx < 40 => t.fvars[idx] = if add { t.fvars[idx] + v } else { v },
            (true, false) if idx < 5 => t.sysvars[idx] = if add { t.sysvars[idx] + v as i32 } else { v as i32 },
            (true, true) if idx < 5 => t.sysfvars[idx] = if add { t.sysfvars[idx] + v } else { v },
            _ => {}
        }
    }
}

fn life_add(m: &mut Match, t: usize, v: f32, kill: bool, absolute: bool) {
    let f = &mut m.fighters[t];
    let dv = if !absolute && v < 0.0 { v / f.defence_mul } else { v };
    let mut nl = f.life + dv.trunc();
    if !kill && nl < 1.0 && f.life > 0.0 { nl = 1.0; }
    f.life = nl.clamp(0.0, f.life_max);
}

fn target_indices(m: &Match, i: usize, sc: &Controller) -> Vec<usize> {
    let id = if sc.has("id", 0) { m.num(i, sc, "id", -1.0, 0) as i32 } else { -1 };
    m.fighters[i].targets.iter().filter(|t| id < 0 || t.1 == id).filter_map(|t| m.idx(t.0)).collect()
}

pub fn run(m: &mut Match, i: usize, sc: &Controller) {
    let n = |m: &Match, k: &str, d: f64| m.num(i, sc, k, d, 0);
    let n2 = |m: &Match, k: &str, d: f64, idx: usize| m.num(i, sc, k, d, idx);
    let l = m.fighters[i].lscl;
    match sc.kind.as_str() {
        "changestate" | "selfstate" => {
            let v = n(m, "value", 0.0) as i32;
            if sc.has("ctrl", 0) { m.fighters[i].ctrl = n(m, "ctrl", 0.0) != 0.0; }
            let owner = if sc.kind == "selfstate" { m.fighters[i].def.clone() } else { m.fighters[i].st_def.clone() };
            let anim = if sc.has("anim", 0) { Some(n(m, "anim", 0.0) as i32) } else { None };
            m.change_state(i, v, owner);
            if let Some(a) = anim { let d = m.fighters[i].def.clone(); m.fighters[i].change_anim(a, &d, 1); }
        }
        "targetstate" => {
            let v = n(m, "value", 0.0) as i32;
            let d = m.fighters[i].def.clone();
            for t in target_indices(m, i, sc) { m.change_state(t, v, d.clone()); }
        }
        "changeanim" | "changeanim2" => {
            let v = n(m, "value", 0.0) as i32;
            let e = if sc.has("elem", 0) { n(m, "elem", 1.0) as i32 } else { 1 };
            let from = if sc.kind == "changeanim2" { m.fighters[i].st_def.clone() } else { m.fighters[i].def.clone() };
            m.fighters[i].change_anim(v, &from, e);
        }
        "velset" | "veladd" | "velmul" => {
            let x = if sc.has("x", 0) { Some(n(m, "x", 0.0) as f32) } else { None };
            let y = if sc.has("y", 0) { Some(n(m, "y", 0.0) as f32) } else { None };
            let f = &mut m.fighters[i];
            match sc.kind.as_str() {
                "velset" => { if let Some(x) = x { f.vel.x = x * l; } if let Some(y) = y { f.vel.y = y * l; } }
                "veladd" => { if let Some(x) = x { f.vel.x += x * l; } if let Some(y) = y { f.vel.y += y * l; } }
                _ => { if let Some(x) = x { f.vel.x *= x; } if let Some(y) = y { f.vel.y *= y; } }
            }
        }
        "posset" => {
            let x = if sc.has("x", 0) { Some(n(m, "x", 0.0) as f32) } else { None };
            let y = if sc.has("y", 0) { Some(n(m, "y", 0.0) as f32) } else { None };
            let cx = m.camera.x;
            let f = &mut m.fighters[i];
            if let Some(x) = x { f.pos.x = cx + x * l; }
            if let Some(y) = y { f.pos.y = y * l; }
        }
        "posadd" => {
            let x = if sc.has("x", 0) { Some(n(m, "x", 0.0) as f32) } else { None };
            let y = if sc.has("y", 0) { Some(n(m, "y", 0.0) as f32) } else { None };
            let f = &mut m.fighters[i];
            if let Some(x) = x { f.pos.x += x * l * f.facing; }
            if let Some(y) = y { f.pos.y += y * l; }
        }
        "posfreeze" => { let v = n(m, "value", 1.0) != 0.0; m.fighters[i].pos_freeze = v; }
        "gravity" => { let f = &mut m.fighters[i]; f.vel.y += f.c("movement.yaccel", 0.44) * l; }
        "ctrlset" => { let v = n(m, "value", 0.0) != 0.0; m.fighters[i].ctrl = v; }
        "statetypeset" => {
            let f = &mut m.fighters[i];
            if let Some(s) = sc.raw("statetype").and_then(|s| s.trim().chars().next()) { f.state_type = s.to_ascii_uppercase(); }
            if let Some(s) = sc.raw("movetype").and_then(|s| s.trim().chars().next()) { f.move_type = s.to_ascii_uppercase(); }
            if let Some(s) = sc.raw("physics").and_then(|s| s.trim().chars().next()) { f.physics = s.to_ascii_uppercase(); }
        }
        "turn" => { m.fighters[i].facing *= -1.0; }
        "hitdef" => { let hd = build(m, i, sc); let f = &mut m.fighters[i]; f.hitdef = Some(hd); f.hitdef_active = true; f.hitdef_targets.clear(); }
        "reversaldef" => {
            let attr = sc.raw("reversal.attr").unwrap_or("SCA").to_string();
            let hd = build(m, i, sc);
            m.fighters[i].reversal = Some((attr, hd));
        }
        "hitoverride" => {
            let attr = sc.raw("attr").unwrap_or("SCA,AA,AT,AP").to_string();
            let slot = (n(m, "slot", 0.0) as usize).min(7);
            let (state_no, time, force_air) = (n(m, "stateno", -1.0) as i32, n(m, "time", 1.0) as i32, n(m, "forceair", 0.0) != 0.0);
            m.fighters[i].hit_overrides[slot] = if state_no < 0 { None } else { Some(HitOverride { attr, state_no, time, force_air }) };
        }
        "fallenvshake" => {
            let s = m.fighters[i].ghv.fall_env_shake;
            if s.time > 0 { m.shake(s.time, s.freq, s.ampl, s.phase); m.fighters[i].ghv.fall_env_shake.time = 0; }
        }
        "hitvelset" => {
            let (x, y) = (n(m, "x", 1.0) != 0.0, n(m, "y", 1.0) != 0.0);
            let f = &mut m.fighters[i];
            if x { f.vel.x = f.ghv.xvel; } if y { f.vel.y = f.ghv.yvel; }
        }
        "hitfallvel" => { let f = &mut m.fighters[i]; if f.ghv.fall { if f.ghv.fall_xvel != 0.0 { f.vel.x = f.ghv.fall_xvel; } f.vel.y = f.ghv.fall_yvel; } }
        "hitfallset" => {
            let v = n(m, "value", -1.0);
            let xv = if sc.has("xvel", 0) { Some(n(m, "xvel", 0.0) as f32) } else { None };
            let yv = if sc.has("yvel", 0) { Some(n(m, "yvel", 0.0) as f32) } else { None };
            let f = &mut m.fighters[i];
            if v >= 0.0 { f.ghv.fall = v != 0.0; }
            if let Some(x) = xv { f.ghv.fall_xvel = x * l; } if let Some(y) = yv { f.ghv.fall_yvel = y * l; }
        }
        "hitfalldamage" => { let (d, k) = (m.fighters[i].ghv.fall_damage, m.fighters[i].ghv.fall_kill); life_add(m, i, -d, k, true); m.fighters[i].ghv.fall_damage = 0.0; }
        "playsnd" => {
            let r = sc.raw("value").unwrap_or("");
            if let Some((g, idx, _)) = parse_sound_ref(Some(r)) {
                let own = !r.trim().starts_with(['f', 'F']);
                let vol = n(m, "volumescale", 100.0) as f32 / 100.0;
                let ch = n(m, "channel", -1.0) as i32;
                let (pv, abs) = if sc.has("abspan", 0) { (n(m, "abspan", 0.0), true) } else { (n(m, "pan", 0.0), false) };
                let pan = crate::sctrl::sound_pan(m, i, pv as f32, abs);
                let looping = n(m, "loop", 0.0) != 0.0;
                let freq_mul = n(m, "freqmul", 1.0) as f32;
                let f = &m.fighters[i];
                let ev = Event::Sound { def: if own { Some(f.def.clone()) } else { None }, group: g, index: idx, own, volume: vol, channel: ch, owner: f.id, builtin: None, pan, looping, freq_mul };
                m.events.push(ev);
            }
        }
        "varset" => set_vars(m, i, sc, false, i),
        "varadd" => set_vars(m, i, sc, true, i),
        "parentvarset" | "parentvaradd" => { if let Some(p) = m.fighters[i].parent.and_then(|p| m.idx(p)) { set_vars(m, i, sc, sc.kind == "parentvaradd", p); } }
        "varrandom" => {
            let idx = n(m, "v", 0.0) as usize;
            let (lo, hi) = if sc.has("range", 1) { (n2(m, "range", 0.0, 0), n2(m, "range", 0.0, 1)) } else if sc.has("range", 0) { (0.0, n(m, "range", 1000.0)) } else { (0.0, 1000.0) };
            let r = m.random();
            if idx < 60 { m.fighters[i].vars[idx] = (lo + (r as f64 * (hi - lo + 1.0)).floor()) as i32; }
        }
        "varrangeset" => {
            let first = n(m, "first", 0.0) as usize;
            if sc.has("fvalue", 0) { let last = n(m, "last", 39.0) as usize; let v = n(m, "fvalue", 0.0); for k in first..=last.min(39) { m.fighters[i].fvars[k] = v; } }
            else { let last = n(m, "last", 59.0) as usize; let v = n(m, "value", 0.0) as i32; for k in first..=last.min(59) { m.fighters[i].vars[k] = v; } }
        }
        "poweradd" => { let v = n(m, "value", 0.0) as f32; m.add_power(i, v); }
        "powerset" => { let v = n(m, "value", 0.0) as f32; if let Some(r) = m.idx(m.fighters[i].root) { let f = &mut m.fighters[r]; f.power = v.clamp(0.0, f.power_max); } }
        "lifeadd" => { let (v, k, a) = (n(m, "value", 0.0) as f32, n(m, "kill", 1.0) != 0.0, n(m, "absolute", 0.0) != 0.0); life_add(m, i, v, k, a); }
        "lifeset" => { let v = n(m, "value", 0.0) as f32; let f = &mut m.fighters[i]; f.life = v.clamp(0.0, f.life_max); }
        "targetlifeadd" => {
            let (v, k, a) = (n(m, "value", 0.0) as f32, n(m, "kill", 1.0) != 0.0, n(m, "absolute", 0.0) != 0.0);
            let mul = if a { 1.0 } else { m.fighters[i].attack_mul };
            for t in target_indices(m, i, sc) { life_add(m, t, v * mul, k, a); }
        }
        "targetpoweradd" => { let v = n(m, "value", 0.0) as f32; for t in target_indices(m, i, sc) { m.add_power(t, v); } }
        "targetvelset" | "targetveladd" => {
            let x = if sc.has("x", 0) { Some(n(m, "x", 0.0) as f32 * l) } else { None };
            let y = if sc.has("y", 0) { Some(n(m, "y", 0.0) as f32 * l) } else { None };
            let add = sc.kind == "targetveladd";
            for t in target_indices(m, i, sc) {
                let f = &mut m.fighters[t];
                if let Some(x) = x { if add { f.vel.x += x } else { f.vel.x = x } }
                if let Some(y) = y { if add { f.vel.y += y } else { f.vel.y = y } }
            }
        }
        "targetfacing" => { let v = n(m, "value", 1.0); let fa = m.fighters[i].facing; for t in target_indices(m, i, sc) { m.fighters[t].facing = if v > 0.0 { fa } else { -fa }; } }
        "targetbind" => {
            let (time, x, y) = (n(m, "time", 1.0) as i32, n2(m, "pos", 0.0, 0) as f32 * l, n2(m, "pos", 0.0, 1) as f32 * l);
            let me = m.fighters[i].id;
            for t in target_indices(m, i, sc) { m.fighters[t].bind = Some(Bind { to: me, time, x, y, kind: BindKind::Target }); m.update_bind(t); }
        }
        "bindtotarget" | "bindtoparent" | "bindtoroot" => {
            let (time, x, y) = (n(m, "time", 1.0) as i32, n2(m, "pos", 0.0, 0) as f32 * l, n2(m, "pos", 0.0, 1) as f32 * l);
            let f = &m.fighters[i];
            let (to, kind) = match sc.kind.as_str() {
                "bindtotarget" => { let id = if sc.has("id", 0) { n(m, "id", -1.0) as i32 } else { -1 }; match f.targets.iter().find(|t| id < 0 || t.1 == id) { Some(t) => (t.0, BindKind::Target), None => return } }
                "bindtoparent" => match f.parent { Some(p) => (p, BindKind::Parent), None => return },
                _ => (f.root, BindKind::Root),
            };
            m.fighters[i].bind = Some(Bind { to, time, x, y, kind });
            m.update_bind(i);
        }
        "targetdrop" => { let keep = if sc.has("excludeid", 0) { n(m, "excludeid", -1.0) as i32 } else { -1 }; m.fighters[i].targets.retain(|t| keep >= 0 && t.1 == keep); }
        "explod" => m.spawn_explod(i, sc),
        "gamemakeanim" => {
            let (v, px, py, under) = (n(m, "value", 0.0) as i32, n2(m, "pos", 0.0, 0) as f32, n2(m, "pos", 0.0, 1) as f32, n(m, "under", 0.0) != 0.0);
            let f = &m.fighters[i]; let (x, y) = (f.pos.x + px * f.facing * l, f.pos.y + py * l);
            m.spawn_fx(i, v, x, y, under);
        }
        "removeexplod" => { let id = if sc.has("id", 0) { n(m, "id", -1.0) as i32 } else { -1 }; let me = m.fighters[i].id; m.explods.retain(|e| !(e.owner == me && (id < 0 || e.id == id))); }
        "explodbindtime" | "modifyexplod" => {
            let id = if sc.has("id", 0) { n(m, "id", -1.0) as i32 } else { -1 };
            let me = m.fighters[i].id;
            let bt = if sc.has("time", 0) { Some(n(m, "time", 0.0) as i32) } else if sc.has("value", 0) { Some(n(m, "value", 0.0) as i32) } else { None };
            let vel = if sc.has("vel", 0) { Some((n2(m, "vel", 0.0, 0) as f32 * l, n2(m, "vel", 0.0, 1) as f32 * l)) } else { None };
            let acc = if sc.has("accel", 0) { Some((n2(m, "accel", 0.0, 0) as f32 * l, n2(m, "accel", 0.0, 1) as f32 * l)) } else { None };
            let rt = if sc.has("removetime", 0) { Some(n(m, "removetime", -2.0) as i32) } else { None };
            let sc2 = if sc.has("scale", 0) { Some([n2(m, "scale", 1.0, 0) as f32, n2(m, "scale", 1.0, 1) as f32]) } else { None };
            let is_bind = sc.kind == "explodbindtime";
            for e in m.explods.iter_mut().filter(|e| e.owner == me && (id < 0 || e.id == id)) {
                if is_bind { if let Some(t) = bt { e.bind_time = t; } continue; }
                if let Some(v) = vel { e.vel.x = v.0; e.vel.y = v.1; }
                if let Some(v) = acc { e.accel.x = v.0; e.accel.y = v.1; }
                if let Some(t) = rt { e.remove_time = t; }
                if let Some(s) = sc2 { e.scale = s; }
            }
        }
        "helper" => m.spawn_helper(i, sc),
        "destroyself" => m.destroy_helper(i),
        "projectile" => m.spawn_projectile(i, sc),
        "width" => {
            let k = if sc.has("player", 0) { "player" } else if sc.has("edge", 0) { "edge" } else { "value" };
            let (a, b) = (n2(m, k, 0.0, 0) as f32 * l, n2(m, k, 0.0, 1) as f32 * l);
            m.fighters[i].width_override = Some((a, b));
        }
        "sprpriority" => { let v = n(m, "value", 0.0) as i32; m.fighters[i].spr_priority = v; }
        "assertspecial" => {
            for k in ["flag", "flag2", "flag3", "flag4", "flag5"] {
                if let Some(v) = sc.raw(k) {
                    let fl = v.trim().to_ascii_lowercase();
                    let bit = asf::from_name(&fl);
                    if bit != 0 { m.fighters[i].asserts |= bit; } else if fl == "timerfreeze" { m.global_asserts |= 1; } else if fl == "intro" { m.global_asserts |= 2; } else if fl == "roundnotover" { m.global_asserts |= 4; }
                }
            }
        }
        "playerpush" => { let v = n(m, "value", 1.0) != 0.0; m.fighters[i].player_push = v; }
        "screenbound" => { let v = n(m, "value", 0.0) != 0.0; m.fighters[i].screen_bound = v; }
        "afterimage" => { let (t, len) = (n(m, "time", 1.0) as i32, n(m, "length", 20.0) as usize); m.fighters[i].after_image = Some(AfterImage { time: t, length: len.min(30), trail: vec![] }); }
        "afterimagetime" => { let t = if sc.has("time", 0) { n(m, "time", 0.0) } else { n(m, "value", 0.0) } as i32; if let Some(a) = m.fighters[i].after_image.as_mut() { a.time = t; } }
        "palfx" => {
            let p = PalFx { time: n(m, "time", 0.0) as i32, add: [n2(m, "add", 0.0, 0) as f32, n2(m, "add", 0.0, 1) as f32, n2(m, "add", 0.0, 2) as f32], mul: [n2(m, "mul", 256.0, 0) as f32, n2(m, "mul", 256.0, 1) as f32, n2(m, "mul", 256.0, 2) as f32] };
            m.fighters[i].palfx = Some(p);
        }
        "allpalfx" | "bgpalfx" | "envcolor" => {
            let (k, d) = if sc.kind == "envcolor" { ("value", 255.0) } else { ("add", 0.0) };
            let t = n(m, "time", if sc.kind == "envcolor" { 1.0 } else { 0.0 }) as i32;
            let c = [n2(m, k, d, 0) as f32, n2(m, k, d, 1) as f32, n2(m, k, d, 2) as f32];
            if t > 0 { m.flash = Some((t, c)); }
        }
        "trans" => { let t = sc.raw("trans").unwrap_or("none").to_ascii_lowercase(); let (a, b) = (n2(m, "alpha", 256.0, 0) as f32, n2(m, "alpha", 0.0, 1) as f32); m.fighters[i].trans = Some((t, a, b)); }
        "angledraw" => {
            let v = if sc.has("value", 0) { Some(n(m, "value", 0.0) as f32) } else { None };
            let s = if sc.has("scale", 0) { Some([n2(m, "scale", 1.0, 0) as f32, n2(m, "scale", 1.0, 1) as f32]) } else { None };
            let f = &mut m.fighters[i]; f.angle_draw = true; if let Some(v) = v { f.angle = v; } if let Some(s) = s { f.draw_scale = s; }
        }
        "angleset" => { let v = n(m, "value", 0.0) as f32; m.fighters[i].angle = v; }
        "angleadd" => { let v = n(m, "value", 0.0) as f32; m.fighters[i].angle += v; }
        "anglemul" => { let v = n(m, "value", 1.0) as f32; m.fighters[i].angle *= v; }
        "nothitby" | "hitby" => {
            let attr = sc.raw("value").or(sc.raw("value2")).unwrap_or("").to_ascii_uppercase();
            let t = n(m, "time", 1.0) as i32;
            if sc.kind == "nothitby" { m.fighters[i].not_hit_by = vec![(attr, t)]; } else { m.fighters[i].hit_by = vec![(attr, t)]; }
        }
        "movehitreset" => { let f = &mut m.fighters[i]; f.move_contact = 0; f.move_hit = 0; f.move_guarded = 0; f.move_reversed = 0; }
        "hitadd" => { let v = n(m, "value", 0.0) as i32; m.fighters[i].hit_count += v; }
        "attackmulset" => { let v = n(m, "value", 1.0) as f32; m.fighters[i].attack_mul = v; }
        "defencemulset" => { let v = n(m, "value", 1.0) as f32; m.fighters[i].defence_mul = v; }
        "attackdist" => { let v = n(m, "value", 160.0) as f32; if let Some(h) = m.fighters[i].hitdef.as_mut() { h.guard_dist = v; } }
        "makedust" => { let (px, py) = (n2(m, "pos", 0.0, 0) as f32, n2(m, "pos", 0.0, 1) as f32); let f = &m.fighters[i]; let (x, y) = (f.pos.x + px * f.facing, f.pos.y + py); m.spawn_dust(i, x, y); }
        _ => { if !crate::sctrl::run(m, i, sc) { crate::ikemen::run(m, i, sc); } }
    }
    let _ = Arc::strong_count(&m.common);
}
