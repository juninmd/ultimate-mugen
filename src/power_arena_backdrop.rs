use crate::gfx::Texture2D;

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static BACKDROP: Option<Texture2D> =
        crate::power_arena::load_texture(include_bytes!("../assets/stages/power_arena.png"));
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn get() -> Option<Texture2D> {
    BACKDROP.with(|texture| texture.clone())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn prefetch() {
    BACKDROP.with(|_| ());
}

#[cfg(target_arch = "wasm32")]
const FILE: &str = "power_arena.png";

#[cfg(target_arch = "wasm32")]
pub(crate) fn get() -> Option<Texture2D> {
    crate::screen_art::fetch::get("assets/stages/", FILE, decode)
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn prefetch() {
    get();
}

#[cfg(target_arch = "wasm32")]
fn decode(data: &[u8]) -> Option<Texture2D> {
    crate::power_arena::load_texture(data)
}
