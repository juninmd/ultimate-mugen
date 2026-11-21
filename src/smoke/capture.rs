#[cfg(not(target_arch = "wasm32"))]
pub async fn take(_app: &mut crate::app::App, out: &str, file: &str) -> Result<Option<crate::smoke::stats::ImageStats>, String> {
    let path = format!("{out}/{file}");
    crate::gfx::get_screen_data().export_png(&path);
    crate::gfx::next_frame().await;
    let bytes = std::fs::read(&path).map_err(|e| format!("screenshot not written: {e}"))?;
    crate::smoke::stats::of_png(&bytes).map(Some)
}

#[cfg(target_arch = "wasm32")]
pub async fn take(app: &mut crate::app::App, _out: &str, file: &str) -> Result<Option<crate::smoke::stats::ImageStats>, String> {
    use js_sys::Reflect;
    use wasm_bindgen::JsValue;
    let global = js_sys::global();
    let seq = js_sys::Date::now();
    let request = format!("{{\"seq\":{seq},\"file\":\"{file}\"}}");
    let _ = Reflect::set(&global, &"__ume_smoke_shot".into(), &JsValue::from_str(&request));
    for _ in 0..300 {
        app.frame().await;
        crate::gfx::next_frame().await;
        let ack = Reflect::get(&global, &"__ume_smoke_ack".into()).ok().and_then(|v| v.as_f64());
        if ack == Some(seq) {
            return Ok(None);
        }
    }
    Ok(None)
}

#[cfg(target_arch = "wasm32")]
pub fn publish(report: &str) {
    let _ = js_sys::Reflect::set(&js_sys::global(), &"__ume_smoke_report".into(), &wasm_bindgen::JsValue::from_str(report));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn publish(_report: &str) {}
