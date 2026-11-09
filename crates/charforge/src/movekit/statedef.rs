use super::controller::Ctrl;
use super::hitdef::{hit_controller, hit_elements};
use super::kinds::{is_air, power_cost};
use super::value::{get_bool, get_int, get_table, get_text, list, pair, tables, text};
use crate::movekit::animation::hit_frames;
use toml::Table;

fn kind_of(move_: &Table) -> String {
    get_text(move_, "kind").unwrap_or_else(|| "normal".into())
}

fn header(move_: &Table, number: i64, anim: i64) -> String {
    let kind = kind_of(move_);
    let helper = get_bool(move_, "helper").unwrap_or(false);
    let air = is_air(&kind);
    let crouch = kind == "crouch";
    let stance = get_text(move_, "type").unwrap_or_else(|| (if air { "A" } else if crouch { "C" } else { "S" }).into());
    let physics = get_text(move_, "physics").unwrap_or_else(|| (if helper { "N" } else if air { "A" } else if crouch { "C" } else { "S" }).into());
    let mut lines = vec![
        format!("[Statedef {number}]"), format!("type = {stance}"), "movetype = A".into(), format!("physics = {physics}"),
        "juggle = 1".into(), "ctrl = 0".into(), format!("anim = {anim}"), format!("sprpriority = {}", get_int(move_, "sprpriority", 2)),
    ];
    if !air && !helper {
        lines.push("facep2 = 1".into());
    }
    if move_.contains_key("velset") {
        lines.push(format!("velset = {}", pair(move_.get("velset"), "0, 0")));
    }
    let cost = get_int(move_, "power", power_cost(&kind));
    if cost != 0 && !move_.contains_key("_phase_of") {
        lines.push(format!("poweradd = {}", -cost.abs()));
    }
    lines.join("\n") + "\n\n"
}

fn super_pause(move_: &Table, defaults: &Table) -> String {
    let spec = get_table(move_, "superpause").cloned().unwrap_or_default();
    Ctrl::new("SuperPause", "super pause")
        .field("trigger1", "!time")
        .field("time", get_int(&spec, "time", 30))
        .field("movetime", get_int(&spec, "movetime", 0))
        .field("anim", get_int(&spec, "anim", -1))
        .field("sound", get_text(&spec, "sound").or_else(|| get_text(defaults, "super_sound")).unwrap_or_else(|| "-1".into()))
        .field("darken", get_int(&spec, "darken", 1))
        .field("unhittable", 1)
        .field("poweradd", 0)
        .field("p2defmul", 1)
        .finish()
}

fn trigger_of(item: &Table, fallback: &str) -> String {
    get_text(item, "trigger").unwrap_or_else(|| match get_text(item, "elem") {
        Some(elem) => format!("animelem = {elem}"),
        None => fallback.to_string(),
    })
}

fn effects(move_: &Table) -> String {
    let mut out = String::new();
    for vfx in tables(move_, "vfx") {
        let anim = get_text(vfx, "anim").unwrap_or_default();
        out += &Ctrl::new("Explod", &get_text(vfx, "label").unwrap_or_else(|| "vfx".into()))
            .field("trigger1", trigger_of(vfx, "!time")).field("anim", &anim)
            .field("id", get_text(vfx, "id").unwrap_or(anim)).field("pos", pair(vfx.get("pos"), "0, 0"))
            .field("postype", get_text(vfx, "postype").unwrap_or_else(|| "p1".into())).field("facing", get_int(vfx, "facing", 1))
            .field("bindtime", get_int(vfx, "bind", -1)).field("removetime", get_int(vfx, "removetime", -2))
            .field("scale", pair(vfx.get("scale"), "1, 1")).field("sprpriority", get_int(vfx, "sprpriority", 3))
            .field("ownpal", 1).field("removeongethit", 1).field("supermovetime", 999).field("pausemovetime", 999)
            .field("ignorehitpause", 1).opt("trans", get_text(vfx, "trans")).finish();
    }
    for helper in tables(move_, "helpers") {
        let state = get_text(helper, "state").unwrap_or_default();
        out += &Ctrl::new("Helper", &get_text(helper, "label").unwrap_or_else(|| "helper".into()))
            .field("trigger1", trigger_of(helper, "!time")).field("stateno", &state)
            .field("id", get_text(helper, "id").unwrap_or(state)).field("pos", pair(helper.get("pos"), "0, 0"))
            .field("postype", get_text(helper, "postype").unwrap_or_else(|| "p1".into()))
            .opt("facing", get_text(helper, "facing")).field("ownpal", 1).field("persistent", 0).finish();
    }
    out
}

fn cancels(move_: &Table) -> String {
    let mut out = String::new();
    for cancel in list(move_, "cancel") {
        let (command, state, condition, contact) = match cancel.as_str() {
            Some(s) => {
                let mut parts = s.split(':');
                (parts.next().unwrap_or("").to_string(), parts.next().unwrap_or("").to_string(), None, "movecontact".to_string())
            }
            None => {
                let t = cancel.as_table().cloned().unwrap_or_default();
                (get_text(&t, "command").unwrap_or_default(), get_text(&t, "state").unwrap_or_default(), get_text(&t, "condition"), get_text(&t, "contact").unwrap_or_else(|| "movecontact".into()))
            }
        };
        let extra = condition.map(|c| format!(" && ({c})")).unwrap_or_default();
        out += &Ctrl::new("ChangeState", &format!("cancel into {state}")).field("value", &state)
            .field("triggerall", format!("(roundstate = 2) && ({contact})"))
            .field("trigger1", format!("(command = \"{command}\"){extra}")).finish();
    }
    out
}

fn ending(move_: &Table) -> String {
    let kind = kind_of(move_);
    let air = is_air(&kind);
    let land = |out: &mut String| {
        *out += &Ctrl::new("ChangeState", "land").field("trigger1", "(pos y >= 0) && (vel y > 0)")
            .field("value", get_text(move_, "land").unwrap_or_else(|| "52".into())).finish();
    };
    let mut out = String::new();
    if let Some(next) = get_text(move_, "_next") {
        if air {
            land(&mut out);
        }
        return out + &Ctrl::new("ChangeState", "next phase").field("trigger1", "!animtime").field("value", next).finish();
    }
    if get_bool(move_, "helper").unwrap_or(false) {
        return Ctrl::new("DestroySelf", "end").field("trigger1", "!animtime").finish();
    }
    let end = get_text(move_, "end").unwrap_or_else(|| (if air { "50" } else if kind == "crouch" { "11" } else { "0" }).into());
    if air {
        land(&mut out);
        return out + &Ctrl::new("ChangeState", "end").field("trigger1", "!animtime").field("value", end).field("ctrl", 1).finish();
    }
    out + &Ctrl::new("ChangeState", "end").field("trigger1", "!animtime").field("value", format!("cond(pos y < 0, 50, {end})")).field("ctrl", 1).finish()
}

pub fn build_state(move_: &Table, defaults: &Table, action_text: &str) -> Result<String, String> {
    let number = get_int(move_, "state", 0);
    let kind = kind_of(move_);
    let mut out = header(move_, number, get_int(move_, "anim", number));
    if (kind == "super" && !move_.contains_key("_phase_of")) || move_.contains_key("superpause") {
        out += &super_pause(move_, defaults);
    }
    if let Some(voice) = get_text(move_, "voice") {
        out += &Ctrl::new("PlaySnd", "voice").field("trigger1", "!time").field("value", voice).field("channel", get_int(defaults, "voice_channel", 0)).finish();
    }
    for sound in tables(move_, "sounds") {
        out += &Ctrl::new("PlaySnd", "sound").field("trigger1", trigger_of(sound, "!time")).field("value", get_text(sound, "value").unwrap_or_default()).finish();
    }
    for vel in tables(move_, "vel") {
        out += &Ctrl::new(&get_text(vel, "type").unwrap_or_else(|| "VelSet".into()), "velocity").field("trigger1", trigger_of(vel, "!time"))
            .opt("x", get_text(vel, "x")).opt("y", get_text(vel, "y")).finish();
    }
    let frames = hit_frames(action_text);
    for hit in tables(move_, "hit") {
        for element in hit_elements(hit, &frames) {
            out += &hit_controller(hit, element, move_, defaults)?;
        }
    }
    out += &effects(move_);
    out += &cancels(move_);
    if let Some(extra) = move_.get("extra") {
        out += &(text(extra).trim().to_string() + "\n\n");
    }
    Ok(out + &ending(move_))
}
