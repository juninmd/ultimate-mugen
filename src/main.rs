mod app;
mod audio;
mod controls;
mod gamepad;
mod gfx;
mod hud;
mod i18n;
mod keybinds;
mod market;
mod menu_art;
mod mugen_tour;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod netlink;
mod pixel_tour;
mod poly;
mod postfx;
mod registry;
mod render;
mod replays;
mod screenpack;
mod session;
mod settings;
mod shots;
mod sound_mix;
mod stage_tour;
mod stagefx;
mod stages;
mod storage;
mod texgen;
mod ui;
mod update;
#[cfg(target_arch = "wasm32")]
mod web_sprites;
mod vfx;
#[cfg(test)]
mod vfx_camera_tests;

use crate::gfx::*;

type WindowConf = ume_shell::Config;

fn conf() -> WindowConf {
    // window size: saved setting, overridable with `--res 3840x2160`, `--4k`, `--fullscreen`
    let args: Vec<String> = std::env::args().collect();
    // headless probes (no window): used by the release pipeline to verify the stamped version and the published manifest
    if args.iter().any(|a| a == "--version") {
        println!("ultimate-mugen {}", update::VERSION);
        std::process::exit(0);
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    if args.iter().any(|a| a == "--check-update") {
        match update::fetch_latest(update::VERSION) {
            Ok(Some(u)) => println!(
                "update available: {} -> {} ({})",
                update::VERSION,
                u.version,
                update::RELEASES_URL
            ),
            Ok(None) => println!("up to date: {}", update::VERSION),
            Err(e) => {
                eprintln!("update check failed: {e}");
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    if args.iter().any(|a| a == "--update") {
        std::process::exit(update::update_now());
    }
    let st = settings::Settings::load();
    let (mut w, mut h) = st.window_size();
    if let Some(r) = args
        .iter()
        .position(|a| a == "--res")
        .and_then(|i| args.get(i + 1))
    {
        if let Some((a, b)) = r.split_once(['x', 'X']) {
            if let (Ok(a), Ok(b)) = (a.parse(), b.parse()) {
                (w, h) = (a, b);
            }
        }
    }
    if args.iter().any(|a| a == "--4k") {
        (w, h) = (3840, 2160);
    }
    // a browser only grants fullscreen from a user gesture: never auto-request it there, the request would be
    // rejected and the window would end up believing it is fullscreen
    let fullscreen = !cfg!(target_arch = "wasm32") && (st.fullscreen || args.iter().any(|a| a == "--fullscreen"));
    ume_shell::Config {
        title: "Ultimate MUGEN".into(),
        width: w,
        height: h,
        fullscreen,
        max_frames: std::env::var("UME_FRAMES")
            .ok()
            .and_then(|v| v.parse().ok()),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn ume_is_touch() -> i32;
    fn ume_set_import_visible(v: i32);
}

/// Show the page's "Importar MUGEN" button only outside fights (web).
pub fn set_import_button(visible: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        static mut LAST: i32 = -1;
        let v = visible as i32;
        if unsafe { LAST } != v {
            unsafe {
                LAST = v;
            }
            ume_set_import_visible(v);
        }
    }
    let _ = visible;
}

/// Browser imports: JS writes a .zip into wasm memory and queues it for the registry.
#[cfg(target_arch = "wasm32")]
pub mod webimport {
    use std::sync::Mutex;
    pub static QUEUE: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn ume_import(bytes: Vec<u8>) {
        QUEUE.lock().unwrap().push(bytes);
    }
}
pub fn take_imports() -> Vec<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        return std::mem::take(&mut *webimport::QUEUE.lock().unwrap());
    }

    vec![]
}

/// Query string of the page (`?p1=...)`, percent-decoded, for the web deep link.
#[cfg(target_arch = "wasm32")]
pub fn query_params() -> Vec<(String, String)> {
    let Ok(location) = js_sys::Reflect::get(&js_sys::global(), &"location".into()) else { return vec![] };
    let Ok(search) = js_sys::Reflect::get(&location, &"search".into()) else { return vec![] };
    let raw = search.as_string().unwrap_or_default();
    raw.trim_start_matches('?').split('&').filter(|p| !p.is_empty()).map(|pair| {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        (percent_decode(k), percent_decode(v))
    }).collect()
}

/// Console log from Rust, used by the browser diagnostics (`?perf=1`).
#[cfg(target_arch = "wasm32")]
pub fn web_log(msg: &str) {
    let Ok(console) = js_sys::Reflect::get(&js_sys::global(), &"console".into()) else { return };
    let Ok(log) = js_sys::Reflect::get(&console, &"log".into()) else { return };
    if let Some(f) = wasm_bindgen::JsCast::dyn_ref::<js_sys::Function>(&log) { let _ = f.call1(&console, &js_sys::JsString::from(msg)); }
}

fn perf_clock() -> f64 {
    #[cfg(target_arch = "wasm32")]
    { js_sys::Date::now() }
    #[cfg(not(target_arch = "wasm32"))]
    { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64() * 1000.0).unwrap_or(0.0) }
}

pub fn init_perf() {
    ume_formats::perf::set_clock(perf_clock);
    #[cfg(target_arch = "wasm32")]
    ume_formats::perf::set_sink(web_log);
    ume_formats::perf::enable(perf_enabled());
}

#[allow(dead_code)]
pub fn perf_enabled() -> bool {
    #[cfg(target_arch = "wasm32")]
    if query_params().iter().any(|(k, v)| k == "perf" && v != "0") { return true; }
    false
}

#[cfg(target_arch = "wasm32")]
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => match u8::from_str_radix(&s[i + 1..i + 3], 16) {
                Ok(v) => { out.push(v); i += 3; }
                Err(_) => { out.push(b'%'); i += 1; }
            },
            b'+' => { out.push(b' '); i += 1; }
            c => { out.push(c); i += 1; }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[allow(dead_code)]
fn main() {
    if let Err(e) = run_async(conf(), game()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

async fn game() {
    let args: Vec<String> = std::env::args().collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    init_perf();
    let shots = args.iter().position(|a| a == "--shots").and_then(|i| args.get(i + 1)).cloned();
    #[cfg(target_arch = "wasm32")]
    let touch_dev = ume_is_touch() != 0;
    #[cfg(not(target_arch = "wasm32"))]
    let touch_dev = cfg!(any(target_os = "android", target_os = "ios"));
    let mut app = app::App::new(has("--touch") || touch_dev, (shots.is_none() || has("--shots-audio")) && !has("--mute")).await;
    #[cfg(target_arch = "wasm32")]
    app.apply_deep_link();
    #[cfg(target_arch = "wasm32")]
    crate::web_sprites::prefetch_roster(app.reg.chars.iter().filter(|entry| !entry.mugen).map(|entry| entry.id.trim_start_matches("umf:").to_string()).collect());
    if let Some(dir) = shots { shots::run(&mut app, &dir).await; return; }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    match netlink::NetArgs::parse(&args) {
        Ok(Some(n)) => app.start_netplay(&n).await,
        Ok(None) => {}
        Err(e) => app.msg = e,
    }
    loop {
        app.frame().await;
        if app.quit {
            break;
        }
        next_frame().await;
    }
}
