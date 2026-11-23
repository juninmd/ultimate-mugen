use super::compile::extras;
use super::types::*;
use std::collections::HashMap;
use ume_engine::chardef::MoveInfo;
use ume_engine::commands::{parse_command, CommandDef};

pub(super) fn compile_moves(nc: &NativeChar, anim_no: &HashMap<String, i32>, block: i32, commands: &mut Vec<CommandDef>, minus1_out: &mut String, cns_out: &mut String) -> (Vec<MoveInfo>, HashMap<String, i32>) {
    let (mut minus1, mut cns) = (std::mem::take(minus1_out), std::mem::take(cns_out));
    let rank = |m: &NativeMove| (if super::ultimate::is(&m.id) { 400 } else { match m.kind.as_str() { "super" => 300, "special" | "finisher" => 200, _ => 100 } }) + m.input.split(',').count() as i32 * 5 + m.input.contains('+') as i32 * 2;
    let mut order = super::forms::collect_form_moves(nc);
    order.sort_by_key(|(m, _, _)| -rank(m));
    let plan = super::chain::ChainPlan::at(order.iter().map(|(m, _, _)| *m), block);
    let mut movelist = vec![];
    let mut move_states = HashMap::new();
    let mut throw_defs = String::new();
    for (index, (m, form_cond, forms_mask)) in order.into_iter().enumerate() {
        let when = m.when.as_deref().unwrap_or("stand");
        let sid = plan.sid(index);
        move_states.entry(m.id.clone()).or_insert(sid);
        let (cns_start, minus1_start) = (cns.len(), minus1.len());
        let cname = format!("{}_{}", if m.kind == "finisher" { "fin" } else { "m" }, m.id);
        let simple_btn = m.input.trim().split('+').all(|t| ["a", "b", "c", "x", "y", "z"].contains(&t.trim()));
        let mut cdef = parse_command(&cname, &m.input, if m.kind == "super" { 30 } else if m.kind == "special" || m.kind == "finisher" { 20 } else { 15 }, if simple_btn { 1 } else { 4 });
        cdef.pause_buffer = if m.kind == "super" { 8 } else if m.kind == "special" || m.kind == "finisher" { 6 } else if simple_btn { 0 } else { 4 };
        let listed = !m.input.trim().is_empty(); if listed { commands.push(cdef); }
        let cost = m.cost.unwrap_or(if super::ultimate::is(&m.id) { super::ultimate::COST } else if m.kind == "super" { 1000.0 } else { 0.0 });
        if listed { movelist.push(MoveInfo { name: m.name.clone(), input: m.input.clone(), command: cname.clone(), cost, drain: false, forms: forms_mask, state: sid }); }
        let mut simple_cmd = String::new();
        if let Some(sp) = &m.simple {
            let sc = format!("simple_{sp}_{}", m.id);
            let inp = match sp.as_str() { "sp1" => "d", "sp2" => "w", "sp3" => "e", _ => "d+w" };
            let mut sc_def = parse_command(&sc, inp, 1, 4); sc_def.pause_buffer = 6;
            commands.push(sc_def);
            simple_cmd = format!(" || command = \"{sc}\"");
        }
        let air_entry = format!("statetype = A{}", plan.air_entry_states(index));
        let fly_entry = format!("stateno = {}", super::abilities::FLY_STATE);
        let state_cond = match when { "air" => air_entry.as_str(), "fly" => fly_entry.as_str(), "crouch" => "statetype = C || (statetype = S && command = \"holddown\")", _ => "statetype = S || statetype = C && command != \"holddown\"" };
        let crouch_ok = match when { "crouch" => "\ntriggerall = command = \"holddown\"", "stand" => "\ntriggerall = command != \"holddown\" || statetype = S", _ => "" };
        let cancel = super::rekka::lead(m, &plan, index);
        let form_line = form_cond.as_ref().map(|fc| format!("triggerall = {fc}\n")).unwrap_or_default();
        let cost_line = if cost > 0.0 { format!("triggerall = power >= {cost}\n") } else { String::new() } + &super::requires::trigger_lines(m);
        if listed { minus1 += &format!("[State -1, {}]\ntype = ChangeState\nvalue = {sid}\ntriggerall = !IsHelper && (command = \"{cname}\"{simple_cmd})\ntriggerall = {state_cond}{crouch_ok}\n{form_line}{cost_line}{cancel}{}\n\n", m.name, if m.burst == Some(true) { "\ntrigger2 = movetype = H && stateno != [5100, 5150]" } else { "" }); }
        let anim = anim_no.get(&m.anim).copied().unwrap_or(0);
        let st = match when { "air" | "fly" => 'A', "crouch" => 'C', _ => 'S' };
        let jumps = m.moves.iter().any(|v| v.vel[1].is_some_and(|y| y != 0.0));
        let ph = match when { "air" => 'A', "fly" => 'N', "crouch" => 'C', _ => if jumps || m.flight == Some(true) { 'N' } else { 'S' } };
        let pa = if cost > 0.0 { -cost } else if m.kind == "special" { 30.0 } else { 15.0 };
        cns += &format!("[Statedef {sid}]\ntype = {st}\nmovetype = A\nphysics = {ph}\nanim = {anim}\nctrl = 0\n{}poweradd = {pa}\nsprpriority = 2\n\n", if when == "air" { "" } else { "velset = 0, 0\n" }); cns += &super::armor::armor_state(nc, m, sid); cns += &extras::voice_and_traits_state(m, sid); let (throw_cns, throw_def) = super::throw::throw_states(m, sid, &anim_no); cns += &throw_cns; throw_defs += &throw_def; cns += &super::requires::finisher_done(m, sid);
        let freeze = if super::ultimate::is(&m.id) { super::ultimate::FREEZE } else { 55 };
        cns += &super::move_fx::super_pause_cns(m, sid, freeze);
        cns += &super::move_fx::move_fx_cns(m, sid, &anim_no);
        cns += &super::cinematic::move_cns(m, sid, &anim_no);
        if let Some(inv) = m.invuln { cns += &format!("[State {sid}, inv]\ntype = NotHitBy\ntrigger1 = AnimElemNo(0) = [{}, {}]\nvalue = SCA\ntime = 1\n\n", inv[0], inv[1]); }
        if let Some(tp) = &m.teleport { for step in tp.to_vec() { cns += &super::move_fx::teleport_cns(&step, sid); } }
        cns += &super::blocks::states(m, sid, &anim_no);
        if ph == 'N' && !matches!(when, "air" | "fly") {
            cns += &format!("[State {sid}, grav]\ntype = VelAdd\n{}trigger1 = Pos Y < 0 || Vel Y < 0\ny = const(movement.yaccel)\n\n{}[State {sid}, fix]\ntype = PosSet\ntrigger1 = Pos Y > 0\ny = 0\n\n", super::move_motion::own_gravity(m), super::move_fx::land_cns(m.land_at, sid, anim));
        }
        let sm = if super::ultimate::is(&m.id) { super::ultimate::DAMAGE } else if m.kind == "super" { 1.3 } else { 1.0 };
        let attr = format!("{st}, {}A", match m.kind.as_str() { "normal" => "N", "special" | "finisher" => "S", _ => "H" });
        for h in super::multihit::glue(&m.hits).iter() {
            cns += &format!("[State {sid}, hit]\ntype = HitDef\ntrigger1 = AnimElem = {}\n{}\n", h.at,
                extras::hitdef_from_spec(h, sm, &attr, false));
        }
        let mut pid = sid;
        for p in super::blocks::projectiles(m, nc.anims.get(&m.anim).map_or(0, |a| a.frames.len())) {
            let pa = anim_no.get(&p.anim).copied().unwrap_or(0);
            let ph2 = p.hit_anim.as_ref().and_then(|h| anim_no.get(h)).map(|h| format!("projhitanim = {h}\nprojremanim = {h}\n")).unwrap_or_default();
            let pattr = format!("{st}, {}P", if m.kind == "super" { "H" } else { "S" });
            cns += &format!("[State {sid}, proj]\ntype = Projectile\ntrigger1 = AnimElem = {}\nprojid = {pid}\nprojanim = {pa}\n{ph2}offset = {}, {}\nvelocity = {}, {}\n{}projhits = {}\nprojmisstime = {}\nprojpriority = {}\nprojremovetime = {}\n{}\n",
                p.at, p.offset[0], p.offset[1], p.speed[0], p.speed[1], p.accel.map(|a| format!("accel = {}, {}\n", a[0], a[1])).unwrap_or_default(),
                p.hits.unwrap_or(1), p.miss_time.unwrap_or(6), p.priority.unwrap_or(1), p.remove_time.unwrap_or(-1),
                extras::hitdef_projectile(&p, sm, super::multihit::projectile_knockback(&p), &pattr));
            pid += 1;
        }
        cns += &super::move_fx::follow_up_cns(m, sid, m.follow_up.as_ref().and_then(|f| plan.sid_of(&f.to)));
        cns += &super::hold_release::cns(m, sid, &plan, commands);
        if when != "air" {
            if m.teleport.is_some() && when != "fly" { cns += &format!("[State {sid}, end_air]\ntype = ChangeState\ntrigger1 = AnimTime = 0 && Pos Y < 0\nvalue = 50\nctrl = 1\n\n"); }
            cns += &format!("[State {sid}, end]\ntype = ChangeState\ntrigger1 = AnimTime = 0\nvalue = {}\nctrl = 1\n\n", match when { "crouch" => 11, "fly" => super::abilities::FLY_STATE, _ => 0 });
        }
        if listed && super::air_variant::eligible(m, when) {
            let (entry, body) = (super::air_variant::entry(&minus1[minus1_start..], sid, &format!("{state_cond}{crouch_ok}")), super::air_variant::body(&cns[cns_start..], sid));
            minus1 += &entry; cns += &body;
        }
    }
    cns += &throw_defs;
    (*minus1_out, *cns_out) = (minus1, cns);
    (movelist, move_states)
}
