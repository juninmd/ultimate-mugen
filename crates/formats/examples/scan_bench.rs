//! Startup cost of a large MUGEN folder: index the tree, discover .def files, key every fighter.
//! Usage: cargo run --release -p ume-formats --example scan_bench -- [mugen_dir]
use std::time::Instant;
use ume_formats::vfs::{add_dir, char_key, find_char_defs, find_stage_defs, text, Vfs};

#[cfg(windows)]
fn rss_mb() -> f64 {
    #[repr(C)]
    struct Counters { cb: u32, fault: u32, peak_ws: usize, ws: usize, _rest: [usize; 6] }
    extern "system" { fn GetCurrentProcess() -> isize; fn K32GetProcessMemoryInfo(p: isize, c: *mut Counters, cb: u32) -> i32; }
    let mut c = Counters { cb: std::mem::size_of::<Counters>() as u32, fault: 0, peak_ws: 0, ws: 0, _rest: [0; 6] };
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    if ok == 0 { 0.0 } else { c.ws as f64 / (1024.0 * 1024.0) }
}

#[cfg(not(windows))]
fn rss_mb() -> f64 { 0.0 }

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| format!("{}/../../mugen", env!("CARGO_MANIFEST_DIR")));
    let root = std::path::Path::new(&dir);
    if !root.is_dir() { eprintln!("not a directory: {dir}"); std::process::exit(1); }

    let t = Instant::now();
    let mut vfs = Vfs::new();
    let n = add_dir(&mut vfs, root, "").expect("scan");
    let scan = t.elapsed().as_secs_f64();
    println!("index   {:8.2} s  {n} files  rss {:.0} MB", scan, rss_mb());

    let t = Instant::now();
    let chars = find_char_defs(&vfs);
    let stages = find_stage_defs(&vfs);
    println!("discover{:8.2} s  {} chars, {} stages  rss {:.0} MB", t.elapsed().as_secs_f64(), chars.len(), stages.len(), rss_mb());

    let t = Instant::now();
    for def in &chars { let bytes = vfs.get(def); let _ = bytes.map(|v| text(&v)).unwrap_or_default(); }
    println!("read def{:8.2} s  rss {:.0} MB", t.elapsed().as_secs_f64(), rss_mb());

    let t = Instant::now();
    for def in &chars { let _ = char_key(&vfs, def); }
    println!("char_key{:8.2} s  rss {:.0} MB", t.elapsed().as_secs_f64(), rss_mb());

    let t = Instant::now();
    for def in &stages { let _ = char_key(&vfs, def); }
    println!("stage_key{:7.2} s  rss {:.0} MB", t.elapsed().as_secs_f64(), rss_mb());
    println!("total   {:8.2} s", scan + 0.0);
}
