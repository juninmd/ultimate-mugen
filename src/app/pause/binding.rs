use crate::controls::shortcuts::{key_bits, Shortcuts};
use crate::gfx::KeyCode;
use ume_engine::input::k;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    Set(usize),
    Cleared(usize),
}

pub fn held_slots(pad_bits: impl IntoIterator<Item = u32>, down: impl Fn(KeyCode) -> bool) -> u32 {
    pad_bits.into_iter().fold(0, |b, p| b | (p & k::SLOT_MASK)) | key_bits(down)
}

pub fn fresh_slot(held: u32, prev: u32) -> Option<usize> {
    let fresh = held & !prev;
    k::SLOTS.iter().position(|bit| fresh & bit != 0)
}

pub fn toggle(shortcuts: &mut Shortcuts, fighter: &str, slot: usize, command: &str) -> Bound {
    let cleared = shortcuts.slot_of(fighter, command) == Some(slot);
    shortcuts.assign(fighter, slot, command);
    if cleared {
        Bound::Cleared(slot)
    } else {
        Bound::Set(slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_digits_map_to_slots() {
        for (i, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4].into_iter().enumerate() {
            let held = held_slots([], |kc| kc == key);
            assert_eq!(held, k::SLOTS[i]);
            assert_eq!(fresh_slot(held, 0), Some(i));
        }
    }

    #[test]
    fn gamepad_shoulders_map_to_slots_and_ignore_other_bits() {
        let held = held_slots([k::SLOT3 | k::X | k::U], |_| false);
        assert_eq!(held, k::SLOT3);
        assert_eq!(held_slots([k::SLOT1, k::SLOT4], |_| false), k::SLOT1 | k::SLOT4);
    }

    #[test]
    fn a_held_slot_fires_once() {
        assert_eq!(fresh_slot(k::SLOT2, 0), Some(1));
        assert_eq!(fresh_slot(k::SLOT2, k::SLOT2), None);
        assert_eq!(fresh_slot(k::SLOT2 | k::SLOT4, k::SLOT2), Some(3));
        assert_eq!(fresh_slot(0, k::SLOT1), None);
    }

    #[test]
    fn toggle_sets_moves_and_clears() {
        let mut s = Shortcuts::default();
        assert_eq!(toggle(&mut s, "kfm", 0, "fireball"), Bound::Set(0));
        assert_eq!(toggle(&mut s, "kfm", 3, "fireball"), Bound::Set(3));
        assert_eq!(s.slot_of("kfm", "fireball"), Some(3));
        assert_eq!(s.command("kfm", 0), None);
        assert_eq!(toggle(&mut s, "kfm", 3, "fireball"), Bound::Cleared(3));
        assert_eq!(s, Shortcuts::default());
    }

    #[test]
    fn rebinding_a_slot_replaces_the_previous_move() {
        let mut s = Shortcuts::default();
        toggle(&mut s, "kfm", 1, "a");
        toggle(&mut s, "kfm", 1, "b");
        assert_eq!(s.command("kfm", 1), Some("b"));
        assert_eq!(s.slot_of("kfm", "a"), None);
    }

    #[test]
    fn bindings_survive_a_save_round_trip() {
        let mut s = Shortcuts::default();
        toggle(&mut s, "Lian Feng", 2, "jade_lantern");
        assert_eq!(Shortcuts::from_text(&s.to_text()), s);
    }
}
