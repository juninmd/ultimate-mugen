use super::{render_frame, CharDef, HashMap, HitSpec, NativeChar, NativeFrame, NativeMove, Sprite, Value};

pub(super) struct Lv { pub stop: i32, pub stun: i32, pub kb: f32, pub anim: &'static str }
pub(super) fn level(l: Option<&str>) -> Lv {
    match l.unwrap_or("medium") {
        "light" => Lv { stop: 9, stun: 12, kb: -3.5, anim: "Light" },
        "heavy" => Lv { stop: 13, stun: 20, kb: -6.0, anim: "Hard" },
        _ => Lv { stop: 11, stun: 16, kb: -4.5, anim: "Medium" },
    }
}
pub(super) fn guardflag(g: Option<&str>) -> &'static str { match g.unwrap_or("mid") { "high" => "HA", "low" => "L", "unblockable" => "", _ => "MA" } }

pub(super) fn hitdef_from_spec(h: &HitSpec, sm: f32, attr: &str, is_proj: bool) -> String {
    let damage = h.damage * sm;
    let gd = h.guard_damage.map(|g| g * sm).unwrap_or((damage / 8.0).round());
    let lv = level(h.level.as_deref());
    let kb = h.knockback.unwrap_or([lv.kb, 0.0]);
    let launch = h.launch.unwrap_or(false);
    let air = h.air_knockback.unwrap_or([kb[0] * 0.8, if launch { -6.0 } else { -3.5 }]);
    let stop = h.hitstop.unwrap_or(lv.stop);
    let stun = h.hitstun.unwrap_or(lv.stun);
    let spark = h.spark.unwrap_or(match h.level.as_deref() { Some("heavy") => 2, Some("light") => 0, _ => 1 });
    let prio = h.priority.unwrap_or(4);
    let hflag = h.hitflag.as_deref().unwrap_or("MAF");
    let (p1stop, p2stop) = h.pausetime.map(|p| (p[0], p[1])).unwrap_or((if is_proj { 0 } else { stop }, stop));
    let (hs_g, hs_i) = h.hit_sound.map(|s| (s[0], s[1])).unwrap_or((5, spark));
    let (gs_g, gs_i) = h.guard_sound.map(|s| (s[0], s[1])).unwrap_or((6, 0));
    let (sp_x, sp_y) = h.spark_pos.map(|s| (s[0], s[1])).unwrap_or((0.0, 0.0));

    let mut s = format!(
        "attr = {attr}\ndamage = {damage}, {gd}\nanimtype = {}\nguardflag = {}\nhitflag = {hflag}\npriority = {prio}, Hit\npausetime = {p1stop}, {p2stop}\nguard.pausetime = {}, {}\nsparkno = {spark}\nguard.sparkno = 40\nsparkxy = {sp_x}, {sp_y}\nhitsound = {hs_g}, {hs_i}\nguardsound = {gs_g}, {gs_i}\nground.type = {}\nground.slidetime = {}\nground.hittime = {stun}\nground.velocity = {}, {}\nair.velocity = {}, {}\nair.hittime = {}\nguard.velocity = {}\nfall = {}\nair.fall = {}\nfall.recover = 1\ngetpower = {}, {}\ngivepower = {}, {}\n",
        lv.anim, guardflag(h.guard.as_deref()), if is_proj { 0 } else { stop - 2 }, stop - 2,
        if h.guard.as_deref() == Some("low") { "Low" } else { "High" }, (stun as f32 * 0.6).round() as i32,
        kb[0], if launch { if kb[1] != 0.0 { kb[1] } else { -7.0 } } else { kb[1] }, air[0], air[1], stun + 6, kb[0] * 0.7,
        launch as i32, (launch || h.level.as_deref() == Some("heavy")) as i32,
        (damage * 1.2).round(), (damage * 0.5).round(), (damage * 0.6).round(), (damage * 0.3).round());
    if let Some(t) = h.shake { s += &format!("envshake.time = {t}\n"); }
    if let Some(cp) = h.cornerpush { s += &format!("ground.cornerpush.veloff = {cp}\n"); }
    if h.ground_bounce.unwrap_or(false) { s += "fall.damage = 15\nfall.envshake.time = 8\ndown.bounce = 1\n"; }
    s
}

pub(super) fn hitdef_projectile(damage: f32, lvl: Option<&str>, kb: Option<[f32; 2]>, launch: bool, attr: &str) -> String {
    let spec = HitSpec {
        at: 0, damage, guard_damage: None, level: lvl.map(|s| s.into()), guard: None, knockback: kb, air_knockback: None,
        launch: Some(launch), hitstop: None, hitstun: None, spark: None, shake: None, priority: Some(4), hitflag: None,
        pausetime: None, cornerpush: None, hit_sound: None, guard_sound: None, spark_pos: None, ground_bounce: None,
    };
    hitdef_from_spec(&spec, 1.0, attr, true)
}

pub(super) fn passives_cns(nc: &NativeChar) -> String {
    let mut out = String::from("[Statedef -2]\n\n");
    if let Some(pass) = &nc.passives {
        if let Some(mr) = pass.meter_regen {
            if mr > 0.0 { out += &format!("[State -2, meter_regen]\ntype = PowerAdd\ntrigger1 = 1\nvalue = {mr}\n\n"); }
        }
        if let Some(lr) = pass.life_regen {
            if lr != 0.0 { out += &format!("[State -2, life_regen]\ntype = LifeAdd\ntrigger1 = 1\nvalue = {lr}\n\n"); }
        }
    }
    out
}

pub(super) fn voice_state(m: &NativeMove, sid: i32) -> String {
    let Some((group, index)) = m.voice.as_deref().and_then(|voice| {
        let (group, index) = voice.split_once(',')?;
        Some((group.trim().parse::<u16>().ok()?, index.trim().parse::<u16>().ok()?))
    }) else { return String::new() };
    format!("[State {sid}, voice]\ntype = PlaySnd\ntrigger1 = Time = 0\nvalue = {group}, {index}\nchannel = 4\nstopongethit = 1\n\n")
}

pub(super) fn voice_and_traits_state(m: &NativeMove, sid: i32) -> String {
    let mut out = voice_state(m, sid);
    if m.armor.is_some() {
        out += &format!("[State {sid}, armor]\ntype = HitOverride\ntrigger1 = 1\nattr = SCA,AA,AP,AT\nslot = 0\nstateno = {sid}\ntime = 2\nforceair = 0\n\n");
    }
    if let Some(p) = &m.parry {
        let sp = p.spark.unwrap_or(2);
        out += &format!("[State {sid}, parry]\ntype = ReversalDef\ntrigger1 = AnimElemNo(0) >= {} && AnimElemNo(0) <= {}\nreversal.attr = SCA,NA,SA,HA,NP,SP,HP\npausetime = 15, 15\nsparkno = {sp}\nhitsound = 5, 0\n\n", p.window[0], p.window[1]);
    }
    if let Some(ticks) = m.timestop {
        if ticks > 0 {
            out += &format!("[State {sid}, timestop]\ntype = Pause\ntrigger1 = AnimElem = 1\ntime = {ticks}\nmovetime = {ticks}\npausebg = 1\n\n");
        }
    }
    if m.burst == Some(true) {
        out += &format!("[State {sid}, burst_inv]\ntype = NotHitBy\ntrigger1 = 1\nvalue = SCA\ntime = 1\n\n[State {sid}, burst_push]\ntype = VelSet\ntrigger1 = Time = 0\nx = 0\ny = 0\n\n");
    }
    if m.flight == Some(true) {
        out += &format!("[State {sid}, flight_hover]\ntype = VelSet\ntrigger1 = 1\nx = ifelse(command = \"holdfwd\", 3, ifelse(command = \"holdback\", -3, 0))\ny = ifelse(command = \"holdup\", -3, ifelse(command = \"holddown\", 3, 0))\n\n");
    }
    if let Some(th) = &m.throw {
        let p2st = th.p2_state.unwrap_or(820);
        out += &format!("[State {sid}, throw_hit]\ntype = HitDef\ntrigger1 = AnimElem = 1\nattr = S, NT\nhitflag = M-\npriority = 7, Miss\nsparkno = -1\np1stateno = {sid}\np2stateno = {p2st}\n\n");
        if let Some(b) = th.bind {
            out += &format!("[State {sid}, throw_bind]\ntype = TargetBind\ntrigger1 = AnimElem = 2\npos = {}, {}\n\n", b[0], b[1]);
        }
        out += &format!("[State {sid}, throw_dmg]\ntype = TargetLifeAdd\ntrigger1 = AnimElem = 3\nvalue = -{}\nkill = 1\n\n", th.damage as i32);
    }
    out
}

pub(super) fn awakening_palette(nc: &NativeChar, palette: Option<&Value>) -> Option<Value> {
    let awake = nc.awakening_palette.as_ref()?;
    let mut colors = palette.and_then(|p| p.get("colors").unwrap_or(p).as_object()).cloned().unwrap_or_default();
    colors.extend(awake.get("colors").unwrap_or(awake).as_object()?.clone());
    colors.insert("_awakening".into(), Value::Bool(true));
    Some(Value::Object(colors))
}

pub(super) fn insert_awakened(sprites: &mut HashMap<(i32, i32), Sprite>, nc: &NativeChar, fr: &NativeFrame, index: i32, with_images: bool, palette: Option<&Value>) {
    if !with_images || fr.effect.is_some() { return; }
    if let Some(palette) = palette {
        let awake = render_frame(nc, fr, true, Some(palette));
        sprites.insert((2, index), Sprite { image: awake.image, width: awake.width, height: awake.height, axis_x: awake.axis_x, axis_y: awake.axis_y, scale: awake.scale });
    }
}

pub(super) fn insert_voices(d: &mut CharDef, nc: &NativeChar) {
    if nc.id != "kaito" { return; }
    d.sounds.insert((1000, 0), include_bytes!("../../../../assets/chars/voices/kaito/dragon.wav").to_vec());
    d.sounds.insert((1000, 1), include_bytes!("../../../../assets/chars/voices/kaito/rush.wav").to_vec());
    d.sounds.insert((1000, 2), include_bytes!("../../../../assets/chars/voices/kaito/gale.wav").to_vec());
    d.sounds.insert((1000, 3), include_bytes!("../../../../assets/chars/voices/kaito/awake.wav").to_vec());
}
