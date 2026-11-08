use super::*;
use ume_engine::input::k;

use super::*;
#[test]
fn both_sticks_transform_and_nothing_else_does() {
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::L3 || b == Btn::R3), k::T);
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::Select), k::T);
    for b in [
        Btn::West,
        Btn::North,
        Btn::South,
        Btn::East,
        Btn::R1,
        Btn::R2,
        Btn::L1,
        Btn::L2,
        Btn::L3,
        Btn::R3,
        Btn::Start,
        Btn::DUp,
    ] {
        assert_eq!(
            pad_bits(0.0, 0.0, |x| x == b) & k::T,
            0,
            "{b:?} must not transform"
        );
    }
    assert_eq!(pad_bits(0.0, 0.0, |_| false), 0);
}
#[test]
fn right_stick_alone_charges_ki_and_never_together_with_the_transform() {
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::R3), k::S);
    assert_eq!(
        pad_bits(0.0, 0.0, |b| b == Btn::L3 || b == Btn::R3) & k::S,
        0
    );
}
#[test]
fn shoulders_and_triggers_fill_the_four_shortcut_slots() {
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::L1), k::SLOT1);
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::L2), k::SLOT2);
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::R1), k::SLOT3);
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::R2), k::SLOT4);
}
#[test]
fn face_buttons_sticks_and_dpad_map_as_documented() {
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::West), k::X);
    assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::South), k::A);
    assert_eq!(pad_bits(0.9, 0.0, |_| false), k::R);
    assert_eq!(pad_bits(-0.9, 0.9, |_| false), k::L | k::U);
    assert_eq!(pad_bits(0.2, -0.2, |_| false), 0, "dead zone");
    assert_eq!(
        pad_bits(0.0, -0.9, |b| b == Btn::L3
            || b == Btn::R3
            || b == Btn::East),
        k::D | k::B | k::T
    );
}
#[test]
fn pad_kind_is_detected_from_name_or_vendor() {
    assert_eq!(
        PadKind::detect("Xbox Wireless Controller", None),
        PadKind::Xbox
    );
    assert_eq!(
        PadKind::detect("Xbox 360 Controller (XInput STANDARD GAMEPAD)", None),
        PadKind::Xbox
    );
    assert_eq!(PadKind::detect("Controller", Some(0x045E)), PadKind::Xbox);
    assert_eq!(
        PadKind::detect("Wireless Controller", None),
        PadKind::PlayStation
    );
    assert_eq!(
        PadKind::detect(
            "DualSense Wireless Controller (STANDARD GAMEPAD Vendor: 054c Product: 0ce6)",
            None
        ),
        PadKind::PlayStation
    );
    assert_eq!(
        PadKind::detect("8BitDo Pro 2", Some(0x2DC8)),
        PadKind::Generic
    );
}
