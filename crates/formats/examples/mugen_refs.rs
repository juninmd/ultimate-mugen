use std::path::Path;
use ume_formats::audit::refs::missing_in_char;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

fn main() {
    let strict = std::env::args().any(|a| a == "--strict");
    let mut broken = 0usize;
    for dir in std::env::args().skip(1).filter(|a| a != "--strict") {
        let mut vfs = Vfs::new();
        if let Err(e) = add_dir(&mut vfs, Path::new(&dir), "c") {
            println!("{dir}: io error {e}");
            continue;
        }
        for def in find_char_defs(&vfs) {
            let missing = missing_in_char(&vfs, &def);
            if missing.is_empty() {
                println!("OK {def}");
                continue;
            }
            broken += 1;
            println!("MISSING {def} ({})", missing.len());
            for m in &missing {
                println!("  [{}] {} = {}", m.source, m.key, m.value);
            }
        }
    }
    if strict && broken > 0 {
        eprintln!("{broken} character(s) reference missing files");
        std::process::exit(1);
    }
}
