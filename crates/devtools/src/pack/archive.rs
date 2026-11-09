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

pub fn pack_character(id: &str, folder: &Path, target: &Path) -> Result<usize, String> {
    let mut names: Vec<String> = std::fs::read_dir(folder).map_err(|e| e.to_string())?.filter_map(Result::ok).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    let images: Vec<&String> = names.iter().filter(|n| n.to_lowercase().ends_with(".png")).collect();
    let meta = names.iter().find(|n| n.as_str() == "meta.json");
    let count = images.len() + meta.is_some() as usize;
    if count == 0 {
        return Ok(0);
    }
    let mut payload = MAGIC.to_vec();
    payload.push(VERSION);
    payload.extend_from_slice(&(id.len() as u16).to_le_bytes());
    payload.extend_from_slice(id.as_bytes());
    payload.extend_from_slice(&(count as u32).to_le_bytes());
    let mut scales = BTreeMap::new();
    for name in images {
        let data = std::fs::read(folder.join(name)).map_err(|e| format!("{name}: {e}"))?;
        let (encoded, scale) = encode_sprite(name, &data)?;
        scales.insert(name.clone(), scale);
        entry(&mut payload, name, &encoded);
    }
    if let Some(name) = meta {
        let raw = std::fs::read(folder.join(name)).map_err(|e| format!("{name}: {e}"))?;
        entry(&mut payload, "meta.json", &rescale_meta(&raw, &scales)?);
    }
    std::fs::write(target, &payload).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(payload.len())
}

pub fn pack_all(sprites_root: &Path, out_root: &Path) -> Result<Vec<(String, usize)>, String> {
    let target_dir = out_root.join("assets/chars/sprites");
    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    let mut folders: Vec<_> = std::fs::read_dir(sprites_root).map_err(|e| format!("{}: {e}", sprites_root.display()))?.filter_map(Result::ok).filter(|e| e.path().is_dir()).collect();
    folders.sort_by_key(|e| e.file_name());
    let results = ume_formats::par::map(&folders, |folder| {
        let id = folder.file_name().to_string_lossy().into_owned();
        pack_character(&id, &folder.path(), &target_dir.join(format!("{id}.pack"))).map(|size| (id, size))
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
