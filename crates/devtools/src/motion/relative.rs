use std::path::{Component, Path, PathBuf};

pub fn relative_path(to: &Path, from_dir: &Path) -> PathBuf {
    let to_parts: Vec<Component> = to.components().collect();
    let from_parts: Vec<Component> = from_dir.components().collect();
    let common = to_parts.iter().zip(&from_parts).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in common..from_parts.len() {
        out.push("..");
    }
    for part in &to_parts[common..] {
        out.push(part.as_os_str());
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    out
}
