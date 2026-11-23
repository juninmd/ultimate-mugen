use super::*;
use crate::story::epilogue::{merged, FALLBACK};

#[test]
fn writing_one_language_keeps_the_others() {
    let mut e = merged(None).get("kaito").cloned().unwrap();
    let before_en = read(&e, Field::Boss, "en");
    write(&mut e, Field::Boss, "pt", "  Fala nova  ");
    assert_eq!(read(&e, Field::Boss, "pt"), "Fala nova");
    assert_eq!(read(&e, Field::Boss, "en"), before_en);
}

#[test]
fn fields_cover_title_dialogue_and_every_page() {
    let e = merged(None).get(FALLBACK).cloned().unwrap();
    assert_eq!(fields(&e).len(), 4 + e.pages.len());
    assert_eq!(fields(&e)[0], Field::Scene);
    assert_eq!(fields(&e)[4], Field::Page(0));
}

#[test]
fn text_is_capped() {
    let mut e = Epilogue::default();
    e.pages.push(LocalText::default());
    write(&mut e, Field::Page(0), "en", &"y".repeat(900));
    assert_eq!(read(&e, Field::Page(0), "en").chars().count(), TEXT_MAX_CHARS);
}

#[test]
fn an_epilogue_table_survives_a_toml_round_trip() {
    let mut table = merged(None);
    let mut edited = table.get("kaito").cloned().unwrap();
    write(&mut edited, Field::Reply, "es", "Mía.");
    table.insert("kaito".into(), edited.clone());
    let text = toml::to_string(&table).unwrap();
    let back = epilogue_store::parse_toml(&text).unwrap();
    assert_eq!(back.get("kaito"), Some(&edited));
    assert_eq!(back.len(), table.len());
}

#[test]
fn scenes_cycle_through_every_stage_with_art_and_wrap() {
    let ids = crate::stages::scene_ids();
    assert!(ids.len() >= 21);
    assert_eq!(next_scene(ids[0], -1), *ids.last().unwrap());
    assert_eq!(next_scene(ids.last().unwrap(), 1), ids[0]);
    for id in &ids {
        assert!(crate::stages::illustration_preview(id).is_some(), "{id}");
    }
    assert_eq!(next_scene("unknown", 1), ids[1]);
}

#[test]
fn scene_is_readable_as_a_field() {
    let e = merged(None).get("kaito").cloned().unwrap();
    assert_eq!(read(&e, Field::Scene, "en"), e.scene);
}
