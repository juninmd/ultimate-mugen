#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

pub(super) fn register_dir(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect();
    files.sort();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        if !ume_native::is_kit_text(&text) {
            continue;
        }
        if let Err(error) = ume_native::register_kit(&text) {
            eprintln!("umf kit {}: {error}", file.display());
        }
    }
}
