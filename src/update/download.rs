use super::manifest::{Asset, Update};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::manifest::MAX_BINARY_BYTES;
use super::MANIFEST_URL;

const MANIFEST_LIMIT: u64 = 64 * 1024;
const MANIFEST_TIMEOUT: Duration = Duration::from_secs(8);
const BINARY_TIMEOUT: Duration = Duration::from_secs(180);

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(timeout))
        .build()
        .into()
}

pub fn fetch_manifest(current: &str) -> Result<Option<Update>, String> {
    let mut res = agent(MANIFEST_TIMEOUT)
        .get(MANIFEST_URL)
        .call()
        .map_err(|e| e.to_string())?;
    let body = res
        .body_mut()
        .with_config()
        .limit(MANIFEST_LIMIT)
        .read_to_string()
        .map_err(|e| e.to_string())?;
    Ok(super::manifest::parse_manifest(
        &body,
        current,
        super::manifest::platform(),
    ))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn staged_path(version: &str) -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot locate the running executable: {e}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "the running executable has no parent directory".to_string())?;
    let extension = if cfg!(windows) { ".exe" } else { "" };
    Ok(dir.join(format!(".update-v{version}{extension}")))
}

pub fn download_to<F>(asset: &Asset, stage: &Path, progress: F) -> Result<u64, String>
where
    F: FnMut(u64, u64),
{
    if !asset.url.starts_with("https://") {
        return Err("refusing a plain HTTP update download".into());
    }
    let mut res = agent(BINARY_TIMEOUT)
        .get(&asset.url)
        .call()
        .map_err(|e| e.to_string())?;
    let total = res
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(asset.size);
    let mut file = std::fs::File::create(stage)
        .map_err(|e| format!("cannot write next to the executable: {e}"))?;
    let mut reader = res.body_mut().as_reader();
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut written = 0u64;
    let mut report = progress;
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        written += n as u64;
        if written > asset.size.min(MAX_BINARY_BYTES) {
            return Err("update download exceeded its announced size".into());
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        report(written, total);
    }
    file.flush().map_err(|e| e.to_string())?;
    let digest: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if digest != asset.sha256 {
        return Err(format!(
            "sha256 mismatch: got {digest}, expected {}",
            asset.sha256
        ));
    }
    if written != asset.size {
        return Err(format!(
            "size mismatch: got {written} bytes, expected {}",
            asset.size
        ));
    }
    Ok(written)
}

pub fn verify_staged(stage: &Path, version: &str, asset: &Asset) -> Result<(), String> {
    let bytes = std::fs::read(stage).map_err(|e| e.to_string())?;
    if bytes.len() as u64 != asset.size {
        return Err("staged update has the wrong size".into());
    }
    let digest = sha256_hex(&bytes);
    if digest != asset.sha256 {
        return Err("staged update has the wrong sha256".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(stage, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }
    let output = std::process::Command::new(stage)
        .arg("--version")
        .output()
        .map_err(|e| format!("staged update is not runnable: {e}"))?;
    let reported = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let expected = format!("ultimate-mugen {version}");
    if !output.status.success() || reported != expected {
        return Err(format!(
            "staged update reports '{reported}', expected '{expected}'"
        ));
    }
    Ok(())
}

pub fn download_and_stage<F>(version: &str, asset: &Asset, progress: F) -> Result<PathBuf, String>
where
    F: FnMut(u64, u64),
{
    let stage = staged_path(version)?;
    let _ = std::fs::remove_file(&stage);
    if let Err(e) = download_to(asset, &stage, progress) {
        let _ = std::fs::remove_file(&stage);
        return Err(e);
    }
    if let Err(e) = verify_staged(&stage, version, asset) {
        let _ = std::fs::remove_file(&stage);
        return Err(e);
    }
    Ok(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset_for(bytes: &[u8]) -> Asset {
        Asset {
            url: "https://github.com/o/r/app".into(),
            sha256: sha256_hex(bytes),
            size: bytes.len() as u64,
        }
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ume-update-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn sha256_matches_the_reference_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn staging_rejects_a_tampered_or_truncated_payload() {
        let dir = temp_dir("tamper");
        let stage = dir.join("stage.bin");
        std::fs::write(&stage, b"payload").unwrap();
        let wrong_hash = Asset {
            sha256: "0".repeat(64),
            ..asset_for(b"payload")
        };
        assert!(verify_staged(&stage, "0.4.0", &wrong_hash)
            .unwrap_err()
            .contains("sha256"));
        let wrong_size = Asset {
            size: 9,
            ..asset_for(b"payload")
        };
        assert!(verify_staged(&stage, "0.4.0", &wrong_size)
            .unwrap_err()
            .contains("size"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn plain_http_downloads_are_refused_before_any_connection() {
        let dir = temp_dir("http");
        let stage = dir.join("stage.bin");
        let asset = Asset {
            url: "http://github.com/o/r/app".into(),
            sha256: "0".repeat(64),
            size: 1,
        };
        assert!(download_to(&asset, &stage, |_, _| {})
            .unwrap_err()
            .contains("HTTP"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
