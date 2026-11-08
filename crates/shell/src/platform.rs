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

/// Prefer the discrete GPU on desktop. `WGPU_POWER_PREF` (`low`, `high`, `none`) overrides it, which is how a laptop
/// user keeps the integrated GPU on battery; browsers ignore the hint.
pub fn power_preference() -> wgpu::PowerPreference {
    wgpu::PowerPreference::from_env().unwrap_or(if cfg!(target_arch = "wasm32") { wgpu::PowerPreference::None } else { wgpu::PowerPreference::HighPerformance })
}

/// Atlas side per platform: 4096 on desktop (80 MB of VRAM across the two atlases) and 2048 on phones (20 MB).
/// `UME_ATLAS` overrides it and the device limit always wins, so a small mobile GPU never gets an unsupported texture.
pub fn atlas_side(max_texture_dimension: u32) -> u32 {
    let preferred = std::env::var("UME_ATLAS").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(if cfg!(any(target_os = "android", target_os = "ios")) { 2048 } else { crate::ATLAS_SIZE });
    let ceiling = max_texture_dimension.max(256);
    preferred.clamp(256, ceiling).next_power_of_two().min(ceiling)
}
