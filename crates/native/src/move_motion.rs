use super::types::NativeMove;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VelRange {
    #[serde(default)]
    pub from: i32,
    #[serde(default)]
    pub to: i32,
    #[serde(default)]
    pub x: Option<f32>,
    #[serde(default)]
    pub y: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FrameRange {
    #[serde(default)]
    pub from: i32,
    #[serde(default)]
    pub to: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PosSetSpec {
    #[serde(default)]
    pub at: i32,
    #[serde(default)]
    pub x: Option<f32>,
    #[serde(default)]
    pub y: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PalFxSpec {
    #[serde(default)]
    pub at: i32,
    pub time: i32,
    #[serde(default)]
    pub add: Option<[i32; 3]>,
    #[serde(default)]
    pub mul: Option<[i32; 3]>,
    #[serde(default, rename = "sinAdd")]
    pub sin_add: Option<[i32; 4]>,
    #[serde(default)]
    pub color: Option<f32>,
    #[serde(default, rename = "invertAll")]
    pub invert_all: Option<bool>,
}

fn range_trigger(from: i32, to: i32) -> String {
    let mut conds = vec![];
    if from > 1 {
        conds.push(format!("AnimElemNo(0) >= {from}"));
    }
    if to > 0 {
        conds.push(format!("AnimElemNo(0) <= {to}"));
    }
    if conds.is_empty() { "1".into() } else { conds.join(" && ") }
}

fn at_trigger(at: i32) -> String {
    if at > 1 { format!("AnimElem = {at}") } else { "Time = 0".into() }
}

fn axis(name: &str, v: Option<f32>) -> String {
    v.map(|v| format!("{name} = {v}\n")).unwrap_or_default()
}

fn triple(name: &str, v: Option<[i32; 3]>) -> String {
    v.map(|v| format!("{name} = {}, {}, {}\n", v[0], v[1], v[2])).unwrap_or_default()
}

pub(super) fn own_gravity(m: &NativeMove) -> String {
    m.vel_add.iter().filter(|v| v.y.is_some()).map(|v| format!("triggerall = !({})\n", range_trigger(v.from, v.to))).collect()
}

pub(super) fn motion_cns(m: &NativeMove, sid: i32) -> String {
    let mut out = String::new();
    for v in &m.moves {
        out += &format!("[State {sid}, vel]\ntype = VelSet\ntrigger1 = AnimElem = {}\n{}{}\n", v.at, axis("x", v.vel[0]), axis("y", v.vel[1]));
    }
    for (kind, list) in [("VelMul", &m.vel_mul), ("VelAdd", &m.vel_add)] {
        for v in list {
            out += &format!("[State {sid}, vel]\ntype = {kind}\ntrigger1 = {}\n{}{}\n", range_trigger(v.from, v.to), axis("x", v.x), axis("y", v.y));
        }
    }
    for r in &m.pos_freeze {
        out += &format!("[State {sid}, posfreeze]\ntype = PosFreeze\ntrigger1 = {}\n\n", range_trigger(r.from, r.to));
    }
    for p in &m.pos_set {
        out += &format!("[State {sid}, posset]\ntype = PosSet\ntrigger1 = {}\n{}{}\n", at_trigger(p.at), axis("x", p.x), axis("y", p.y));
    }
    for p in &m.pal_fx {
        let sin = p.sin_add.map(|s| format!("sinadd = {}, {}, {}, {}\n", s[0], s[1], s[2], s[3])).unwrap_or_default();
        let color = p.color.map(|c| format!("color = {c}\n")).unwrap_or_default();
        let invert = if p.invert_all.unwrap_or(false) { "invertall = 1\n" } else { "" };
        out += &format!("[State {sid}, palfx]\ntype = PalFX\ntrigger1 = {}\ntime = {}\n{}{}{sin}{color}{invert}\n", at_trigger(p.at), p.time.max(1), triple("add", p.add), triple("mul", p.mul));
    }
    out
}
