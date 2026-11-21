use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

pub fn install() {
    std::panic::set_hook(Box::new(|info| console_error(&format!("panic: {info}"))));
}
