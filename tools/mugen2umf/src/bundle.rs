use std::path::{Path, PathBuf};
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_native::NativeChar;

pub struct Loaded {
    pub json_path: PathBuf,
    pub fighter: NativeChar,
}

pub fn find_json(out: &Path) -> Result<PathBuf, String> {
    if out.is_file() {
        return Ok(out.to_path_buf());
    }
    let mut found: Vec<PathBuf> = std::fs::read_dir(out)
        .map_err(|e| format!("{}: {e}", out.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    found.sort();
    found.into_iter().next().ok_or_else(|| format!("no UMF .json in {}", out.display()))
}

pub fn load(out: &Path) -> Result<Loaded, String> {
    let json_path = find_json(out)?;
    let raw = std::fs::read_to_string(&json_path).map_err(|e| format!("{}: {e}", json_path.display()))?;
    let fighter = ume_native::parse_char(&raw)?;
    Ok(Loaded { json_path, fighter })
}

pub fn sprite_dir(json_path: &Path) -> PathBuf {
    json_path.parent().unwrap_or(Path::new(".")).join("sprites")
}

pub fn compile(loaded: &Loaded, pal_no: i32, with_images: bool) -> Result<CharDef, String> {
    let spec = loaded.fighter.mugen.as_ref().ok_or("this UMF has no exact `mugen` block; convert it again with mugen2umf")?;
    let dir = sprite_dir(&loaded.json_path);
    let read = move |name: &str| std::fs::read(dir.join(name)).ok().map(Arc::new);
    Ok(ume_native::exact::compile_exact_with(&loaded.fighter, spec, &read, pal_no, with_images, None))
}
