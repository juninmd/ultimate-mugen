use crate::matchs::{Match, BASE_H, BASE_W};

pub const CLOSE_MAX: f32 = 2.5;
pub const FAR_MAX: f32 = 0.6;
pub const PAN_Y_LIMIT: f32 = 120.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhotoView {
    pub dx: f32,
    pub dy: f32,
    pub zoom: f32,
}

impl Default for PhotoView {
    fn default() -> Self {
        PhotoView { dx: 0.0, dy: 0.0, zoom: 1.0 }
    }
}

impl PhotoView {
    pub fn pan(&mut self, dx: f32, dy: f32) {
        if dx.is_finite() && dy.is_finite() {
            self.dx += dx;
            self.dy = (self.dy + dy).clamp(-PAN_Y_LIMIT, PAN_Y_LIMIT);
        }
    }

    pub fn zoom_by(&mut self, factor: f32) {
        if factor.is_finite() && factor > 0.0 {
            self.zoom = (self.zoom * factor).clamp(FAR_MAX, CLOSE_MAX);
        }
    }

    pub fn is_centered(&self) -> bool {
        *self == PhotoView::default()
    }
}

pub fn closeup(view: &PhotoView) -> f32 {
    view.zoom.max(1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    camera_x: f32,
    camera_y: f32,
    zoom: f32,
    view_w: f32,
    view_h: f32,
    zoffset: f32,
}

pub fn frame(m: &mut Match, view: &PhotoView) -> Option<Framing> {
    if !(m.zoom.is_finite() && m.zoom > 0.0 && view.zoom.is_finite() && view.zoom > 0.0) {
        return None;
    }
    let saved = Framing {
        camera_x: m.camera.x,
        camera_y: m.camera.y,
        zoom: m.zoom,
        view_w: m.view_w,
        view_h: m.view_h,
        zoffset: m.stage.zoffset,
    };
    let zoom = (m.zoom / view.zoom.min(1.0));
    m.zoom = zoom;
    m.view_w = BASE_W * zoom;
    m.view_h = BASE_H * zoom;
    let (lo, hi) = (m.stage.bound_left, m.stage.bound_right);
    m.camera.x = crate::clamp::ordered(m.camera.x + view.dx, lo, hi);
    m.camera.y += view.dy;
    if m.zoffset0 != 0.0 {
        m.stage.zoffset = m.zoffset0 + (m.view_h - BASE_H) * m.ground_k;
    }
    Some(saved)
}

pub fn unframe(m: &mut Match, saved: Framing) {
    m.camera.x = saved.camera_x;
    m.camera.y = saved.camera_y;
    m.zoom = saved.zoom;
    m.view_w = saved.view_w;
    m.view_h = saved.view_h;
    m.stage.zoffset = saved.zoffset;
}
