use std::path::Path;
use ume_formats::ini::{find_section, parse_ini_no_lines};
use ume_formats::vfs::{add_dir, add_zip, find_char_defs, text, Vfs};

pub struct Source {
    pub vfs: Vfs,
    pub def: String,
}

fn has_files(vfs: &Vfs, def: &str) -> bool {
    let Some(bytes) = vfs.get(def) else { return false };
    let secs = parse_ini_no_lines(&text(bytes));
    find_section(&secs, "files").is_some_and(|f| f.get("sprite").is_some() && f.get("cmd").is_some())
}

fn stem(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).trim_end_matches(".def").to_ascii_lowercase()
}

pub fn open(path: &Path) -> Result<Source, String> {
    let mut vfs = Vfs::new();
    if path.is_dir() {
        add_dir(&mut vfs, path, "").map_err(|e| format!("{}: {e}", path.display()))?;
    } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("zip")) {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        add_zip(&mut vfs, &bytes, "")?;
    } else {
        return Err(format!("not a folder or .zip: {}", path.display()));
    }
    let mut defs: Vec<String> = find_char_defs(&vfs).into_iter().filter(|d| has_files(&vfs, d)).collect();
    defs.sort_by_key(|d| (d.matches('/').count(), d.clone()));
    let folder = path.file_stem().map(|s| s.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    let def = defs.iter().find(|d| stem(d) == folder).or(defs.first()).cloned().ok_or_else(|| format!("no character .def with [Files] sprite and cmd in {}", path.display()))?;
    Ok(Source { vfs, def })
}

pub fn base_dir(def: &str) -> &str {
    def.rsplit_once('/').map_or("", |(b, _)| b)
}
