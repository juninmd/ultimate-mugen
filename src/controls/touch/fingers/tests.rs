use super::*;

fn lay() -> Layout {
    Layout::build(1280.0, 720.0, [0.0; 4], 1.0, false)
}

fn down(id: u64, p: Vec2) -> Sample {
    Sample {
        id,
        pos: p,
        ended: false,
    }
}

fn up(id: u64, p: Vec2) -> Sample {
    Sample {
        id,
        pos: p,
        ended: true,
    }
}

fn in_zone(l: &Layout) -> Vec2 {
    l.stick + vec2(l.stick_radius * 0.5, 0.0)
}

#[test]
fn tapping_an_attack_while_the_stick_is_held_presses_both() {
    let l = lay();
    let mut f = Fingers::default();
    let stick = down(1, in_zone(&l));
    let out = f.update(&[stick], &l, false, false);
    assert!(out.stick.is_some_and(|s| s.bits & k::R != 0));
    let out = f.update(&[stick, down(2, l.pos[0])], &l, false, false);
    assert!(
        out.held[0],
        "the attack button registers with the stick finger still down"
    );
    assert!(
        out.stick.is_some_and(|s| s.bits & k::R != 0),
        "the stick keeps its direction"
    );
}

#[test]
fn a_stick_finger_sliding_over_a_button_never_presses_it() {
    let l = lay();
    let mut f = Fingers::default();
    f.update(&[down(1, in_zone(&l))], &l, false, false);
    let out = f.update(&[down(1, l.pos[1])], &l, false, false);
    assert!(!out.held[1]);
    assert!(out.stick.is_some());
}

#[test]
fn a_button_finger_dragged_into_the_stick_zone_does_not_steer() {
    let l = lay();
    let mut f = Fingers::default();
    f.update(&[down(1, l.pos[0])], &l, false, false);
    let out = f.update(&[down(1, in_zone(&l))], &l, false, false);
    assert!(out.stick.is_none());
    assert!(!out.held[0]);
}

#[test]
fn sliding_between_buttons_moves_the_press() {
    let l = lay();
    let mut f = Fingers::default();
    f.update(&[down(1, l.pos[0])], &l, false, false);
    let out = f.update(&[down(1, l.pos[1])], &l, false, false);
    assert!(out.held[1] && !out.held[0]);
}

#[test]
fn a_tap_shorter_than_a_frame_still_releases_its_button() {
    let l = lay();
    let mut f = Fingers::default();
    let out = f.update(&[up(7, l.pos[3])], &l, false, false);
    assert!(out.released[3]);
}

#[test]
fn a_vanished_finger_releases_what_it_held() {
    let l = lay();
    let mut f = Fingers::default();
    f.update(&[down(1, l.pos[2]), down(2, in_zone(&l))], &l, false, false);
    let out = f.update(&[], &l, false, false);
    assert!(out.released[2] && out.stick.is_none() && f.is_empty());
}

#[test]
fn a_second_finger_in_the_stick_zone_is_ignored() {
    let l = lay();
    let mut f = Fingers::default();
    f.update(&[down(1, in_zone(&l))], &l, false, false);
    let other = in_zone(&l) + vec2(0.0, -40.0);
    let out = f.update(&[down(1, in_zone(&l)), down(2, other)], &l, false, false);
    let stick = out.stick.unwrap();
    assert_eq!(stick.origin, l.stick);
}

#[test]
fn the_floating_stick_is_centred_where_the_finger_lands_and_follows_a_long_drag() {
    let l = lay();
    let mut f = Fingers::default();
    let start = vec2(l.zone[0] + 200.0, l.zone[1] + 200.0);
    let out = f.update(&[down(1, start)], &l, false, true);
    assert_eq!(out.stick.unwrap().origin, start);
    assert_eq!(out.stick.unwrap().bits, 0);
    let far = start + vec2(l.stick_radius * 3.0, 0.0);
    let out = f.update(&[down(1, far)], &l, false, true);
    let s = out.stick.unwrap();
    assert!(
        (s.origin.x - (far.x - l.stick_radius)).abs() < 0.01,
        "origin trails the finger"
    );
    assert!(s.bits & k::R != 0);
    let back = far - vec2(l.stick_radius * 1.2, 0.0);
    let out = f.update(&[down(1, back)], &l, false, true);
    assert!(
        out.stick.unwrap().bits & k::L != 0,
        "reversing direction is immediate"
    );
}

#[test]
fn the_floating_origin_stays_inside_the_stick_zone() {
    let l = lay();
    let mut f = Fingers::default();
    let out = f.update(&[down(1, vec2(2.0, l.zone[1] + 2.0))], &l, false, true);
    let o = out.stick.unwrap().origin;
    assert!(o.x >= l.stick_radius - 0.01 && o.y >= l.zone[1] + l.stick_radius - 0.01);
}

#[test]
fn the_stick_maps_eight_directions_with_a_dead_zone() {
    let r = 100.0;
    assert_eq!(stick_bits(vec2(5.0, 5.0), r, false), 0);
    assert_eq!(stick_bits(vec2(60.0, 0.0), r, false), k::R);
    assert_eq!(stick_bits(vec2(-60.0, 3.0), r, false), k::L);
    assert_eq!(stick_bits(vec2(0.0, 60.0), r, false), k::D);
    assert_eq!(stick_bits(vec2(0.0, -60.0), r, false), k::U);
    assert_eq!(stick_bits(vec2(50.0, 50.0), r, false), k::R | k::D);
    assert_eq!(stick_bits(vec2(-50.0, -50.0), r, false), k::L | k::U);
    assert_eq!(
        stick_bits(vec2(19.0, 0.0), r, true),
        k::R,
        "an active direction survives a smaller dead zone"
    );
    assert_eq!(stick_bits(vec2(19.0, 0.0), r, false), 0);
}

#[test]
fn closing_the_extras_panel_drops_fingers_held_on_hidden_buttons() {
    let l = lay();
    let mut f = Fingers::default();
    let extra = CORE_BUTTONS;
    let s = down(1, l.pos[extra]);
    assert!(f.update(&[s], &l, true, false).held[extra]);
    assert!(!f.update(&[s], &l, false, false).held[extra]);
}

#[test]
fn the_extras_toggle_fires_once_per_touch() {
    let l = lay();
    let mut f = Fingers::default();
    let s = down(1, l.toggle);
    assert!(f.update(&[s], &l, false, false).toggle);
    assert!(
        !f.update(&[s], &l, false, false).toggle,
        "a held finger does not keep toggling"
    );
    assert!(!f.update(&[up(1, l.toggle)], &l, false, false).toggle);
    assert!(
        f.update(&[up(2, l.toggle)], &l, false, false).toggle,
        "a tap shorter than a frame still toggles"
    );
}
