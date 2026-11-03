//! Ultimate MUGEN — modern 2D fighting engine (Rust + macroquad), MUGEN-compatible.
mod app;
mod audio;
mod controls;
mod gamepad;
mod hud;
mod registry;
mod render;
mod poly;
mod postfx;
mod session;
mod settings;
mod shots;
mod stage_tour;
mod stagefx;
mod stages;
mod texgen;
mod ui;
mod vfx;

use macroquad::prelude::*;

fn conf() -> Conf {
    // window size: saved setting, overridable with `--res 3840x2160`, `--4k`, `--fullscreen`
    let args: Vec<String> = std::env::args().collect();
    let st = settings::Settings::load();
    let (mut w, mut h, _) = settings::RESOLUTIONS[st.res.min(settings::RESOLUTIONS.len() - 1)];
    if let Some(r) = args.iter().position(|a| a == "--res").and_then(|i| args.get(i + 1)) {
        if let Some((a, b)) = r.split_once(['x', 'X']) { if let (Ok(a), Ok(b)) = (a.parse(), b.parse()) { (w, h) = (a, b); } }
    }
    if args.iter().any(|a| a == "--4k") { (w, h) = (3840, 2160); }
    let fullscreen = st.fullscreen || args.iter().any(|a| a == "--fullscreen");
    Conf { window_title: "Ultimate MUGEN".into(), window_width: w as i32, window_height: h as i32, fullscreen, high_dpi: true, sample_count: 4, window_resizable: true, ..Default::default() }
}

#[cfg(target_arch = "wasm32")]
extern "C" { fn ume_is_touch() -> i32; fn ume_set_import_visible(v: i32); }

/// Show the page's "Importar MUGEN" button only outside fights (web).
pub fn set_import_button(visible: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        static mut LAST: i32 = -1;
        let v = visible as i32;
        unsafe { if LAST != v { LAST = v; ume_set_import_visible(v); } }
    }
    let _ = visible;
}

/// Browser imports: JS writes a .zip into wasm memory and queues it for the registry.
#[cfg(target_arch = "wasm32")]
pub mod webimport {
    use std::sync::Mutex;
    pub static QUEUE: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
    #[no_mangle]
    pub extern "C" fn ume_alloc(n: usize) -> *mut u8 { let mut v = Vec::<u8>::with_capacity(n.max(1)); let p = v.as_mut_ptr(); std::mem::forget(v); p }
    #[no_mangle]
    pub extern "C" fn ume_import(p: *mut u8, n: usize) { let v = unsafe { Vec::from_raw_parts(p, n, n.max(1)) }; QUEUE.lock().unwrap().push(v); }
}
pub fn take_imports() -> Vec<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    { return std::mem::take(&mut *webimport::QUEUE.lock().unwrap()); }
    #[allow(unreachable_code)]
    vec![]
}

#[macroquad::main(conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let shots = args.iter().position(|a| a == "--shots").and_then(|i| args.get(i + 1)).cloned();
    #[cfg(target_arch = "wasm32")]
    let touch_dev = unsafe { ume_is_touch() } != 0;
    #[cfg(not(target_arch = "wasm32"))]
    let touch_dev = cfg!(any(target_os = "android", target_os = "ios"));
    let mut app = app::App::new(has("--touch") || touch_dev, shots.is_none() && !has("--mute")).await;
    if let Some(dir) = shots { shots::run(&mut app, &dir).await; return; }
    loop {
        app.frame().await;
        if app.quit { break; }
        next_frame().await;
    }
}
