use super::manifest::Update;
use super::{download, install, RELEASES_URL, VERSION};

pub fn fetch_latest(current: &str) -> Result<Option<Update>, String> {
    download::fetch_manifest(current)
}

pub fn update_now() -> i32 {
    use std::io::Write;
    match download::fetch_manifest(VERSION) {
        Ok(None) => {
            println!("up to date: {VERSION}");
            0
        }
        Ok(Some(update)) => {
            let Some(binary) = update.binary else {
                eprintln!(
                    "v{} has no binary for this platform; download it from {RELEASES_URL}",
                    update.version
                );
                return 1;
            };
            println!("updating {VERSION} -> {}", update.version);
            let mut bucket = u64::MAX;
            let staged = download::download_and_stage(&update.version, &binary, |done, total| {
                let percent = if total > 0 {
                    done.saturating_mul(100) / total
                } else {
                    0
                };
                if percent != bucket {
                    bucket = percent;
                    print!("\r{percent}%");
                    let _ = std::io::stdout().flush();
                }
            });
            match staged.and_then(|stage| install::install(&stage)) {
                Ok(exe) => {
                    println!("\ninstalled v{} at {}", update.version, exe.display());
                    println!("restart the game to run the new version");
                    0
                }
                Err(error) => {
                    eprintln!("\n{error}");
                    1
                }
            }
        }
        Err(error) => {
            eprintln!("update check failed: {error}");
            1
        }
    }
}
