//! Turns a Rust panic on the web build into a readable report.
//!
//! The browser build aborts on the first panic, so the page only saw `RuntimeError: unreachable executed` and the
//! fatal overlay said "ran out of memory or hit an internal error". This hook prints the panic location and message
//! to the console and hands them to the page (`window.ume_panic`) so the overlay names the real cause.
#[cfg(target_arch = "wasm32")]
pub fn install() {
    use wasm_bindgen::{JsCast, JsValue};

    std::panic::set_hook(Box::new(|info| {
        let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_else(|| "?".into());
        let payload = info.payload().downcast_ref::<&str>().map(|s| (*s).to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_else(|| "panic".into());
        let text = format!("panicked at {location}: {payload}");
        let global = js_sys::global();
        let call = |owner: &JsValue, name: &str, arg: &JsValue| {
            let Ok(function) = js_sys::Reflect::get(owner, &name.into()) else { return };
            if let Some(function) = function.dyn_ref::<js_sys::Function>() {
                let _ = function.call1(owner, arg);
            }
        };
        if let Ok(console) = js_sys::Reflect::get(&global, &"console".into()) {
            call(&console, "error", &JsValue::from_str(&text));
        }
        call(&global, "ume_panic", &JsValue::from_str(&text));
        let _ = js_sys::Reflect::set(&global, &"__ume_panic".into(), &JsValue::from_str(&text));
    }));
}

/// `?panic=1` aborts on purpose so the reporter can be exercised end to end (test hook only).
#[cfg(target_arch = "wasm32")]
pub fn debug_trigger() {
    if crate::query_params().iter().any(|(k, v)| k == "panic" && v == "1") {
        panic!("panic reporter probe");
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn install() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn debug_trigger() {}
