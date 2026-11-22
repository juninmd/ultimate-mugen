use super::*;

const SPRITES: &str = "sprites/";

fn is_umf_fighter(json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(json).is_ok_and(|value| {
        value.get("format").and_then(|format| format.as_str()).is_some_and(|format| format.starts_with("umf/"))
            && value.get("moves").is_some()
            && value.get("anims").is_some()
    })
}

fn sprite_files(vfs: &Vfs, dir: &str) -> Vec<(String, Vec<u8>)> {
    let prefix = format!("{dir}{SPRITES}");
    vfs.iter()
        .filter_map(|(key, file)| {
            let name = key.strip_prefix(&prefix)?;
            (!name.contains('/') && (name.ends_with(".png") || name.ends_with(".wav") || name.ends_with(".act") || name == "meta.json")).then(|| (name.to_string(), file.to_vec()))
        })
        .collect()
}

impl Registry {
    /// Registers every UMF fighter found next to a `sprites/` folder in a pack: a json plus its PNG poses, no `.def`.
    pub(super) fn add_umf_bundles(&mut self, vfs: &Vfs) -> Vec<String> {
        let candidates: Vec<String> = vfs.keys().filter(|key| key.ends_with(".json") && !key.ends_with("meta.json")).cloned().collect();
        let mut added = Vec::new();
        for key in candidates {
            let Some(file) = vfs.get(&key) else { continue };
            let json = text(&file);
            if !is_umf_fighter(&json) {
                continue;
            }
            let dir = key.rsplit_once('/').map(|(dir, _)| format!("{dir}/")).unwrap_or_default();
            let files = sprite_files(vfs, &dir);
            let result = ume_native::parse_char(&json).and_then(|fighter| {
                if !files.is_empty() {
                    ume_native::insert_sprite_pack(&fighter.id, &ume_native::encode_sprite_pack(fighter.sprite_dir(), &files)?)?;
                }
                self.add_native_json(&json)
            });
            match result {
                Ok(_) => added.push(key),
                Err(error) => {
                    eprintln!("umf bundle {key}: {error}");
                    self.import_errors.push(format!("{key}: {error}"));
                }
            }
        }
        added
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vfs {
        let mut fighter: serde_json::Value = serde_json::from_str(include_str!("../../assets/chars/kaito.json")).unwrap();
        fighter["id"] = "bundle_probe".into();
        let mut vfs = Vfs::new();
        vfs.insert("probe/probe.json".into(), fighter.to_string().into_bytes());
        vfs.insert("probe/sprites/01_idle.png".into(), include_bytes!("../../assets/chars/sprites/kaito/01_idle.png").to_vec());
        vfs.insert("probe/sprites/notes.txt".into(), b"ignored".to_vec());
        vfs
    }

    #[test]
    fn json_with_sprites_folder_becomes_a_native_fighter() {
        let mut registry = Registry::new();
        let added = registry.add_umf_bundles(&fixture());
        assert_eq!(added, vec!["probe/probe.json".to_string()]);
        assert!(registry.chars.iter().any(|entry| entry.id == "umf:bundle_probe"));
        assert!(ume_native::has_sprite_pack("bundle_probe"));
    }

    #[test]
    fn plain_json_and_kits_are_not_fighters() {
        assert!(!is_umf_fighter(r#"{"format":"umf-kit/1","id":"x"}"#));
        assert!(!is_umf_fighter(r#"{"hello":1}"#));
        assert!(!is_umf_fighter("not json"));
    }

    #[test]
    fn real_goku_ssj4_bundle_registers_when_the_assets_repo_is_present() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../mugen-assets/umf/Goku_SSJ4_UMF");
        let Ok(json) = std::fs::read(dir.join("goku_ssj4_umf.json")) else { return };
        let mut vfs = Vfs::new();
        vfs.insert("g/goku_ssj4_umf.json".into(), json);
        for entry in std::fs::read_dir(dir.join("sprites")).unwrap().flatten() {
            vfs.insert(format!("g/sprites/{}", entry.file_name().to_string_lossy()), std::fs::read(entry.path()).unwrap());
        }
        let mut registry = Registry::new();
        let added = registry.add_umf_bundles(&vfs);
        assert_eq!(added.len(), 1, "{:?}", registry.import_errors);
        assert!(registry.import_errors.is_empty());
    }
}
