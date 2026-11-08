use super::Lang;

#[cfg(target_arch = "wasm32")]
fn raw() -> String {
    let global = js_sys::global();
    js_sys::Reflect::get(&global, &"navigator".into())
        .and_then(|n| js_sys::Reflect::get(&n, &"language".into()))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn raw() -> String {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
        .unwrap_or_default()
}

pub fn from_code(code: &str) -> Lang {
    match code.get(..2).map(str::to_ascii_lowercase).as_deref() {
        Some("pt") => Lang::Pt,
        Some("es") => Lang::Es,
        _ => Lang::En,
    }
}

pub fn language() -> Lang {
    from_code(&raw())
}
