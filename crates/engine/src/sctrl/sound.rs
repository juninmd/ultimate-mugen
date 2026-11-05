use crate::chardef::Controller;
use crate::matchs::{Event, Match};

pub fn stop(m: &mut Match, i: usize, sc: &Controller) {
    let channel = m.num(i, sc, "channel", -1.0, 0) as i32;
    let owner = m.fighters[i].id;
    m.events.push(Event::StopSound { owner, channel });
}

pub fn pan(m: &mut Match, i: usize, sc: &Controller) {
    let channel = m.num(i, sc, "channel", -1.0, 0) as i32;
    let (value, absolute) = if sc.has("abspan", 0) { (m.num(i, sc, "abspan", 0.0, 0), true) } else { (m.num(i, sc, "pan", 0.0, 0), false) };
    let owner = m.fighters[i].id;
    let pan = sound_pan(m, i, value as f32, absolute);
    m.events.push(Event::SoundPan { owner, channel, pan });
}

pub fn sound_pan(m: &Match, i: usize, value: f32, absolute: bool) -> f32 {
    let f = &m.fighters[i];
    let x = if absolute { m.camera.x + value * f.lscl } else { f.pos.x + value * f.lscl * f.facing };
    let half_w = (m.view_w / 2.0).max(1.0);
    let pan = ((x - m.camera.x) / half_w).clamp(-1.0, 1.0);
    if pan.is_nan() { 0.0 } else { pan }
}
