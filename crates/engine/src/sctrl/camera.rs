use crate::camera::ZoomRequest;
use crate::chardef::Controller;
use crate::fighter::V2;
use crate::matchs::Match;

pub fn zoom(m: &mut Match, i: usize, sc: &Controller) {
    let l = m.fighters[i].lscl;
    let request = ZoomRequest {
        scale: m.num(i, sc, "scale", 1.0, 0) as f32,
        delta: m.num(i, sc, "delta", 0.0, 0) as f32,
        pos: V2 { x: m.num(i, sc, "pos", 0.0, 0) as f32 * l, y: m.num(i, sc, "pos", 0.0, 1) as f32 * l },
        lag: m.num(i, sc, "lag", 0.0, 0) as f32,
        delay: m.num(i, sc, "zoomindelay", 0.0, 0).max(0.0) as i32,
        time: m.num(i, sc, "time", 1.0, 0) as i32,
        bound: m.num(i, sc, "bound", 1.0, 0) != 0.0,
    };
    if request.scale > 0.0 { m.request_zoom(request); }
}

pub fn camera(m: &mut Match, i: usize, sc: &Controller) {
    let l = m.fighters[i].lscl;
    let scale = if sc.has("zoom", 0) { m.num(i, sc, "zoom", 1.0, 0) as f32 } else { m.camera_scale() };
    let request = ZoomRequest {
        scale,
        delta: 0.0,
        pos: V2 { x: m.num(i, sc, "pos", 0.0, 0) as f32 * l, y: m.num(i, sc, "pos", 0.0, 1) as f32 * l },
        lag: m.num(i, sc, "lag", 0.0, 0) as f32,
        delay: 0,
        time: m.num(i, sc, "time", 1.0, 0) as i32,
        bound: m.num(i, sc, "bound", 1.0, 0) != 0.0,
    };
    if request.scale > 0.0 { m.request_zoom(request); }
}

pub fn ground_level_offset(m: &mut Match, i: usize, sc: &Controller) {
    let value = m.num(i, sc, "value", 0.0, 0) as f32;
    let f = &mut m.fighters[i];
    f.ground_level = -value * f.lscl;
}

pub fn unsupported_depth(m: &mut Match, sc: &Controller) {
    let message = format!("{} is accepted but ignored: the engine has no Z axis", sc.kind);
    m.warn_once(&message);
}

pub fn slowmo(m: &mut Match, i: usize, sc: &Controller) {
    let ticks = m.num(i, sc, "time", 0.0, 0).max(0.0) as i32;
    if ticks > 0 {
        m.slow_mo = m.slow_mo.max(ticks);
    }
}

pub fn cinema(m: &mut Match, i: usize, sc: &Controller) {
    let bars = m.num(i, sc, "bars", 1.0, 0).clamp(0.0, 1.0) as f32;
    let slow = m.num(i, sc, "slow", 0.0, 0).max(0.0) as i32;
    let invert = m.num(i, sc, "invert", 0.0, 0) != 0.0;
    let lines = m.num(i, sc, "lines", 0.0, 0).clamp(0.0, 1.0) as f32;
    let flash = [
        m.num(i, sc, "flash", 0.0, 0).clamp(0.0, 255.0) as u8,
        m.num(i, sc, "flash", 0.0, 1).clamp(0.0, 255.0) as u8,
        m.num(i, sc, "flash", 0.0, 2).clamp(0.0, 255.0) as u8,
    ];
    if slow > 0 {
        m.slow_mo = m.slow_mo.max(slow);
    }
    let fighter = m.fighters[i].root;
    m.events.push(crate::matchs::Event::Cinema { fighter, bars, slow, invert, lines, flash });
}
