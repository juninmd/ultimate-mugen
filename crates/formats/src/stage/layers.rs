//! `[BG ...]` layer parsing for MUGEN stage .def files.
//! `type = parallax` is a layer that repeats without end on the X axis (tile 1 on X)
//! and drifts with its `velocity`, so an author does not need `tile = 1, 0` to scroll a sky strip.

use super::{pair, BgLayer};
use crate::air::Animation;
use crate::ini::{nums, Section};
use std::collections::HashMap;

pub(super) fn parse_layers(secs: &[Section], anims: &HashMap<i32, Animation>) -> Vec<BgLayer> {
let is_bg = |s: &Section| {
    s.lname == "bg" || (s.lname.starts_with("bg") && s.lname[2..].starts_with(|c: char| c.is_whitespace()))
};
let mut layers = Vec::new();
for s in secs.iter().filter(|s| is_bg(s)) {
    let kind = s.get("type").unwrap_or("normal").to_lowercase();
    let n = |k: &str, d: &[f32]| nums(s.get(k), d);
    let sp = pair(n("spriteno", &[0.0, 0.0]), [0.0, 0.0]);
    let delta_v = n("delta", &[1.0, 1.0]);
    let delta = [delta_v[0], delta_v.get(1).copied().unwrap_or(delta_v[0])];
    let tile = if kind == "parallax" { [1.0, 0.0] } else { pair(n("tile", &[0.0, 0.0]), [0.0, 0.0]) };
    let anim = if kind == "anim" { anims.get(&(n("actionno", &[0.0])[0] as i32)).cloned() } else { None };
    let rect = |k: &str| s.get(k).map(|_| n(k, &[0.0, 0.0, 0.0, 0.0])).filter(|w| w.len() >= 4).map(|w| [w[0], w[1], w[2], w[3]]);
    let spacing_key = if s.get("tilespacing").is_some() { "tilespacing" } else { "spacing" };
    let scale_start = pair(n("scalestart", &[1.0, 1.0]), [1.0, 1.0]);
    let scale_delta = pair(n("scaledelta", &[0.0, 0.0]), [0.0, 0.0]);
    let three = |k: &str| { let v = n(k, &[0.0, 0.0, 0.0]); [v[0], v.get(1).copied().unwrap_or(0.0), v.get(2).copied().unwrap_or(0.0)] };
    let alpha = n("alpha", &[256.0, 0.0]);
    layers.push(BgLayer {
        sprite: (sp[0] as i32, sp[1] as i32),
        start: pair(n("start", &[0.0, 0.0]), [0.0, 0.0]),
        delta,
        tile: [tile[0] as i32, tile[1] as i32],
        tile_spacing: pair(n(spacing_key, &[0.0, 0.0]), [0.0, 0.0]),
        velocity: pair(n("velocity", &[0.0, 0.0]), [0.0, 0.0]),
        layer: n("layerno", &[0.0])[0] as i32,
        trans: s.get("trans").unwrap_or("none").to_lowercase(),
        mask: n("mask", &[0.0])[0] != 0.0,
        anim,
        id: n("id", &[0.0])[0] as i32,
        position_link: n("positionlink", &[0.0])[0] != 0.0,
        window: rect("window"),
        mask_window: rect("maskwindow"),
        window_delta: s.get("windowdelta").map(|_| pair(n("windowdelta", &[0.0, 0.0]), [0.0, 0.0])),
        scale_start,
        scale_delta,
        angle: n("angle", &[0.0])[0],
        x_shear: n("xshear", &[0.0])[0],
        sin_x: three("sin.x"),
        sin_y: three("sin.y"),
        alpha: [alpha[0], alpha.get(1).copied().unwrap_or(0.0)],
        kind,
    });
}
    layers
}
