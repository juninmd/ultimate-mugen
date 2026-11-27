//! UMF → CharDef compiler: generates commands, [Statedef -1], move states, HitDefs and projectiles as CNS.
use super::effects::{render_effect, FrameArt};
use super::types::*;
use ume_engine::chardef::{default_constants, AnimFrame, Animation, Blend, CharDef, MoveInfo, Source};
use ume_engine::cns::parse_states;
use ume_engine::commands::parse_command;
use serde_json::Value;
use std::collections::HashMap;
#[path = "compile_extras.rs"] pub(super) mod extras;

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

const BOSS_SPEED: f32 = 1.25;
const PACE_LOCKED: [i32; 4] = [0, 11, 20, 21];

/// Full compile (all animations).
pub fn compile_native(nc: &NativeChar, pal_no: i32, with_images: bool) -> CharDef { compile_native_with(nc, pal_no, with_images, None) }
/// Fast compile for menus (select grid, VS screen): rasterizes only idle + win animations (~10x cheaper than the full set).

fn frame_key(fr: &NativeFrame) -> String { serde_json::to_string(&(&fr.pose, &fr.effect, &fr.hit, &fr.sprite)).unwrap_or_default() }

pub(super) fn render_frame(nc: &NativeChar, fr: &NativeFrame, with_images: bool, palette: Option<&Value>) -> FrameArt {
    if let Some(mut art) = super::pixel::try_load(nc, fr, palette.filter(|_| with_images)) {
        if !with_images {
            art.image = None;
        }
        return art;
    }
    match &fr.effect {
        Some(effect) => render_effect(effect, with_images, palette),
        None => super::pixel_blank::boxes_only(nc, fr),
    }
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
    super::low_normals::complete(super::air_normals::complete(out))
}

pub fn compile_native_with(nc: &NativeChar, pal_no: i32, with_images: bool, only: Option<&[i32]>) -> CharDef {
    if let Some(spec) = &nc.mugen { return super::exact::compile_exact(nc, spec, pal_no, with_images, only); }
    let nc = &super::power_normals::with_power_normals(with_defaults(nc));
    let palette = if pal_no > 1 && !nc.palettes.is_empty() { Some(&nc.palettes[(pal_no as usize - 2) % nc.palettes.len()]) } else { None };
    let mut sprites = HashMap::new();
    let mut anims = HashMap::new();
    let anim_no = extras::anim_numbers(nc);
    let wanted = |no: i32| only.is_none_or(|o| o.contains(&no));
    // pass 1: unique frames to rasterize (in parallel)
    let mut seen = std::collections::HashSet::new();
    let mut jobs: Vec<(String, &NativeFrame)> = vec![];
    for (name, a) in nc.anims.iter() {
        if !wanted(anim_no[name]) { continue; }
        for fr in &a.frames { let k = frame_key(fr); if seen.insert(k.clone()) { jobs.push((k, fr)); } }
    }
    let variants = super::frame_raster::VariantPalettes::new(nc, palette, extras::awakening_palette(nc, palette).as_ref(), with_images);
    let mut rendered = super::frame_raster::raster_frames(nc, jobs, with_images, palette, &variants);
    let mut cache: HashMap<String, ((i32, i32), Vec<[f32; 4]>, Vec<[f32; 4]>)> = HashMap::new();
    let mut idx = 0;
    for (name, a) in nc.anims.iter() {
        let no = anim_no[name];
        if !wanted(no) { continue; }
        let mut frames = vec![];
        for fr in &a.frames {
            let key = frame_key(fr);
            let entry = cache.entry(key.clone()).or_insert_with(|| {
                let raster = rendered.remove(&key).expect("frame rendered");
                let (r, k) = (raster.base, (1, idx));
                idx += 1;
                for (group, variant) in raster.variants { sprites.insert((group, k.1), variant); }
                sprites.insert(k, super::frame_raster::sprite_of(&r));
                (k, r.hurt, r.hit)
            });
            let hurt = match &fr.hurt { Some(Value::String(s)) if s == "none" => vec![], Some(v @ Value::Array(_)) => serde_json::from_value(v.clone()).unwrap_or(entry.1.clone()), _ => entry.1.clone() };
            let hit = entry.2.clone();
            frames.push(AnimFrame {
                group: entry.0 .0, index: entry.0 .1, x: fr.offset.map(|o| o[0]).unwrap_or(0.0), y: fr.offset.map(|o| o[1]).unwrap_or(0.0), time: if fr.ticks > 0 && nc.speed.is_some_and(|k| k > 1.0 && (k > BOSS_SPEED || !PACE_LOCKED.contains(&no))) { ((fr.ticks as f32 / nc.speed.unwrap_or(1.0)).round() as i32).max(1) } else { fr.ticks },
                flip_h: false, flip_v: false, blend: if fr.effect.as_ref().is_some_and(|effect| !matches!(effect.get("type").and_then(Value::as_str), Some("dragon" | "getsuga"))) { Blend::Add } else { Blend::None }, alpha: None, xscale: 1.0, yscale: 1.0, angle: 0.0,
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
    if let Some(v) = s.air_jumps { set("movement.airjump.num", v); } if let Some(v) = s.air_dashes { set("movement.airdash.num", v); }
    if let Some(v) = s.height { set("size.height", v); }
    if let Some(w) = s.width { set("size.ground.front", w[0]); set("size.ground.back", w[1]); }
    if let Some(v) = s.guard_max { set("data.guardpoints", v); } if let Some(v) = s.dizzy_max { set("data.dizzypoints", v); }
    if let Some(v) = s.weight { set("movement.weight", v); } if let Some(v) = s.chip_defense { set("data.chip.defence", v); }

    let mut commands = vec![
        parse_command("holdfwd", "/$F", 1, 1), parse_command("holdback", "/$B", 1, 1), parse_command("holdup", "/$U", 1, 1), parse_command("holddown", "/$D", 1, 1),
        parse_command("FF", "F, F", 10, 1), parse_command("BB", "B, B", 10, 1), parse_command("UU", "U, U", 10, 1),
    ];
    let ab = super::abilities::ability_code(nc.abilities.as_ref(), &anim_no);
    commands.extend(ab.commands);
    let mut minus1 = String::from("[Statedef -1]\n\n") + &ab.minus1;
    for (command, input, state) in [("FF", "F,F", 100), ("BB", "B,B", 105)] {
        if nc.moves.iter().any(|m| m.kind == "movement" && m.input.replace(' ', "") == input) {
            continue;
        }
        minus1 += &format!("[State -1, {command}]\ntype = ChangeState\nvalue = {state}\ntriggerall = !IsHelper && command = \"{command}\"\ntrigger1 = statetype = S && ctrl\n\n");
    }
    let mut cns = String::new();
    let (mut movelist, move_states) = super::compile_moves::compile_moves(nc, &anim_no, 0, &mut commands, &mut minus1, &mut cns);
    cns += &ab.states; movelist.extend(ab.movelist);
    let mut states = HashMap::new();
    let mut minus2 = extras::passives_cns(nc) + &ab.minus2;
    extras::append_exotic(nc, &anim_no, &move_states, &mut commands, &mut movelist, &mut minus1, &mut minus2, &mut cns);
    parse_states(&(minus1 + &minus2 + &cns), &mut states, true);
    super::scripts::compile_scripts(nc.scripts.as_ref(), &anim_no, &move_states, &mut states);
    if with_images {
        if let Some(avatar) = super::portrait::portrait(nc, palette) { sprites.insert((9000, 1), avatar); }
    }
    let portrait = sprites.get(&(9000, 1)).cloned().or_else(|| anims.get(&0).and_then(|a: &Animation| a.frames.first()).and_then(|f| sprites.get(&(f.group, f.index)).cloned()));
    let hex = nc.color.trim_start_matches('#');
    let cv = u32::from_str_radix(hex, 16).unwrap_or(0x5a648c);
    let mut d = CharDef::empty(&format!("umf:{}", nc.id));
    d.name = nc.name.clone(); d.display_name = nc.name.clone(); d.author = nc.author.clone(); d.source = Source::Native;
    d.constants = k; d.refresh_sizes(); d.sprites = sprites; d.anims = anims; d.commands = commands; d.states = states; d.portrait = portrait;
    d.ai_commands = movelist.iter().map(|m: &MoveInfo| m.command.clone()).collect(); d.movelist = movelist;
    d.color = [(cv >> 16) as u8, (cv >> 8) as u8, cv as u8]; d.series = nc.series.clone(); d.element = nc.element; d.description = nc.description.clone().unwrap_or_default();
    d.intro = nc.intro.clone().unwrap_or_default(); d.taunt = nc.taunt.clone().unwrap_or_default(); d.win_quote = nc.win_quote.clone().unwrap_or_default();
    d.form_name = nc.form.clone().unwrap_or_default(); d.forms = super::forms::build_forms(nc, pal_no, with_images, only); d.boss = nc.boss; d.passives = nc.passives.as_ref().map(|p| p.rules()).unwrap_or_default(); d.rivals = ume_engine::rivals::Rivals::new(&nc.id, &nc.rivals); extras::insert_voices(&mut d, nc);
    if let Some(p) = &nc.profile { d.element = d.element.or_else(|| p.element.as_deref().and_then(ume_engine::element::Element::parse)); d.weakness = p.weakness.as_deref().and_then(ume_engine::element::Element::parse); }
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
