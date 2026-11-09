use std::path::Path;
use ume_formats::audit::lint::lint_text;
use ume_formats::vfs::{add_dir, text, Vfs};

fn is_state_file(key: &str) -> bool {
    key.ends_with(".cns") || key.ends_with(".st")
}

fn main() {
    let strict = std::env::args().any(|a| a == "--strict");
    let mut total = 0usize;
    for dir in std::env::args().skip(1).filter(|a| a != "--strict") {
        let mut vfs = Vfs::new();
        if let Err(e) = add_dir(&mut vfs, Path::new(&dir), "c") {
            println!("{dir}: io error {e}");
            continue;
        }
        for (key, file) in vfs.iter() {
            if !is_state_file(key) {
                continue;
            }
            for f in lint_text(key, &text(file.as_slice())) {
                total += 1;
                println!("{} [{}] {}: {}", f.file, f.section, f.key, f.message);
            }
        }
    }
    println!("{total} finding(s)");
    if strict && total > 0 {
        std::process::exit(1);
    }
}
