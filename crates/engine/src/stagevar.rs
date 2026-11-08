use crate::matchs::Match;
use std::collections::HashSet;
use std::sync::Mutex;

pub enum StageValue {
    Num(f64),
    Text(String),
}

const KEYS: &[&str] = &[
    "info.name",
    "info.displayname",
    "info.author",
    "info.mugenversion",
    "camera.startx",
    "camera.starty",
    "camera.boundleft",
    "camera.boundright",
    "camera.boundhigh",
    "camera.boundlow",
    "camera.verticalfollow",
    "camera.tension",
    "camera.zoomin",
    "camera.zoomout",
    "playerinfo.p1startx",
    "playerinfo.p2startx",
    "playerinfo.autoturn",
    "stageinfo.zoffset",
    "stageinfo.localcoord.x",
    "stageinfo.localcoord.y",
    "stageinfo.roundtime",
    "camera.verticalfollowdelta",
    "camera.yscrollspeed",
    "camera.zoominspeed",
    "camera.zoomoutspeed",
    "camera.lowestcap",
    "camera.dynamicboundhigh",
    "camera.boundhighdelta",
    "shadow.intensity",
    "shadow.yscale",
    "reflection.intensity",
    "playerinfo.leftbound",
    "playerinfo.rightbound",
];

const TEXT_KEYS: [&str; 4] = [
    "info.name",
    "info.displayname",
    "info.author",
    "info.mugenversion",
];
const INT_KEYS: [&str; 6] = [
    "stageinfo.localcoord.x",
    "stageinfo.localcoord.y",
    "stageinfo.roundtime",
    "playerinfo.autoturn",
    "camera.lowestcap",
    "camera.dynamicboundhigh",
];

pub fn canonical(raw: &str) -> Option<&'static str> {
    let raw = raw.trim().trim_matches('"').to_ascii_lowercase();
    match raw.as_str() {
        "localcoord" | "stageinfo.localcoord" => return Some("stageinfo.localcoord.x"),
        "autoturn" => return Some("playerinfo.autoturn"),
        _ => {}
    }
    KEYS.iter()
        .copied()
        .find(|full| **full == *raw || full.split_once('.').is_some_and(|(_, bare)| bare == raw))
}

pub fn is_text(key: &str) -> bool {
    TEXT_KEYS.contains(&key)
}

pub fn is_float(raw: &str) -> bool {
    canonical(raw).is_some_and(|k| !is_text(k) && !INT_KEYS.contains(&k))
}

pub fn warn_unknown_once(raw: &str) {
    if canonical(raw).is_some() {
        return;
    }
    static SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    let Ok(mut seen) = SEEN.lock() else { return };
    if seen
        .get_or_insert_with(HashSet::new)
        .insert(raw.to_string())
    {
        eprintln!("StageVar({raw}): unknown parameter, evaluates to 0");
    }
}

pub fn value(m: &Match, raw: &str) -> Option<StageValue> {
    let key = canonical(raw)?;
    let info = &m.stage;
    let st = m.bg.as_ref().map(|b| b.stage.as_ref());
    let n = |v: f32| Some(StageValue::Num(v as f64));
    let t = |v: &str| Some(StageValue::Text(v.to_string()));
    match key {
        "info.name" => t(st.map_or(info.name.as_str(), |s| s.info_name.as_str())),
        "info.displayname" => t(st.map_or(info.name.as_str(), |s| {
            if s.display_name.is_empty() {
                s.info_name.as_str()
            } else {
                s.display_name.as_str()
            }
        })),
        "info.author" => t(st.map_or("", |s| s.author.as_str())),
        "info.mugenversion" => t(st.map_or("", |s| s.mugen_version.as_str())),
        "camera.startx" => n(st.map_or(info.start_x, |s| s.start_x)),
        "camera.starty" => n(st.map_or(info.start_y, |s| s.start_y)),
        "camera.boundleft" => n(st.map_or(info.bound_left, |s| s.bound_left)),
        "camera.boundright" => n(st.map_or(info.bound_right, |s| s.bound_right)),
        "camera.boundhigh" => n(st.map_or(info.bound_high, |s| s.bound_high)),
        "camera.boundlow" => n(st.map_or(info.bound_low, |s| s.bound_low)),
        "camera.verticalfollow" => n(st.map_or(info.vertical_follow, |s| s.vertical_follow)),
        "camera.tension" => n(st.map_or(info.tension, |s| s.tension)),
        "camera.zoomin" => n(st.map_or(info.zoom_in, |s| s.zoom_in)),
        "camera.zoomout" => n(st.map_or(info.zoom_out, |s| s.zoom_out)),
        "camera.verticalfollowdelta" => n(st.map_or(info.camera.vertical_follow_delta, |s| {
            s.vertical_follow_delta
        })),
        "camera.yscrollspeed" => n(st.map_or(info.camera.y_scroll_speed, |s| s.y_scroll_speed)),
        "camera.zoominspeed" => n(st.map_or(info.camera.zoom_in_speed, |s| s.zoom_in_speed)),
        "camera.zoomoutspeed" => n(st.map_or(info.camera.zoom_out_speed, |s| s.zoom_out_speed)),
        "camera.lowestcap" => n(st.map_or(info.camera.lowest_cap as i32 as f32, |s| {
            s.lowest_cap as i32 as f32
        })),
        "camera.dynamicboundhigh" => n(st
            .map_or(info.camera.dynamic_bound_high as i32 as f32, |s| {
                s.dynamic_bound_high as i32 as f32
            })),
        "camera.boundhighdelta" => {
            n(st.map_or(info.camera.bound_high_delta, |s| s.bound_high_delta))
        }
        "playerinfo.p1startx" => n(st.map_or(info.p1x, |s| s.p1x)),
        "playerinfo.p2startx" => n(st.map_or(info.p2x, |s| s.p2x)),
        "playerinfo.leftbound" => n(st.map_or(info.bound_left, |s| s.bound_left)),
        "playerinfo.rightbound" => n(st.map_or(info.bound_right, |s| s.bound_right)),
        "playerinfo.autoturn" => {
            n(st.map_or(info.auto_turn as i32 as f32, |s| s.auto_turn as i32 as f32))
        }
        "stageinfo.zoffset" => n(st.map_or(info.zoffset, |s| s.zoffset)),
        "stageinfo.localcoord.x" => n(st.map_or(320.0, |s| s.local_w)),
        "stageinfo.localcoord.y" => n(st.map_or(240.0, |s| s.local_h)),
        "stageinfo.roundtime" => n(st.map_or(-1.0, |s| s.round_time)),
        "shadow.intensity" => n(st.map_or(128.0, |s| s.shadow.intensity)),
        "shadow.yscale" => n(st.map_or(0.4, |s| s.shadow.yscale)),
        "reflection.intensity" => n(st.map_or(0.0, |s| s.reflection)),
        _ => None,
    }
}
