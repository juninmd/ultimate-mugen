use super::*;

const LANGS: [&str; 3] = ["en", "pt", "es"];
const BIO_MAX_CHARS: usize = 260;

fn built_in_ids() -> Vec<String> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/chars");
    let mut ids: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter_map(|name| name.strip_suffix(".json").map(str::to_string))
        .filter(|id| id != "index")
        .collect();
    ids.sort();
    ids
}

#[test]
fn bundled_profiles_parse() {
    assert!(serde_json::from_str::<BTreeMap<String, CharProfile>>(BUNDLED_TEXT).is_ok());
}

#[test]
fn every_built_in_fighter_has_a_full_profile_in_every_language() {
    for id in built_in_ids() {
        let profile = bundled(&id).unwrap_or_else(|| panic!("{id} has no profile"));
        for lang in LANGS {
            let nationality = profile
                .nationality
                .0
                .get(lang)
                .map(String::as_str)
                .unwrap_or("");
            let bio = profile.bio.0.get(lang).map(String::as_str).unwrap_or("");
            assert!(!nationality.trim().is_empty(), "{id} nationality {lang}");
            assert!(!bio.trim().is_empty(), "{id} bio {lang}");
            assert!(
                bio.chars().count() <= BIO_MAX_CHARS,
                "{id} bio {lang} is too long"
            );
        }
    }
}

#[test]
fn every_profile_belongs_to_a_built_in_fighter() {
    let ids = built_in_ids();
    for id in bundled_table().keys() {
        assert!(ids.contains(id), "{id} has no fighter file");
    }
}

#[test]
fn prefixed_ids_resolve() {
    assert!(bundled("umf:kaito").is_some());
    assert!(bundled("umf:nobody").is_none());
}

#[test]
fn wrap_breaks_on_words_and_keeps_every_word() {
    let lines = wrap("one two three four five", 9.0, |s| s.chars().count() as f32);
    assert_eq!(lines, ["one two", "three", "four five"]);
}

#[test]
fn wrap_keeps_a_word_longer_than_the_line() {
    let lines = wrap("abcdefghijkl mn", 5.0, |s| s.chars().count() as f32);
    assert_eq!(lines, ["abcdefghijkl", "mn"]);
}

#[test]
fn profile_query_matches_alignment_element_weakness_and_country() {
    let p: CharProfile = serde_json::from_str(
        r#"{"country":"jp","element":"fire","weakness":"water","alignment":"villain","origin":{"en":"Kyoto"},"tags":["rushdown"]}"#,
    )
    .unwrap();
    assert!(matches_query(&p, "fire"));
    assert!(matches_query(&p, "water"));
    assert!(matches_query(&p, "villain"));
    assert!(matches_query(&p, "kyoto"));
    assert!(matches_query(&p, "rushdown"));
    assert!(!matches_query(&p, "ice"));
    assert!(alignment_label("hero").is_some());
    assert!(alignment_label("anti-hero").is_some());
    assert!(alignment_label("unknown").is_none());
}
