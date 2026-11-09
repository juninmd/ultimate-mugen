use super::controller::Ctrl;
use super::kinds::{is_air, kind_defaults};
use super::value::{get_bool, get_int, get_table, get_text, list, pair};
use toml::{Table, Value};

fn stance(move_: &Table, kind: &str) -> &'static str {
    if is_air(kind) {
        "A"
    } else if kind == "crouch" || get_text(move_, "type").as_deref() == Some("C") {
        "C"
    } else {
        "S"
    }
}

pub fn hit_controller(hit: &Table, element: i64, move_: &Table, defaults: &Table) -> Result<String, String> {
    let kind = get_text(move_, "kind").unwrap_or_else(|| "normal".into());
    let base = kind_defaults(&kind)?;
    let default_text = |key: &str, fallback: &str| get_text(defaults, key).unwrap_or_else(|| fallback.to_string());
    let fall = get_bool(hit, "fall").unwrap_or(base.fall);
    let attr = get_text(hit, "attr").unwrap_or_else(|| format!("{}, {}", stance(move_, &kind), base.attribute));
    let mut ctrl = Ctrl::new("HitDef", &format!("hit elem {element}"))
        .field("trigger1", format!("animelem = {element}"))
        .field("attr", attr)
        .field("animtype", get_text(hit, "animtype").unwrap_or_else(|| "Medium".into()))
        .field("damage", pair(hit.get("damage"), base.damage))
        .field("guardflag", get_text(hit, "guardflag").unwrap_or_else(|| base.guardflag.to_string()))
        .field("pausetime", pair(hit.get("pausetime"), base.pausetime))
        .field("hitsound", get_text(hit, "hitsound").unwrap_or_else(|| default_text("hitsound", "S5,0")))
        .field("guardsound", get_text(hit, "guardsound").unwrap_or_else(|| default_text("guardsound", "S6,0")))
        .field("sparkno", get_text(hit, "sparkno").unwrap_or_else(|| default_text("sparkno", "-1")))
        .field("guard.sparkno", get_text(hit, "guard_sparkno").unwrap_or_else(|| default_text("guard_sparkno", "-1")))
        .field("sparkxy", pair(hit.get("sparkxy"), "0, -30"))
        .field("ground.type", get_text(hit, "ground_type").unwrap_or_else(|| base.ground_type.to_string()))
        .field("ground.slidetime", get_int(hit, "slidetime", 12))
        .field("ground.hittime", get_int(hit, "hittime", 15))
        .field("ground.velocity", pair(hit.get("ground_velocity"), base.ground_velocity))
        .field("air.velocity", pair(hit.get("air_velocity"), "-2.5, -3"))
        .field("fall", fall as i32);
    if fall {
        ctrl = ctrl.field("fall.recover", get_bool(hit, "fall_recover").unwrap_or(true) as i32);
    }
    if let Some(extra) = get_table(hit, "extra") {
        for (key, value) in extra {
            ctrl = ctrl.field(key, super::value::text(value));
        }
    }
    Ok(ctrl.finish())
}

pub fn hit_elements(hit: &Table, frames_with_boxes: &[i64]) -> Vec<i64> {
    let from = |key: &str| -> Vec<i64> { list(hit, key).into_iter().filter_map(Value::as_integer).collect() };
    let elems = from("elems");
    if !elems.is_empty() {
        return elems;
    }
    let elem = from("elem");
    if !elem.is_empty() {
        return elem;
    }
    frames_with_boxes.iter().take(1).copied().collect()
}
