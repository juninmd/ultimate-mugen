use crate::spritekit::{sprite_mask, Mask};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct SpriteLibrary {
    entries: HashMap<(i64, i64), (PathBuf, (i32, i32))>,
    masks: HashMap<(i64, i64), Mask>,
}

impl SpriteLibrary {
    pub fn from_manifests(paths: &[PathBuf]) -> Result<SpriteLibrary, String> {
        let mut library = SpriteLibrary::default();
        for path in paths {
            let manifest = crate::spritekit::read_manifest(path)?;
            let base = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
            for entry in manifest["sprites"].as_array().ok_or("manifest has no sprites")? {
                let key = (entry["group"].as_i64().unwrap_or(0), entry["item"].as_i64().unwrap_or(0));
                let axis = (entry["axis"][0].as_i64().unwrap_or(0) as i32, entry["axis"][1].as_i64().unwrap_or(0) as i32);
                library.entries.insert(key, (base.join(entry["file"].as_str().unwrap_or_default()), axis));
            }
        }
        Ok(library)
    }

    pub fn mask_and_axis(&mut self, group: i64, item: i64) -> Result<Option<(&Mask, (i32, i32))>, String> {
        let Some((path, axis)) = self.entries.get(&(group, item)).cloned() else { return Ok(None) };
        if !self.masks.contains_key(&(group, item)) {
            self.masks.insert((group, item), sprite_mask(&path)?);
        }
        Ok(self.masks.get(&(group, item)).map(|m| (m, axis)))
    }
}
