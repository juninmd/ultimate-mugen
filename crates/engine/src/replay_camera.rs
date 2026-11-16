use crate::matchs::Match;

pub const TIGHTEN_MAX: f32 = 1.6;
const SPREAD_BASE: f32 = 260.0;
const ATTACK_LEAD: f32 = 0.35;
const PAN_EASE: f32 = 0.12;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Subject {
    pub x: f32,
    pub attacking: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub x: f32,
    pub zoom: f32,
}

pub fn direct(prev_x: f32, base_zoom: f32, subjects: &[Subject]) -> Shot {
    let valid = subjects.iter().filter(|s| s.x.is_finite());
    let (lo, hi, attacker) = valid.clone().fold((f32::MAX, f32::MIN, None), |(lo, hi, att), s| {
        (lo.min(s.x), hi.max(s.x), att.or(s.attacking.then_some(s.x)))
    });
    if lo > hi || !prev_x.is_finite() || !base_zoom.is_finite() {
        return Shot { x: prev_x, zoom: base_zoom };
    }
    let mid = (lo + hi) * 0.5;
    let focus = attacker.map_or(mid, |ax| mid + (ax - mid) * ATTACK_LEAD);
    let tighten = (SPREAD_BASE / ((hi - lo) + SPREAD_BASE * 0.5)).clamp(1.0, TIGHTEN_MAX);
    Shot { x: prev_x + (focus - prev_x) * PAN_EASE, zoom: base_zoom * tighten }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    camera_x: f32,
    zoom: f32,
    view_w: f32,
    view_h: f32,
}

pub fn frame(m: &mut Match) -> Option<Framing> {
    if !(m.zoom.is_finite() && m.zoom > 0.0) {
        return None;
    }
    let saved = Framing { camera_x: m.camera.x, zoom: m.zoom, view_w: m.view_w, view_h: m.view_h };
    let subjects: Vec<Subject> = m.roots().map(|f| Subject { x: f.pos.x, attacking: f.hitdef_active }).collect();
    let shot = direct(m.camera.x, m.zoom, &subjects);
    let base_w = m.view_w / m.zoom;
    let base_h = m.view_h / m.zoom;
    m.camera.x = shot.x;
    m.zoom = shot.zoom;
    m.view_w = base_w * shot.zoom;
    m.view_h = base_h * shot.zoom;
    Some(saved)
}

pub fn unframe(m: &mut Match, saved: Framing) {
    m.camera.x = saved.camera_x;
    m.zoom = saved.zoom;
    m.view_w = saved.view_w;
    m.view_h = saved.view_h;
}
