use super::*;

fn effective_country(id: &str, raw: &serde_json::Value) -> String {
    raw.pointer("/profile/country")
        .and_then(|code| code.as_str())
        .map(str::to_string)
        .or_else(|| bundled(id).map(|profile| profile.country.clone()))
        .unwrap_or_else(|| panic!("{id} has no country"))
        .to_lowercase()
}

#[test]
fn every_built_in_fighter_has_a_country_nobody_else_uses() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/chars");
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let name = entry.file_name().into_string().unwrap();
        let Some(id) = name.strip_suffix(".json").filter(|id| *id != "index") else {
            continue;
        };
        let raw: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(entry.path()).unwrap()).unwrap();
        let country = effective_country(id, &raw);
        assert!(crate::flag::design(&country).is_some(), "{id}: no flag {country}");
        if let Some(other) = seen.insert(country.clone(), id.to_string()) {
            panic!("{id} and {other} share country {country}");
        }
    }
}

#[test]
fn bundled_and_built_in_profiles_agree_on_the_country() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/chars");
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let name = entry.file_name().into_string().unwrap();
        let Some(id) = name.strip_suffix(".json").filter(|id| *id != "index") else {
            continue;
        };
        let raw: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(entry.path()).unwrap()).unwrap();
        if let (Some(own), Some(table)) = (raw.pointer("/profile/country"), bundled(id)) {
            let own = own.as_str().unwrap().to_lowercase();
            assert_eq!(own, table.country.to_lowercase(), "{id} disagrees");
        }
    }
}
