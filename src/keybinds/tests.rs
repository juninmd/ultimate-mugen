use super::*;

#[test]
fn charge_is_rebindable_and_old_bindings_migrate() {
    let mut m = KeyMap::from_text("p1.14=R\np2.14=KpEnter\n");
    assert_eq!(m.bits(0, |k| k == KeyCode::R), k::S);
    assert_eq!(m.bits(0, |k| k == KeyCode::F), k::TAG);
    assert_eq!(m.bits(1, |k| k == KeyCode::KpDecimal), k::S);
    m.assign(0, 15, KeyCode::Space);
    assert_eq!(KeyMap::from_text(&m.to_text()), m);
}

#[test]
fn depth_buttons_have_defaults_and_old_files_keep_their_keys() {
    let m = KeyMap::default();
    assert_eq!(m.bits(0, |k| k == KeyCode::Z), k::LANE_UP);
    assert_eq!(m.bits(0, |k| k == KeyCode::X), k::LANE_DOWN);
    assert_eq!(m.bits(1, |k| k == KeyCode::KpSubtract), k::LANE_UP);
    let legacy = KeyMap::from_text("p1.4=Z
p1.5=X
");
    assert_eq!(legacy.bits(0, |k| k == KeyCode::Z), k::X, "a user key is never stolen by the new defaults");
    let lane_keys: Vec<_> = legacy.slots[0][16..].to_vec();
    assert!(lane_keys.iter().all(|key| !legacy.slots[0][..16].contains(key)));
}

#[test]
fn defaults_match_the_documented_layout() {
    let m = KeyMap::default();
    assert_eq!(m.bits(0, |k| k == KeyCode::Q), k::X);
    assert_eq!(m.bits(0, |k| k == KeyCode::W), k::Y);
    assert_eq!(m.bits(0, |k| k == KeyCode::E), k::Z);
    assert_eq!(m.bits(0, |k| k == KeyCode::A), k::A);
    assert_eq!(m.bits(0, |k| k == KeyCode::S), k::B);
    assert_eq!(m.bits(0, |k| k == KeyCode::D), k::C);
    assert_eq!(
        m.bits(0, |k| k == KeyCode::Up || k == KeyCode::Right),
        k::U | k::R
    );
    assert_eq!(
        m.bits(1, |k| k == KeyCode::I || k == KeyCode::L),
        k::U | k::R
    );
    assert_eq!(m.bits(1, |k| k == KeyCode::Kp4), k::X);
    assert_eq!(m.bits(1, |k| k == KeyCode::Q), 0, "P2 never reads P1 keys");
}

#[test]
fn assigning_a_used_key_swaps_instead_of_duplicating() {
    let mut m = KeyMap::default();
    m.assign(0, 4, KeyCode::W);
    assert_eq!(m.slots[0][4], KeyCode::W);
    assert_eq!(m.slots[0][5], KeyCode::Q, "HP takes LP's old key");
}

#[test]
fn text_round_trip_keeps_custom_bindings() {
    let mut m = KeyMap::default();
    m.assign(1, 0, KeyCode::Space);
    assert_eq!(KeyMap::from_text(&m.to_text()), m);
}

#[test]
fn saved_file_with_the_previous_defaults_upgrades_to_the_new_ones() {
    for layout in previous_defaults() {
        let mut old = KeyMap::default();
        old.slots = layout;
        assert_eq!(KeyMap::from_text(&old.to_text()), KeyMap::default());
    }
}

#[test]
fn corrupt_file_never_leaves_two_actions_on_one_key() {
    let m = KeyMap::from_text(
        "p1.0=Q
p1.1=Q
p2.0=Nope
garbage",
    );
    assert_eq!(m.slots[0], KeyMap::default().slots[0]);
    assert_eq!(m.slots[1], KeyMap::default().slots[1]);
}
