pub const ZOOM_MIN: f32 = 1.0;
pub const ZOOM_MAX: f32 = 1.6;
const SPREAD_BASE: f32 = 260.0;
const ATTACK_LEAD: f32 = 0.35;
const PAN_EASE: f32 = 0.12;
const ZOOM_EASE: f32 = 0.06;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReplayCamera {
    shot: Shot,
}

impl ReplayCamera {
    pub fn new(x: f32) -> Self {
        Self { shot: Shot { x, zoom: ZOOM_MIN } }
    }

    pub fn shot(&self) -> Shot {
        self.shot
    }

    pub fn update(&mut self, subjects: &[Subject]) -> Shot {
        let valid: Vec<Subject> = subjects.iter().copied().filter(|s| s.x.is_finite()).collect();
        if valid.is_empty() {
            return self.shot;
        }
        let (lo, hi) = valid.iter().fold((f32::MAX, f32::MIN), |(lo, hi), s| (lo.min(s.x), hi.max(s.x)));
        let mid = (lo + hi) * 0.5;
        let attacker = valid.iter().find(|s| s.attacking);
        let focus_x = match attacker {
            Some(a) => mid + (a.x - mid) * ATTACK_LEAD,
            None => mid,
        };
        let spread = hi - lo;
        let target_zoom = (SPREAD_BASE / (spread + SPREAD_BASE * 0.5)).clamp(ZOOM_MIN, ZOOM_MAX);
        self.shot.x += (focus_x - self.shot.x) * PAN_EASE;
        self.shot.zoom += (target_zoom - self.shot.zoom) * ZOOM_EASE;
        self.shot
    }
}
