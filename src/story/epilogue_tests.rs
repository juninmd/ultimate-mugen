use super::*;

const LANGS: [&str; 3] = ["en", "pt", "es"];

fn roster() -> Vec<String> {
    let index: Vec<String> =
        serde_json::from_str(include_str!("../../assets/chars/index.json")).unwrap();
    index
        .into_iter()
        .filter(|id| !id.starts_with("npc_"))
        .collect()
}

#[test]
fn every_playable_fighter_has_its_own_epilogue() {
    let table = merged(None);
    for id in roster() {
        let own = table
            .get(&id)
            .unwrap_or_else(|| panic!("{id} has no epilogue"));
        assert_ne!(Some(own), table.get(FALLBACK), "{id} repeats the fallback");
    }
}

#[test]
fn every_epilogue_is_complete_in_three_languages() {
    for (id, epilogue) in merged(None) {
        assert!(!epilogue.scene.is_empty(), "{id} has no scene");
        assert!(epilogue.pages.len() >= 2, "{id} needs two narration pages");
        let texts = [&epilogue.title, &epilogue.boss, &epilogue.reply];
        for text in texts.into_iter().chain(epilogue.pages.iter()) {
            for lang in LANGS {
                assert!(
                    text.0.get(lang).is_some_and(|line| !line.is_empty()),
                    "{id} lacks {lang}"
                );
            }
        }
    }
}

#[test]
fn every_scene_is_a_stage_with_art() {
    for (id, epilogue) in merged(None) {
        assert!(
            crate::stages::illustration_preview(&epilogue.scene).is_some(),
            "{id} points at unknown stage {}",
            epilogue.scene
        );
    }
}

#[test]
fn beats_open_on_the_rooftop_and_close_at_home() {
    let table = merged(None);
    let beats = pick(&table, "umf:kaito").unwrap().beats("pt");
    assert_eq!(beats.len(), ROOFTOP_BEATS + 2);
    assert_eq!(beats[0].speaker, Speaker::Boss);
    assert_eq!(beats[1].speaker, Speaker::Hero);
    assert!(beats[..ROOFTOP_BEATS].iter().all(|beat| !beat.home));
    assert!(beats[ROOFTOP_BEATS..]
        .iter()
        .all(|beat| beat.home && beat.speaker == Speaker::Narrator));
    assert_eq!(
        pick(&table, "kaito").unwrap().page_count("pt"),
        beats.len() + 1
    );
}

#[test]
fn unknown_fighters_fall_back_and_custom_files_override() {
    let custom =
        parse(r#"{"kaito":{"scene":"dojo","reply":{"en":"Mine."}},"mika":{"scene":"neon"}}"#);
    let table = merged(custom);
    assert_eq!(pick(&table, "kaito").unwrap().reply.pick("pt"), "Mine.");
    assert_eq!(pick(&table, "mika").unwrap().scene, "neon");
    assert_eq!(pick(&table, "stranger"), table.get(FALLBACK));
    assert!(parse("not json").is_none());
}

#[test]
fn the_final_boss_swaps_when_the_player_is_the_kingpin() {
    assert_eq!(final_boss("kaito"), "valdric");
    assert_eq!(final_boss("valdric"), "kaito");
}

#[test]
fn mugen_fighters_get_an_invented_epilogue_with_their_name() {
    let table = merged(None);
    let tale = pick_named(&table, "mugen:goku-ssj4", "Goku SSJ4").unwrap();
    assert_ne!(Some(tale.as_ref()), table.get(FALLBACK));
    for lang in LANGS {
        let beats = tale.beats(lang);
        assert!(beats.iter().any(|beat| beat.text.contains("Goku SSJ4")), "{lang}");
        assert!(beats.iter().all(|beat| !beat.text.contains("{name}")), "{lang}");
    }
}

#[test]
fn the_invented_epilogue_is_stable_and_varies_across_fighters() {
    let table = merged(None);
    let scene_of = |id: &str| pick_named(&table, id, "X").unwrap().scene.clone();
    assert_eq!(scene_of("mugen:ryu"), scene_of("mugen:ryu"));
    let scenes: std::collections::BTreeSet<String> = (0..40)
        .map(|n| scene_of(&format!("mugen:fighter{n}")))
        .collect();
    assert!(scenes.len() >= 4, "{scenes:?}");
}

#[test]
fn own_epilogues_win_over_invented_ones_and_nameless_fighters_use_the_fallback() {
    let table = merged(None);
    assert_eq!(pick_named(&table, "umf:kaito", "Other").unwrap().as_ref(), table.get("kaito").unwrap());
    assert_eq!(pick_named(&table, "mystery", "").unwrap().as_ref(), table.get(FALLBACK).unwrap());
}
