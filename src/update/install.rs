use std::path::{Path, PathBuf};
use std::process::Command;

pub fn backup_path(exe: &Path) -> Option<PathBuf> {
    let mut name = exe.file_name()?.to_os_string();
    name.push(".old");
    Some(exe.with_file_name(name))
}

pub fn cleanup_previous() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    if let Some(backup) = backup_path(&exe) {
        let _ = std::fs::remove_file(backup);
    }
}

fn replace(staged: &Path, exe: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let backup = backup_path(exe).ok_or_else(|| "unexpected executable name".to_string())?;
        let _ = std::fs::remove_file(&backup);
        std::fs::rename(exe, &backup)
            .map_err(|e| format!("cannot move the running executable aside: {e}"))?;
        if let Err(e) = std::fs::rename(staged, exe) {
            let _ = std::fs::rename(&backup, exe);
            return Err(format!("cannot install the downloaded update: {e}"));
        }
        return Ok(());
    }
    #[cfg(not(windows))]
    std::fs::rename(staged, exe).map_err(|e| format!("cannot install the downloaded update: {e}"))
}

pub fn install(staged: &Path) -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot locate the running executable: {e}"))?;
    replace(staged, &exe)?;
    Ok(exe)
}

pub fn relaunch(exe: &Path) -> Result<(), String> {
    let mut command = Command::new(exe);
    command
        .args(std::env::args().skip(1))
        .arg("--no-update-check");
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    command
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("updated, but cannot restart the game: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ume-swap-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn replace_puts_the_new_binary_in_place() {
        let dir = temp_dir("ok");
        let exe = dir.join(if cfg!(windows) { "app.exe" } else { "app" });
        let staged = dir.join(if cfg!(windows) {
            ".update.exe"
        } else {
            ".update"
        });
        std::fs::write(&exe, b"old").unwrap();
        std::fs::write(&staged, b"new").unwrap();
        replace(&staged, &exe).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"new");
        assert!(!staged.exists());
        if let Some(backup) = backup_path(&exe) {
            assert!(!backup.exists() || cfg!(windows));
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_missing_download_leaves_the_installed_binary_untouched() {
        let dir = temp_dir("missing");
        let exe = dir.join("app");
        std::fs::write(&exe, b"old").unwrap();
        assert!(replace(&dir.join(".update"), &exe).is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), b"old");
        let _ = std::fs::remove_dir_all(dir);
    }
}
