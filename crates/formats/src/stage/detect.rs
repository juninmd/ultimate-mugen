use super::{parse_ini_no_lines, Section};

/// glTF/3D stages are rejected by design: detect the model request so the loader can warn once and keep the 2D layers.
pub fn stage_requests_3d_secs(secs: &[Section]) -> bool {
    secs.iter().any(|s| {
        matches!(
            s.lname.as_str(),
            "model" | "model3d" | "stage3d" | "gltf" | "scene"
        ) || s.entries.iter().any(|(k, v)| {
            let v = v.to_ascii_lowercase();
            (k == "model" || k == "gltf" || k == "scene" || k.contains("3dmodel"))
                && (v.ends_with(".glb") || v.ends_with(".gltf") || !v.is_empty())
        })
    })
}

pub fn stage_requests_3d(text: &str) -> bool {
    stage_requests_3d_secs(&parse_ini_no_lines(text))
}

pub(super) fn warn_3d_once(path: &str) {
    use std::sync::{Mutex, OnceLock};
    static SEEN: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    let seen = SEEN.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
    if let Ok(mut seen) = seen.lock() {
        if seen.insert(path.to_string()) {
            eprintln!("stage {path}: glTF/3D models are not supported; rendering the 2D background layers instead");
        }
    }
}
