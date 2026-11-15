use super::encode::encode_sprite;
use super::meta::rescale_meta;
use std::collections::BTreeMap;
use std::path::Path;

const MAGIC: &[u8; 4] = b"UMEP";
const VERSION: u8 = 2;

fn entry(payload: &mut Vec<u8>, name: &str, data: &[u8]) {
    payload.extend_from_slice(&(name.len() as u16).to_le_bytes());
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(&(data.len() as u32).to_le_bytes());
    payload.extend_from_slice(data);
}

fn archive(id: &str, entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut payload = MAGIC.to_vec();
    payload.push(VERSION);
    payload.extend_from_slice(&(id.len() as u16).to_le_bytes());
    payload.extend_from_slice(id.as_bytes());
    payload.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (name, data) in entries {
        entry(&mut payload, name, data);
    }
    payload
}

pub fn pack_character(id: &str, folder: &Path, target: &Path) -> Result<usize, String> {
    pack_tiers(id, folder, target, None)
}

pub fn pack_tiers(id: &str, folder: &Path, target: &Path, menu: Option<(&Path, &[String])>) -> Result<usize, String> {
    let mut names: Vec<String> = std::fs::read_dir(folder).map_err(|e| e.to_string())?.filter_map(Result::ok).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut scales = BTreeMap::new();
    for name in names.iter().filter(|n| n.to_lowercase().ends_with(".png")) {
        let data = std::fs::read(folder.join(name)).map_err(|e| format!("{name}: {e}"))?;
        let (encoded, scale) = encode_sprite(name, &data)?;
        scales.insert(name.clone(), scale);
        files.push((name.clone(), encoded));
    }
    if names.iter().any(|n| n == "meta.json") {
        let raw = std::fs::read(folder.join("meta.json")).map_err(|e| format!("meta.json: {e}"))?;
        files.push(("meta.json".to_string(), rescale_meta(&raw, &scales)?));
    }
    if files.is_empty() {
        return Ok(0);
    }
    let view = |keep: &dyn Fn(&str) -> bool| -> Vec<(&str, &[u8])> { files.iter().filter(|(name, _)| keep(name)).map(|(name, data)| (name.as_str(), data.as_slice())).collect() };
    let payload = archive(id, &view(&|_| true));
    std::fs::write(target, &payload).map_err(|e| format!("{}: {e}", target.display()))?;
    if let Some((menu_target, wanted)) = menu {
        let slim = archive(id, &view(&|name| name == "meta.json" || wanted.iter().any(|w| w == name)));
        std::fs::write(menu_target, &slim).map_err(|e| format!("{}: {e}", menu_target.display()))?;
    }
    Ok(payload.len())
}

fn menu_files(sprites_root: &Path) -> BTreeMap<String, Vec<String>> {
    let mut wanted: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let Some(chars) = sprites_root.parent().and_then(|dir| std::fs::read_dir(dir).ok()) else { return wanted };
    for file in chars.filter_map(Result::ok).filter(|e| e.path().extension().is_some_and(|ext| ext == "json")) {
        let Some(character) = std::fs::read_to_string(file.path()).ok().and_then(|json| serde_json::from_str::<ume_engine::native::NativeChar>(&json).ok()) else { continue };
        let character = std::sync::Arc::new(character);
        ume_engine::native::register_native(character.clone());
        let files = wanted.entry(character.sprite_dir().to_string()).or_default();
        for name in ume_engine::native::menu_sprite_files(&character) {
            if !files.contains(&name) {
                files.push(name);
            }
        }
    }
    wanted
}

pub fn pack_all(sprites_root: &Path, out_root: &Path) -> Result<Vec<(String, usize)>, String> {
    let target_dir = out_root.join("assets/chars/sprites");
    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    let mut folders: Vec<_> = std::fs::read_dir(sprites_root).map_err(|e| format!("{}: {e}", sprites_root.display()))?.filter_map(Result::ok).filter(|e| e.path().is_dir()).collect();
    folders.sort_by_key(|e| e.file_name());
    let wanted = menu_files(sprites_root);
    let results = ume_formats::par::map(&folders, |folder| {
        let id = folder.file_name().to_string_lossy().into_owned();
        let menu_target = target_dir.join(format!("{id}.menu.pack"));
        let menu = wanted.get(&id).map(|files| (menu_target.as_path(), files.as_slice()));
        pack_tiers(&id, &folder.path(), &target_dir.join(format!("{id}.pack")), menu).map(|size| (id, size))
    });
    let mut packed = Vec::new();
    for result in results {
        let (id, size) = result?;
        if size > 0 {
            packed.push((id, size));
        }
    }
    Ok(packed)
}
