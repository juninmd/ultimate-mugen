#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn ume_storage_get(key: &str) -> String;
    fn ume_storage_set(key: &str, value: &str);
}

#[cfg(not(target_arch = "wasm32"))]
fn path(name: &str) -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    if let Some(d) = ume_shell::ANDROID_APP
        .get()
        .and_then(|a| a.internal_data_path())
    {
        return d.join(name);
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(name)))
        .unwrap_or_else(|| name.into())
}

pub fn read(name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let v = ume_storage_get(name);
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(path(name)).ok()
    }
}

pub fn write(name: &str, text: &str) {
    #[cfg(target_arch = "wasm32")]
    ume_storage_set(name, text);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let tmp = path(&format!("{name}.tmp"));
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, path(name));
        }
    }
}
