//! UMF → CharDef compiler: generates commands, [Statedef -1], move states, HitDefs and projectiles as CNS.
use super::rig::{render_effect, render_pose, RigRender};
use super::types::*;
use crate::chardef::{default_constants, AnimFrame, Animation, Blend, CharDef, MoveInfo, Source, Sprite};
use crate::cns::parse_states;
use crate::commands::parse_command;
use serde_json::Value;
use std::collections::HashMap;
#[path = "compile_extras.rs"] mod extras;

pub const STANDARD_ANIMS: &[(&str, i32)] = &[
    ("idle", 0), ("turn", 5), ("crouchTurn", 6), ("crouchStart", 10), ("crouch", 11), ("crouchEnd", 12), ("walk", 20), ("walkBack", 21),
    ("jumpStart", 40), ("jumpUp", 41), ("jumpFwd", 42), ("jumpBack", 43), ("jumpDown", 44), ("land", 47), ("run", 100), ("hop", 105),
    ("guardStart", 120), ("crouchGuardStart", 121), ("airGuardStart", 122), ("guard", 130), ("crouchGuard", 131), ("airGuard", 132),
    ("guardEnd", 140), ("guardHit", 150), ("crouchGuardHit", 152), ("airGuardHit", 154), ("lose", 170), ("draw", 175), ("win", 181), ("intro", 190),
    ("hitLight", 5000), ("hitMedium", 5001), ("hitHeavy", 5002), ("hitLowLight", 5010), ("hitLowMedium", 5011), ("hitLowHeavy", 5012),
    ("hitCrouch", 5020), ("hitAir", 5030), ("fall", 5050), ("bounce", 5100), ("bounceAir", 5101), ("lieDown", 5110), ("getUp", 5120), ("dead", 5150),
    ("trip", 5070), ("airRecover", 5040),
    ("taunt", 195), ("powerUp", 196), ("defeat1", 197), ("defeat2", 198), ("defeat3", 199), ("defeat4", 200),
];

/// Full compile (all animations).
pub fn compile_native(nc: &NativeChar, pal_no: i32, with_images: bool) -> CharDef { compile_native_with(nc, pal_no, with_images, None) }
/// Fast compile for menus (select grid, VS screen): rasterizes only idle + win animations (~10x cheaper than the full set).
pub fn compile_native_preview(nc: &NativeChar, pal_no: i32) -> CharDef { compile_native_with(nc, pal_no, true, Some(&[0, 181])) }

fn frame_key(fr: &NativeFrame) -> String { serde_json::to_string(&(&fr.pose, &fr.effect, &fr.hit, &fr.sprite)).unwrap_or_default() }

pub(super) fn render_frame(nc: &NativeChar, fr: &NativeFrame, with_images: bool, palette: Option<&Value>) -> RigRender {
    let force_rig = std::env::var("UME_FORCE_RIG").is_ok();
    if !force_rig && with_images {
        if let Some(r) = super::pixel::try_load(nc, fr, palette) {
            return r;
        }
    }
    if nc.art.kind != "rig" {
        RigRender { image: None, width: 0.0, height: 0.0, axis_x: 0.0, axis_y: 0.0, scale: 1.0, hurt: vec![], hit: vec![] }
    } else if let Some(e) = &fr.effect {
        render_effect(e, with_images, palette)
    } else {
        let pose = match &fr.pose { Some(Value::String(s)) => nc.art.poses.get(s).cloned().unwrap_or(Value::Null), Some(v) => v.clone(), None => Value::Null };
        render_pose(&nc.art.style, &pose, fr.hit.as_ref(), with_images, palette)
    }
}

/// Rasterize every unique frame, spread over all cores on native targets (single-threaded on wasm).
fn render_all(nc: &NativeChar, jobs: Vec<(String, &NativeFrame)>, with_images: bool, palette: Option<&Value>) -> HashMap<String, RigRender> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).min(jobs.len().max(1)).min(8);
        if n > 1 {
            let next = std::sync::atomic::AtomicUsize::new(0);
            let out = std::sync::Mutex::new(HashMap::with_capacity(jobs.len()));
            std::thread::scope(|sc| {
                for _ in 0..n {
                    sc.spawn(|| loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some((k, fr)) = jobs.get(i) else { break };
                        let r = render_frame(nc, fr, with_images, palette);
                        out.lock().unwrap().insert(k.clone(), r);
                    });
                }
            });
            return out.into_inner().unwrap();
        }
    }
    jobs.into_iter().map(|(k, fr)| { let r = render_frame(nc, fr, with_images, palette); (k, r) }).collect()
}

/// Shared poses/animations every native character gets when it does not define its own: taunt, power-up (transformation)
/// and the four defeat sequences (see `defaults.json`).
pub(super) fn with_defaults(nc: &NativeChar) -> NativeChar {
    let d: Value = serde_json::from_str(include_str!("defaults.json")).unwrap_or(Value::Null);
    let mut out = nc.clone();
    if let Some(ps) = d.get("poses").and_then(|v| v.as_object()) { for (k, v) in ps { out.art.poses.entry(k.clone()).or_insert_with(|| v.clone()); } }
    if let Some(an) = d.get("anims").and_then(|v| v.as_object()) {
        for (k, v) in an {
            if out.anims.get(k).is_none() { if let Ok(a) = serde_json::from_value::<NativeAnim>(v.clone()) { out.anims.0.push((k.clone(), a)); } }
        }
    }
    out
}

pub fn compile_native_with(nc: &NativeChar, pal_no: i32, with_images: bool, only: Option<&[i32]>) -> CharDef {
    let nc = &with_defaults(nc);
    let palette = if pal_no > 1 && !nc.palettes.is_empty() { Some(&nc.palettes[(pal_no as usize - 2) % nc.palettes.len()]) } else { None };
    let mut sprites = HashMap::new();
    let mut anims = HashMap::new();
    let mut anim_no: HashMap<String, i32> = HashMap::new();
    let mut next = 1000;
    for (name, _) in nc.anims.iter() {
        let no = STANDARD_ANIMS.iter().find(|(n, _)| n == name).map(|(_, v)| *v).unwrap_or_else(|| { next += 1; next - 1 });
        anim_no.insert(name.clone(), no);
    }
    let wanted = |no: i32| only.is_none_or(|o| o.contains(&no));
    // pass 1: unique frames to rasterize (in parallel)
    let mut seen = std::collections::HashSet::new();
    let mut jobs: Vec<(String, &NativeFrame)> = vec![];
    for (name, a) in nc.anims.iter() {
        if !wanted(anim_no[name]) { continue; }
        for fr in &a.frames { let k = frame_key(fr); if seen.insert(k.clone()) { jobs.push((k, fr)); } }
    }
    let mut rendered = render_all(nc, jobs, with_images, palette); let awakening_palette = extras::awakening_palette(nc, palette);
    let mut cache: HashMap<String, ((i32, i32), Vec<[f32; 4]>, Vec<[f32; 4]>)> = HashMap::new();
    let mut idx = 0;
    for (name, a) in nc.anims.iter() {
        let no = anim_no[name];
        if !wanted(no) { continue; }
        let mut frames = vec![];
        for fr in &a.frames {
            let key = frame_key(fr);
            let entry = cache.entry(key.clone()).or_insert_with(|| {
                let r = rendered.remove(&key).expect("frame rendered");
                let k = (1, idx);
                idx += 1;
                sprites.insert(k, Sprite { image: r.image, width: r.width, height: r.height, axis_x: r.axis_x, axis_y: r.axis_y, scale: r.scale }); extras::insert_awakened(&mut sprites, nc, fr, k.1, with_images, awakening_palette.as_ref());
                super::forms::insert_form_sprites(&mut sprites, nc, fr, k.1, with_images, palette);
                (k, r.hurt, r.hit)
            });
            let hurt = match &fr.hurt { Some(Value::String(s)) if s == "none" => vec![], Some(v @ Value::Array(_)) => serde_json::from_value(v.clone()).unwrap_or(entry.1.clone()), _ => entry.1.clone() };
            let hit = fr.hit.as_ref().and_then(|h| h.bx).map(|b| vec![b]).unwrap_or(entry.2.clone());
            frames.push(AnimFrame {
                group: entry.0 .0, index: entry.0 .1, x: fr.offset.map(|o| o[0]).unwrap_or(0.0), y: fr.offset.map(|o| o[1]).unwrap_or(0.0), time: if fr.ticks > 0 && nc.speed.is_some_and(|k| k > 1.0) { ((fr.ticks as f32 / nc.speed.unwrap_or(1.0)).round() as i32).max(1) } else { fr.ticks },
                flip_h: false, flip_v: false, blend: if fr.effect.as_ref().is_some_and(|effect| effect.get("type").and_then(Value::as_str) != Some("dragon")) { Blend::Add } else { Blend::None }, alpha: None, xscale: 1.0, yscale: 1.0, angle: 0.0,
                clsn2: if fr.effect.is_some() { hit.clone() } else { hurt }, clsn1: hit,
            });
        }
        let mut an = Animation { id: no, frames, loop_start: a.loop_start.unwrap_or(0), total_time: 0 };
        an.compute_total();
        anims.insert(no, an);
    }

    let mut k = default_constants();
    let s = &nc.stats;
    let mut set = |key: &str, v: f64| { k.insert(key.into(), v); };
    if let Some(v) = s.life { set("data.life", v); }
    if let Some(v) = s.power { set("data.power", v); }
    if let Some(v) = s.attack { set("data.attack", v); }
    if let Some(v) = s.defense { set("data.defence", v); }
    if let Some(w) = s.walk { set("velocity.walk.fwd.x", w[0]); set("velocity.walk.back.x", -w[1].abs()); }
    if let Some(r) = s.run { set("velocity.run.fwd.x", r[0]); set("velocity.run.back.x", -r[1].abs()); }
    if let Some(j) = s.jump { set("velocity.jump.y", j[0]); set("velocity.jump.back.x", j[1]); set("velocity.jump.neu.x", j[2]); set("velocity.jump.fwd.x", j[3]); set("velocity.airjump.y", j[0] * 0.9); }
    if let Some(v) = s.gravity { set("movement.yaccel", v); }
    if let Some(v) = s.air_jumps { set("movement.airjump.num", v); }
    if let Some(v) = s.height { set("size.height", v); }
    if let Some(w) = s.width { set("size.ground.front", w[0]); set("size.ground.back", w[1]); }

    let mut commands = vec![
        parse_command("holdfwd", "/$F", 1, 1), parse_command("holdback", "/$B", 1, 1), parse_command("holdup", "/$U", 1, 1), parse_command("holddown", "/$D", 1, 1),
        parse_command("FF", "F, F", 10, 1), parse_command("BB", "B, B", 10, 1),
    ];
    let ab = super::abilities::ability_code(nc.abilities.as_ref(), &anim_no);
    commands.extend(ab.commands);
    let mut minus1 = String::from("[Statedef -1]\n\n[State -1, run]\ntype = ChangeState\nvalue = 100\ntriggerall = command = \"FF\"\ntrigger1 = statetype = S && ctrl\n\n[State -1, hop]\ntype = ChangeState\nvalue = 105\ntriggerall = command = \"BB\"\ntrigger1 = statetype = S && ctrl\n\n") + &ab.minus1;
    let mut cns = String::new();
    let rank = |m: &NativeMove| (match m.kind.as_str() { "super" => 300, "special" => 200, _ => 100 }) + m.input.split(',').count() as i32 * 5 + m.input.contains('+') as i32 * 2;
    let mut order = super::forms::collect_form_moves(nc);
    order.sort_by_key(|(m, _, _)| -rank(m));
    let plan = super::chain::ChainPlan::new(order.iter().map(|(m, _, _)| *m));
    let mut movelist = vec![];
    let mut throw_defs = String::new();
    for (index, (m, form_cond, forms_mask)) in order.into_iter().enumerate() {
        let when = m.when.as_deref().unwrap_or("stand");
        let sid = plan.sid(index);
        let (cns_start, minus1_start) = (cns.len(), minus1.len());
        let cname = format!("m_{}", m.id);
        let simple_btn = m.input.trim().split('+').all(|t| ["a", "b", "c", "x", "y", "z"].contains(&t.trim()));
        commands.push(parse_command(&cname, &m.input, if m.kind == "super" { 30 } else if m.kind == "special" { 20 } else { 15 }, if simple_btn { 1 } else { 2 }));
        let cost = m.cost.unwrap_or(if m.kind == "super" { 1000.0 } else { 0.0 });
        movelist.push(MoveInfo { name: m.name.clone(), input: m.input.clone(), command: cname.clone(), cost, drain: false, forms: forms_mask });
        let mut simple_cmd = String::new();
        if let Some(sp) = &m.simple {
            let sc = format!("simple_{sp}_{}", m.id);
            let inp = match sp.as_str() { "sp1" => "d", "sp2" => "w", "sp3" => "e", _ => "d+w" };
            commands.push(parse_command(&sc, inp, 1, 2));
            simple_cmd = format!(" || command = \"{sc}\"");
        }
        let air_entry = format!("statetype = A{}", plan.air_entry_states(index));
        let state_cond = match when { "air" => air_entry.as_str(), "crouch" => "statetype = C || (statetype = S && command = \"holddown\")", _ => "statetype = S || statetype = C && command != \"holddown\"" };
        let crouch_ok = match when { "crouch" => "\ntriggerall = command = \"holddown\"", "stand" => "\ntriggerall = command != \"holddown\" || statetype = S", _ => "" };
        let cancel = plan.triggers(index);
        let form_line = form_cond.as_ref().map(|fc| format!("triggerall = {fc}\n")).unwrap_or_default();
        let cost_line = if cost > 0.0 { format!("triggerall = power >= {cost}\n") } else { String::new() };
        minus1 += &format!("[State -1, {}]\ntype = ChangeState\nvalue = {sid}\ntriggerall = command = \"{cname}\"{simple_cmd}\ntriggerall = {state_cond}{crouch_ok}\n{form_line}{cost_line}trigger1 = ctrl{cancel}{}\n\n", m.name, if m.burst == Some(true) { "\ntrigger2 = movetype = H && stateno != [5100, 5150]" } else { "" });
        let anim = anim_no.get(&m.anim).copied().unwrap_or(0);
        let st = match when { "air" => 'A', "crouch" => 'C', _ => 'S' };
        let jumps = m.moves.iter().any(|v| v.vel[1].is_some_and(|y| y != 0.0));
        let ph = match when { "air" => 'A', "crouch" => 'C', _ => if jumps || m.flight == Some(true) { 'N' } else { 'S' } };
        let pa = if cost > 0.0 { -cost } else if m.kind == "special" { 30.0 } else { 15.0 };
        cns += &format!("[Statedef {sid}]\ntype = {st}\nmovetype = A\nphysics = {ph}\nanim = {anim}\nctrl = 0\n{}poweradd = {pa}\nsprpriority = 2\n\n", if when == "air" { "" } else { "velset = 0, 0\n" }); cns += &super::armor::armor_state(nc, m, sid); cns += &extras::voice_and_traits_state(m, sid); let (throw_cns, throw_def) = super::throw::throw_states(m, sid); cns += &throw_cns; throw_defs += &throw_def;
        if m.kind == "super" {
            cns += &format!("[State {sid}, super]\ntype = SuperPause\ntrigger1 = Time = 0\ntime = 55\nanim = 30\npos = 0, -60\nsound = -1\n\n[State {sid}, ai]\ntype = AfterImage\ntrigger1 = Time = 0\nframegap = 2\npalbright = 0, 0, 0\npalcontrast = 100, 150, 256\npaladd = 0, 0, 0\npalmul = .85, .85, .85\ntime = 60\nlength = 10\n\n");
        } else if m.fx.as_deref() == Some("aura") {
            cns += &format!("[State {sid}, ai]\ntype = AfterImage\ntrigger1 = Time = 0\nframegap = 2\npalbright = 0, 0, 0\npalcontrast = 100, 150, 256\npaladd = 0, 0, 0\npalmul = .85, .85, .85\ntime = 20\nlength = 6\n\n");
        }
        if let Some(inv) = m.invuln { cns += &format!("[State {sid}, inv]\ntype = NotHitBy\ntrigger1 = AnimElemNo(0) = [{}, {}]\nvalue = SCA\ntime = 1\n\n", inv[0], inv[1]); }
        if let Some(tp) = m.teleport { cns += &format!("[State {sid}, tp]\ntype = PosAdd\ntrigger1 = AnimElem = {tp}\nx = p2dist x + 40\n\n[State {sid}, tpt]\ntype = Turn\ntrigger1 = AnimElem = {tp}\n\n"); }
        for v in &m.moves {
            cns += &format!("[State {sid}, vel]\ntype = VelSet\ntrigger1 = AnimElem = {}\n{}{}\n", v.at, v.vel[0].map(|x| format!("x = {x}\n")).unwrap_or_default(), v.vel[1].map(|y| format!("y = {y}\n")).unwrap_or_default());
        }
        if ph == 'N' && when != "air" {
            cns += &format!("[State {sid}, grav]\ntype = VelAdd\ntrigger1 = Pos Y < 0 || Vel Y < 0\ny = const(movement.yaccel)\n\n[State {sid}, land]\ntype = ChangeState\ntrigger1 = Pos Y >= 0 && Vel Y > 0\nvalue = 52\n\n[State {sid}, fix]\ntype = PosSet\ntrigger1 = Pos Y > 0\ny = 0\n\n");
        }
        // supers hit 30 % harder than their data (grander, deadlier finishers)
        let sm = if m.kind == "super" { 1.3 } else { 1.0 };
        let attr = format!("{st}, {}A", match m.kind.as_str() { "normal" => "N", "special" => "S", _ => "H" });
        for h in super::multihit::glue(&m.hits).iter() {
            cns += &format!("[State {sid}, hit]\ntype = HitDef\ntrigger1 = AnimElem = {}\n{}\n", h.at,
                extras::hitdef_from_spec(h, sm, &attr, false));
        }
        let mut pid = sid;
        for p in m.projectile.as_ref().map(|p| p.to_vec()).unwrap_or_default() {
            let pa = anim_no.get(&p.anim).copied().unwrap_or(0);
            let ph2 = p.hit_anim.as_ref().and_then(|h| anim_no.get(h)).map(|h| format!("projhitanim = {h}\nprojremanim = {h}\n")).unwrap_or_default();
            let pattr = format!("{st}, {}P", if m.kind == "super" { "H" } else { "S" });
            cns += &format!("[State {sid}, proj]\ntype = Projectile\ntrigger1 = AnimElem = {}\nprojid = {pid}\nprojanim = {pa}\n{ph2}offset = {}, {}\nvelocity = {}, {}\n{}projhits = {}\nprojmisstime = 6\nprojpriority = {}\nprojremovetime = {}\n{}\n",
                p.at, p.offset[0], p.offset[1], p.speed[0], p.speed[1], p.accel.map(|a| format!("accel = {}, {}\n", a[0], a[1])).unwrap_or_default(),
                p.hits.unwrap_or(1), p.priority.unwrap_or(1), p.remove_time.unwrap_or(-1),
                extras::hitdef_projectile(p.damage * sm, Some(p.level.as_deref().unwrap_or("medium")), super::multihit::projectile_knockback(&p), p.launch.unwrap_or(false), &pattr));
            pid += 1;
        }
        if when != "air" { cns += &format!("[State {sid}, end]\ntype = ChangeState\ntrigger1 = AnimTime = 0\nvalue = {}\nctrl = 1\n\n", if when == "crouch" { 11 } else { 0 }); }
        if super::air_variant::eligible(m, when) {
            let (entry, body) = (super::air_variant::entry(&minus1[minus1_start..], sid, &format!("{state_cond}{crouch_ok}")), super::air_variant::body(&cns[cns_start..], sid));
            minus1 += &entry; cns += &body;
        }
    }
    cns += &throw_defs; cns += &ab.states; movelist.extend(ab.movelist);
    let mut states = HashMap::new();
    let minus2 = extras::passives_cns(nc) + &ab.minus2;
    parse_states(&(minus1 + &minus2 + &cns), &mut states, true);
    if with_images && std::env::var("UME_FORCE_RIG").is_err() {
        if let Some(avatar) = super::portrait::portrait(nc, palette) { sprites.insert((9000, 1), avatar); }
    }
    let portrait = sprites.get(&(9000, 1)).cloned().or_else(|| anims.get(&0).and_then(|a: &Animation| a.frames.first()).and_then(|f| sprites.get(&(f.group, f.index)).cloned()));
    let hex = nc.color.trim_start_matches('#');
    let cv = u32::from_str_radix(hex, 16).unwrap_or(0x5a648c);
    let mut d = CharDef::empty(&format!("umf:{}", nc.id));
    d.name = nc.name.clone(); d.display_name = nc.name.clone(); d.author = nc.author.clone(); d.source = Source::Native;
    d.constants = k; d.refresh_sizes(); d.sprites = sprites; d.anims = anims; d.commands = commands; d.states = states; d.portrait = portrait;
    d.ai_commands = movelist.iter().map(|m: &MoveInfo| m.command.clone()).collect(); d.movelist = movelist;
    d.color = [(cv >> 16) as u8, (cv >> 8) as u8, cv as u8]; d.series = nc.series.clone(); d.description = nc.description.clone().unwrap_or_default();
    d.intro = nc.intro.clone().unwrap_or_default(); d.taunt = nc.taunt.clone().unwrap_or_default(); d.win_quote = nc.win_quote.clone().unwrap_or_default();
    d.form_name = nc.form.clone().unwrap_or_default(); d.forms = super::forms::build_forms(nc, pal_no, with_images); d.boss = nc.boss; extras::insert_voices(&mut d, nc);
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Light/medium/heavy hit-stop follows the genre norm (~9/11/13 frames across games; SF V uses 8/12/15):
    /// heavier hits must freeze longer and the values stay in the researched band.
    #[test]
    fn hitstop_scales_with_attack_strength() {
        let (l, m, h) = (extras::level(Some("light")), extras::level(Some("medium")), extras::level(Some("heavy")));
        assert!(l.stop < m.stop && m.stop < h.stop);
        assert_eq!((l.stop, m.stop, h.stop), (9, 11, 13));
        assert!(l.stun < m.stun && m.stun < h.stun, "hit-stun also scales");
    }
}
