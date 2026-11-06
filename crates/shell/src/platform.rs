//! Window helpers that differ per platform.
use winit::window::{Fullscreen, Window};

/// Swapchain depth for presentation. One queued frame is the difference between reading input on the frame it happened
/// and reading it one frame late, which a fighting game feels; drivers accept it on DX12 (`SetMaximumFrameLatency`),
/// Vulkan (swapchain images = latency + 1) and Metal. `UME_FRAME_LATENCY` overrides it for machines that stutter.
pub fn frame_latency() -> u32 {
    std::env::var("UME_FRAME_LATENCY").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(1).clamp(1, 3)
}

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
