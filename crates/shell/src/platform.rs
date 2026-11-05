//! Window helpers that differ per platform.
use winit::window::{Fullscreen, Window};

/// Toggle the window fullscreen. On the web, winit calls `document.exitFullscreen()` unconditionally, and a browser
/// rejects that promise with `TypeError: Not in fullscreen mode` when nothing is fullscreen (it also refuses a
/// fullscreen request made without a user gesture), so only ask the browser when it really is fullscreen.
pub fn set_window_fullscreen(window: &Window, on: bool) {
    #[cfg(target_arch = "wasm32")]
    if !on {
        let in_browser_fullscreen = web_sys::window().and_then(|w| w.document()).and_then(|d| d.fullscreen_element()).is_some();
        if !in_browser_fullscreen { return; }
    }
    window.set_fullscreen(on.then_some(Fullscreen::Borderless(None)));
}
