mod app;
mod audio;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod netlink;
mod sound_mix;
mod controls;
mod gfx;
mod gamepad;
mod keybinds;
mod hud;
mod market;
mod registry;
mod render;
mod poly;
mod postfx;
mod session;
mod settings;
mod mugen_tour;
mod pixel_tour;
mod shots;
mod stage_tour;
mod stagefx;
mod stages;
mod texgen;
mod ui;
mod update;
mod vfx;

use crate::gfx::*;

type WindowConf = ume_shell::Config;

fn conf() -> WindowConf {
    // window size: saved setting, overridable with `--res 3840x2160`, `--4k`, `--fullscreen`
    let args: Vec<String> = std::env::args().collect();
    // headless probes (no window): used by the release pipeline to verify the stamped version and the published manifest
    if args.iter().any(|a| a == "--version") { println!("ultimate-mugen {}", update::VERSION); std::process::exit(0); }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    if args.iter().any(|a| a == "--check-update") {
        match update::fetch_latest(update::VERSION) {
            Ok(Some(u)) => println!("update available: {} -> {} ({})", update::VERSION, u.version, update::RELEASES_URL),
            Ok(None) => println!("up to date: {}", update::VERSION),
            Err(e) => { eprintln!("update check failed: {e}"); std::process::exit(1); }
        }
        std::process::exit(0);
    }
    let st = settings::Settings::load();
    let (mut w, mut h) = st.window_size();
    if let Some(r) = args.iter().position(|a| a == "--res").and_then(|i| args.get(i + 1)) {
        if let Some((a, b)) = r.split_once(['x', 'X']) { if let (Ok(a), Ok(b)) = (a.parse(), b.parse()) { (w, h) = (a, b); } }
    }
    if args.iter().any(|a| a == "--4k") { (w, h) = (3840, 2160); }
    let fullscreen = st.fullscreen || args.iter().any(|a| a == "--fullscreen");
    ume_shell::Config { title: "Ultimate MUGEN".into(), width: w, height: h, fullscreen, max_frames: std::env::var("UME_FRAMES").ok().and_then(|v| v.parse().ok()) }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" { fn ume_is_touch() -> i32; fn ume_set_import_visible(v: i32); }

/// Show the page's "Importar MUGEN" button only outside fights (web).
pub fn set_import_button(visible: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        static mut LAST: i32 = -1;
        let v = visible as i32;
        if unsafe { LAST } != v { unsafe { LAST = v; } ume_set_import_visible(v); }
    }
    let _ = visible;
}

/// Browser imports: JS writes a .zip into wasm memory and queues it for the registry.
#[cfg(target_arch = "wasm32")]
pub mod webimport {
    use std::sync::Mutex;
    pub static QUEUE: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn ume_import(bytes: Vec<u8>) { QUEUE.lock().unwrap().push(bytes); }
}
pub fn take_imports() -> Vec<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    { return std::mem::take(&mut *webimport::QUEUE.lock().unwrap()); }
    #[allow(unreachable_code)]
    vec![]
}

#[allow(dead_code)]
fn main() {
    if let Err(e) = run_async(conf(), game()) { eprintln!("{e}"); std::process::exit(1); }
}

async fn game() {
    let args: Vec<String> = std::env::args().collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let shots = args.iter().position(|a| a == "--shots").and_then(|i| args.get(i + 1)).cloned();
    #[cfg(target_arch = "wasm32")]
    let touch_dev = ume_is_touch() != 0;
    #[cfg(not(target_arch = "wasm32"))]
    let touch_dev = cfg!(any(target_os = "android", target_os = "ios"));
    let mut app = app::App::new(has("--touch") || touch_dev, shots.is_none() && !has("--mute")).await;
    if let Some(dir) = shots { shots::run(&mut app, &dir).await; return; }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    match netlink::NetArgs::parse(&args) {
        Ok(Some(n)) => app.start_netplay(&n).await,
        Ok(None) => {}
        Err(e) => app.msg = e,
    }
    loop {
        app.frame().await;
        if app.quit { break; }
        next_frame().await;
    }
}
