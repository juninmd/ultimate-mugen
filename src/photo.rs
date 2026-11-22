use crate::postfx::Look;
use ume_engine::photo_camera::PhotoView;

mod filter;
pub use filter::Filter;

pub const PAN_SPEED: f32 = 150.0;
pub const ZOOM_RATE: f32 = 1.9;
pub const TOAST_SECONDS: f32 = 2.4;
pub const FLASH_SECONDS: f32 = 0.35;

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub pan: (f32, f32),
    pub zoom: f32,
    pub toggle_hud: bool,
    pub next_filter: bool,
    pub reset: bool,
}

pub struct Photo {
    pub view: PhotoView,
    pub filter: Filter,
    pub hud: bool,
    pub flash: f32,
    pub toast: Option<(String, f32)>,
}

impl Photo {
    pub fn new() -> Self {
        Photo { view: PhotoView::default(), filter: Filter::Off, hud: false, flash: 0.0, toast: None }
    }

    pub fn apply(&mut self, input: Input, dt: f32) {
        let dt = dt.clamp(0.0, 0.1);
        let reach = PAN_SPEED * dt / self.view.zoom.max(0.1);
        self.view.pan(input.pan.0 * reach, input.pan.1 * reach);
        if input.zoom != 0.0 {
            self.view.zoom_by(ZOOM_RATE.powf(input.zoom * dt));
        }
        if input.toggle_hud {
            self.hud = !self.hud;
        }
        if input.next_filter {
            self.filter = self.filter.next();
        }
        if input.reset {
            self.view = PhotoView::default();
        }
        self.flash = (self.flash - dt / FLASH_SECONDS).max(0.0);
        if let Some((_, t)) = &mut self.toast {
            *t -= dt;
        }
        if self.toast.as_ref().is_some_and(|(_, t)| *t <= 0.0) {
            self.toast = None;
        }
    }

    pub fn announce(&mut self, text: String) {
        self.toast = Some((text, TOAST_SECONDS));
    }

    pub fn shutter(&mut self) {
        self.flash = 1.0;
    }

    pub fn styled_look(&self, base: &Look) -> Look {
        self.filter.apply(base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Input {
        Input::default()
    }

    #[test]
    fn panning_slows_down_when_zoomed_in() {
        let mut wide = Photo::new();
        let mut close = Photo::new();
        close.view.zoom = 2.0;
        let go = Input { pan: (1.0, 0.0), ..input() };
        wide.apply(go, 0.1);
        close.apply(go, 0.1);
        assert!(close.view.dx < wide.view.dx);
    }

    #[test]
    fn zoom_input_changes_the_view_and_stays_in_range() {
        let mut p = Photo::new();
        for _ in 0..200 {
            p.apply(Input { zoom: 1.0, ..input() }, 0.1);
        }
        assert_eq!(p.view.zoom, ume_engine::photo_camera::CLOSE_MAX);
    }

    #[test]
    fn reset_recenters_without_touching_filter_or_hud() {
        let mut p = Photo::new();
        p.apply(Input { pan: (1.0, 1.0), zoom: 1.0, toggle_hud: true, next_filter: true, ..input() }, 0.1);
        assert!(!p.view.is_centered() && p.hud && p.filter != Filter::Off);
        p.apply(Input { reset: true, ..input() }, 0.0);
        assert!(p.view.is_centered() && p.hud && p.filter != Filter::Off);
    }

    #[test]
    fn toast_and_flash_expire() {
        let mut p = Photo::new();
        p.announce("saved".into());
        p.shutter();
        for _ in 0..60 {
            p.apply(input(), 0.05);
        }
        assert!(p.toast.is_none() && p.flash == 0.0);
    }

    #[test]
    fn a_stalled_frame_cannot_fling_the_camera() {
        let mut p = Photo::new();
        p.apply(Input { pan: (1.0, 0.0), ..input() }, 30.0);
        assert!(p.view.dx <= PAN_SPEED * 0.1 + 0.01);
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn save_path() -> Result<String, String> {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("photos")))
        .unwrap_or_else(|| std::path::PathBuf::from("photos"));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    Ok(dir.join(format!("ultimate-mugen-{stamp}.png")).to_string_lossy().into_owned())
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
pub fn save_path() -> Result<String, String> {
    Err("unsupported".into())
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn allowed(s: &crate::session::Session) -> bool {
    s.net.is_none()
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
pub fn allowed(_: &crate::session::Session) -> bool {
    true
}
