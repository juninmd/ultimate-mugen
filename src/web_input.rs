#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn ume_is_touch() -> i32;
    fn ume_dpr() -> f32;
    fn ume_safe_inset(side: i32) -> f32;
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_unlock_audio() { ume_shell::audio::unlock() }

pub fn is_touch() -> bool {
    ume_shell::set_pointer_correction(ume_dpr());
    ume_is_touch() != 0
}

pub fn safe_insets() -> [f32; 4] {
    [0, 1, 2, 3].map(ume_safe_inset)
}
