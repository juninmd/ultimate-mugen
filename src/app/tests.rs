use super::{cycle_palette, pause_step};

#[test]
fn palette_cycling_wraps_on_both_ends() {
    assert_eq!(cycle_palette(1, 1), 2);
    assert_eq!(
        cycle_palette(12, 1),
        1,
        "forward from the last palette wraps to the first"
    );
    assert_eq!(
        cycle_palette(1, -1),
        12,
        "backward from the first palette wraps to the last"
    );
    assert_eq!(
        cycle_palette(7, 0),
        7,
        "no direction keeps the current palette"
    );
    assert_eq!(
        cycle_palette(0, 1),
        2,
        "an invalid palette is clamped before cycling"
    );
    assert_eq!(
        cycle_palette(99, -1),
        11,
        "palettes above the range are clamped"
    );
}

#[test]
fn esc_opening_the_pause_menu_does_not_close_it_on_the_same_frame() {
    // Esc is both the pause key and nav::BACK, so the opening frame consumes the edge.
    assert_eq!(pause_step(false, true), (true, false));
}

#[test]
fn pause_menu_is_navigable_once_open() {
    assert_eq!(pause_step(true, false), (true, true));
    assert_eq!(pause_step(true, true), (false, true));
    assert_eq!(pause_step(false, false), (false, false));
}

#[test]
fn character_story_and_ending_scripts_are_unique_for_each_fighter() {
    let roster = [
        "kaito", "shion", "akio", "suzume", "bruno", "sylene", "volta", "lian", "garan",
        "marlow", "kaname", "jin", "daichi", "valdric", "haruki",
    ];
    let mut openings = std::collections::BTreeSet::new();
    let mut prebosses = std::collections::BTreeSet::new();
    let mut awakenings = std::collections::BTreeSet::new();
    let mut endings = std::collections::BTreeSet::new();
    for id in roster {
        let op = super::cutscene::script_for(0, id, id, "", "Mikazuki Oboro", "");
        let pb = super::cutscene::script_for(1, id, id, "", "Mikazuki Oboro", "");
        let aw = super::cutscene::script_for(3, id, id, "", "Mikazuki Oboro", "");
        let en = super::cutscene::script_for(2, id, id, "", "Mikazuki Oboro", "");
        assert!(op.len() >= 4 && pb.len() >= 5 && aw.len() >= 6 && en.len() >= 5);
        assert!(openings.insert(op.last().unwrap().text.clone()));
        assert!(prebosses.insert(pb[3].text.clone()));
        assert!(awakenings.insert(aw[4].text.clone()));
        assert!(endings.insert(en[1].text.clone()));
    }
}
