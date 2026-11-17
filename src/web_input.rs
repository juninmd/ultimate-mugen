#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn ume_is_touch() -> i32;
    fn ume_dpr() -> f32;
}

pub fn is_touch() -> bool {
    ume_shell::set_pointer_correction(ume_dpr());
    ume_is_touch() != 0
}
