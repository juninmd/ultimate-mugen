#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = open)]
    fn window_open(url: &str, target: &str);
}

pub fn open(url: &str) -> bool {
    if !url.starts_with("https://github.com/") {
        return false;
    }
    launch(url)
}

#[cfg(target_arch = "wasm32")]
fn launch(url: &str) -> bool {
    window_open(url, "_blank");
    true
}

#[cfg(all(not(target_arch = "wasm32"), any(target_os = "android", target_os = "ios")))]
fn launch(_url: &str) -> bool {
    false
}

#[cfg(all(not(target_arch = "wasm32"), not(any(target_os = "android", target_os = "ios"))))]
fn launch(url: &str) -> bool {
    use std::process::{Command, Stdio};
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("rundll32");
        c.args(["url.dll,FileProtocolHandler", url]);
        c
    } else if cfg!(target_os = "macos") {
        let mut c = Command::new("open");
        c.arg(url);
        c
    } else {
        let mut c = Command::new("xdg-open");
        c.arg(url);
        c
    };
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().is_ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_github_https_links_open() {
        assert!(!super::open("http://github.com/a/b"));
        assert!(!super::open("https://evil.example/a"));
        assert!(!super::open("file:///etc/passwd"));
    }
}
