pub const ADVANTAGE: f32 = 0.8;
pub const PASSIVE: f32 = 0.85;
pub const GUARD: f32 = 0.9;

pub fn scope<T>(scale: f32, draw: impl FnOnce() -> T) -> T {
    ume_shell::alpha_scale::with(scale * crate::tweaks::current().hud_opacity, draw)
}
