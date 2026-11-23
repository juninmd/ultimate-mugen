#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn ume_vpad_on() -> i32;
    fn ume_vpad_poll() -> u32;
    fn ume_vpad_nav() -> u32;
    fn ume_dpr() -> f32;
    fn ume_safe_inset(side: i32) -> f32;
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_unlock_audio() { ume_shell::audio::unlock() }

pub fn is_touch() -> bool {
    ume_shell::set_pointer_correction(ume_dpr());
    ume_vpad_on() != 0
}

pub fn safe_insets() -> [f32; 4] {
    [0, 1, 2, 3].map(ume_safe_inset)
}

pub fn vpad() -> Option<(u32, u32)> {
    (ume_vpad_on() != 0).then(|| (ume_vpad_poll(), ume_vpad_nav()))
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| console_error(&format!("panic: {info}"))));
}
