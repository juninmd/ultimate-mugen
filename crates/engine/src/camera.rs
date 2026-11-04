use crate::fighter::V2;
use crate::matchs::Match;

#[derive(Clone, Debug)]
pub struct ZoomRequest {
    pub scale: f32,
    pub delta: f32,
    pub pos: V2,
    pub lag: f32,
    pub delay: i32,
    pub time: i32,
    pub bound: bool,
}

const FREE_SCALE_MIN: f32 = 0.25;
const FREE_SCALE_MAX: f32 = 4.0;

impl Match {
    pub fn effective_bound_high(&self) -> f32 {
        let cam = &self.stage.camera;
        if cam.dynamic_bound_high { self.stage.bound_high - cam.bound_high_delta * (self.zoom - 1.0).max(0.0) } else { self.stage.bound_high }
    }

    pub fn camera_scale(&self) -> f32 { 1.0 / self.zoom.max(f32::EPSILON) }

    pub fn zoom_limits(&self) -> (f32, f32) {
        let nearest = 1.0 / self.stage.zoom_in.max(1.0);
        let farthest = self.zoom_max.max(1.0 / self.stage.zoom_out.clamp(f32::EPSILON, 1.0)).max(1.0);
        (nearest, farthest)
    }

    pub fn warn_once(&mut self, message: &str) {
        if self.warnings.iter().any(|w| w == message) { return; }
        eprintln!("{message}");
        self.warnings.push(message.to_string());
    }

    pub fn request_zoom(&mut self, request: ZoomRequest) { self.zoom_request = Some(request); }

    pub(crate) fn step_zoom(&mut self, natural: f32) -> (V2, bool) {
        let mut target = natural;
        let mut speed = None;
        let mut offset = V2::default();
        let mut bound = true;
        let (nearest, farthest) = self.zoom_limits();
        if let Some(r) = self.zoom_request.as_mut() {
            bound = r.bound;
            if r.time <= 0 {
                self.zoom_request = None;
            } else {
                r.time -= 1;
                let wants_in = r.scale > 1.0;
                if wants_in && r.delay > 0 {
                    r.delay -= 1;
                } else {
                    let (lo, hi) = if r.bound { (nearest, farthest) } else { (1.0 / FREE_SCALE_MAX, 1.0 / FREE_SCALE_MIN) };
                    target = (1.0 / r.scale.max(f32::EPSILON)).clamp(lo, hi);
                    speed = Some((1.0 - r.lag).clamp(0.0, 1.0));
                    r.scale += r.delta;
                }
                offset = r.pos;
            }
        }
        let rate = speed.unwrap_or(if target > self.zoom { self.stage.camera.zoom_out_speed } else { self.stage.camera.zoom_in_speed }).clamp(0.0, 1.0);
        if (target - self.zoom).abs() > f32::EPSILON {
            self.zoom += (target - self.zoom) * rate;
            self.apply_zoom();
        }
        (offset, bound)
    }

    pub fn zoom_var(&self, key: &str) -> f64 {
        let (nearest, farthest) = self.zoom_limits();
        let r = self.zoom_request.as_ref();
        match key {
            "scale" => self.camera_scale() as f64,
            "target" => r.map_or(self.camera_scale(), |r| r.scale) as f64,
            "active" => r.is_some() as i32 as f64,
            "time" => r.map_or(0, |r| r.time) as f64,
            "posx" | "pos.x" => r.map_or(0.0, |r| r.pos.x) as f64,
            "posy" | "pos.y" => r.map_or(0.0, |r| r.pos.y) as f64,
            "lag" => r.map_or(0.0, |r| r.lag) as f64,
            "delta" => r.map_or(0.0, |r| r.delta) as f64,
            "delay" | "zoomindelay" => r.map_or(0, |r| r.delay) as f64,
            "bound" => r.map_or(true, |r| r.bound) as i32 as f64,
            "min" => (1.0 / farthest) as f64,
            "max" => (1.0 / nearest) as f64,
            _ => 0.0,
        }
    }
}

pub fn zoom_var_is_float(key: &str) -> bool { !matches!(key, "active" | "time" | "delay" | "zoomindelay" | "bound") }
