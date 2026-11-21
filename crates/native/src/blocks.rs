use super::types::{NativeMove, OneOrMany, ProjectileSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VfxSpec {
    pub at: i32,
    pub anim: String,
    #[serde(default)]
    pub offset: Option<[f32; 2]>,
    #[serde(default)]
    pub time: Option<i32>,
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default)]
    pub follow: Option<bool>,
    #[serde(default)]
    pub behind: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuakeSpec {
    pub at: i32,
    #[serde(default)]
    pub time: Option<i32>,
    #[serde(default)]
    pub amp: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlinkSpec {
    pub at: i32,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub gap: Option<f32>,
    #[serde(default)]
    pub y: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepeatSpec {
    pub count: i32,
    #[serde(default)]
    pub every: Option<i32>,
    #[serde(default, rename = "offsetStep")]
    pub offset_step: Option<[f32; 2]>,
    #[serde(default, rename = "speedStep")]
    pub speed_step: Option<[f32; 2]>,
    #[serde(default, rename = "damageStep")]
    pub damage_step: Option<f32>,
}

const MAX_REPEAT: i32 = 24;

pub(super) fn projectiles(m: &NativeMove, frames: usize) -> Vec<ProjectileSpec> {
    let list = m
        .projectile
        .as_ref()
        .map(OneOrMany::to_vec)
        .unwrap_or_default();
    list.into_iter()
        .flat_map(|p| expand(&p, frames as i32))
        .collect()
}

fn expand(p: &ProjectileSpec, frames: i32) -> Vec<ProjectileSpec> {
    let Some(r) = &p.repeat else {
        return vec![p.clone()];
    };
    let (off, spd) = (
        r.offset_step.unwrap_or([0.0, 0.0]),
        r.speed_step.unwrap_or([0.0, 0.0]),
    );
    (0..r.count.clamp(1, MAX_REPEAT))
        .map(|k| {
            let f = k as f32;
            let mut q = p.clone();
            q.repeat = None;
            q.at = (p.at + k * r.every.unwrap_or(2).max(1)).min(frames.max(p.at));
            q.offset = [p.offset[0] + off[0] * f, p.offset[1] + off[1] * f];
            q.speed = [p.speed[0] + spd[0] * f, p.speed[1] + spd[1] * f];
            q.damage = (p.damage + r.damage_step.unwrap_or(0.0) * f).max(1.0);
            q
        })
        .collect()
}

fn blink(sid: i32, b: &BlinkSpec) -> String {
    let gap = b.gap.unwrap_or(40.0);
    let body = match b.to.as_deref().unwrap_or("behind") {
        "front" => format!(
            "type = PosAdd\ntrigger1 = AnimElem = {}\nx = p2dist x - {gap}\ny = {}\n",
            b.at,
            b.y.unwrap_or(0.0)
        ),
        "above" => format!(
            "type = PosAdd\ntrigger1 = AnimElem = {}\nx = p2dist x\ny = {}\n",
            b.at,
            -b.y.unwrap_or(gap).abs()
        ),
        "back" => format!(
            "type = PosAdd\ntrigger1 = AnimElem = {}\nx = -{gap}\ny = {}\n",
            b.at,
            b.y.unwrap_or(0.0)
        ),
        _ => format!(
            "type = PosAdd\ntrigger1 = AnimElem = {}\nx = p2dist x + {gap}\ny = {}\n",
            b.at,
            b.y.unwrap_or(0.0)
        ),
    };
    let turn = matches!(b.to.as_deref().unwrap_or("behind"), "behind");
    let flip = if turn {
        format!(
            "[State {sid}, blinkturn]\ntype = Turn\ntrigger1 = AnimElem = {}\n\n",
            b.at
        )
    } else {
        String::new()
    };
    format!("[State {sid}, blink]\n{body}\n{flip}")
}

fn vfx(sid: i32, v: &VfxSpec, anims: &HashMap<String, i32>) -> String {
    let o = v.offset.unwrap_or([0.0, -50.0]);
    let id = anims.get(&v.anim).copied().unwrap_or(0);
    let bind = if v.follow == Some(true) {
        format!("bindtime = {}\n", v.time.unwrap_or(-1))
    } else {
        String::new()
    };
    let scale = v
        .scale
        .map(|s| format!("scale = {s}, {s}\n"))
        .unwrap_or_default();
    format!("[State {sid}, vfx]\ntype = Explod\ntrigger1 = AnimElem = {}\nanim = {id}\npostype = p1\npos = {}, {}\nremovetime = {}\nontop = {}\nsprpriority = 3\n{bind}{scale}\n",
        v.at, o[0], o[1], v.time.unwrap_or(-2), (v.behind != Some(true)) as i32)
}

fn quake(sid: i32, q: &QuakeSpec) -> String {
    format!("[State {sid}, quake]\ntype = EnvShake\ntrigger1 = AnimElem = {}\ntime = {}\nfreq = 60\nampl = {}\n\n", q.at, q.time.unwrap_or(8), -q.amp.unwrap_or(4.0).abs())
}

pub(super) fn states(m: &NativeMove, sid: i32, anims: &HashMap<String, i32>) -> String {
    let blinks = m.blink.as_ref().map(OneOrMany::to_vec).unwrap_or_default();
    blinks
        .iter()
        .map(|b| blink(sid, b))
        .chain(m.vfx.iter().map(|v| vfx(sid, v, anims)))
        .chain(m.quake.iter().map(|q| quake(sid, q)))
        .collect()
}

#[cfg(test)]
#[path = "blocks_tests.rs"]
mod tests;
