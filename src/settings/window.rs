use super::*;

impl Settings {
    pub fn window_size(&self) -> (u32, u32) {
        if self.window.0 >= MIN_WINDOW.0 && self.window.1 >= MIN_WINDOW.1 {
            return self.window;
        }
        let (w, h, _) = RESOLUTIONS[self.res.min(RESOLUTIONS.len() - 1)];
        (w, h)
    }
    pub fn apply_window(&self) {
        set_fullscreen(self.fullscreen);
        #[cfg(not(target_arch = "wasm32"))]
        if !self.fullscreen {
            let (w, h) = self.window_size();
            {
                let d = dpi_scale().max(0.5);
                request_new_screen_size(w as f32 / d, h as f32 / d)
            };
        }
    }
}

pub const MIN_WINDOW: (u32, u32) = (320, 180);

pub(super) fn parse_window(v: &str) -> (u32, u32) {
    v.split_once('x')
        .and_then(|(w, h)| Some((w.trim().parse().ok()?, h.trim().parse().ok()?)))
        .unwrap_or((0, 0))
}

#[derive(Default)]
pub struct WindowWatch {
    seen: (u32, u32),
    stable: f32,
}

impl WindowWatch {
    const SETTLE_SECONDS: f32 = 0.8;

    pub fn observe(
        &mut self,
        size: (u32, u32),
        dt: f32,
        fullscreen: bool,
        saved: (u32, u32),
    ) -> Option<(u32, u32)> {
        if fullscreen || size.0 < MIN_WINDOW.0 || size.1 < MIN_WINDOW.1 {
            self.seen = (0, 0);
            return None;
        }
        if size != self.seen {
            self.seen = size;
            self.stable = 0.0;
            return None;
        }
        self.stable += dt;
        (self.stable >= Self::SETTLE_SECONDS && size != saved).then_some(size)
    }
}
