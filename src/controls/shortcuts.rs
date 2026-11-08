use crate::gfx::KeyCode;
use std::collections::BTreeMap;
use ume_engine::input::k;

pub const SLOT_COUNT: usize = 4;
const FILE: &str = "ultimate-mugen.shortcuts";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shortcuts {
    by_fighter: BTreeMap<String, [String; SLOT_COUNT]>,
    /// slot bits held last frame, so a binding fires once per press
    pub prev: [u32; 2],
}

/// Engine bits of the keyboard shortcut keys 1-4 that are held.
pub fn key_bits(down: impl Fn(KeyCode) -> bool) -> u32 {
    [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
        .into_iter()
        .zip(k::SLOTS)
        .fold(0, |b, (kc, bit)| if down(kc) { b | bit } else { b })
}

impl Shortcuts {
    pub fn command(&self, fighter: &str, slot: usize) -> Option<&str> {
        self.by_fighter
            .get(fighter)
            .and_then(|s| s.get(slot))
            .map(String::as_str)
            .filter(|c| !c.is_empty())
    }

    pub fn slot_of(&self, fighter: &str, command: &str) -> Option<usize> {
        self.by_fighter
            .get(fighter)
            .and_then(|s| s.iter().position(|c| c == command))
    }

    pub fn assign(&mut self, fighter: &str, slot: usize, command: &str) {
        if slot >= SLOT_COUNT {
            return;
        }
        let slots = self.by_fighter.entry(fighter.to_string()).or_default();
        if slots[slot] == command {
            slots[slot].clear();
        } else {
            for s in slots.iter_mut().filter(|s| *s == command) {
                s.clear();
            }
            slots[slot] = command.to_string();
        }
        if slots.iter().all(String::is_empty) {
            self.by_fighter.remove(fighter);
        }
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for (name, slots) in &self.by_fighter {
            for (i, c) in slots.iter().enumerate().filter(|(_, c)| !c.is_empty()) {
                out += &format!("{i}\t{name}\t{c}\n");
            }
        }
        out
    }

    pub fn from_text(t: &str) -> Self {
        let mut s = Shortcuts::default();
        for l in t.lines() {
            let mut p = l.splitn(3, '\t');
            let (Some(i), Some(name), Some(c)) = (p.next(), p.next(), p.next()) else {
                continue;
            };
            let Ok(i) = i.parse::<usize>() else { continue };
            if i < SLOT_COUNT && !name.is_empty() && !c.is_empty() {
                s.by_fighter.entry(name.to_string()).or_default()[i] = c.to_string();
            }
        }
        s
    }

    pub fn load() -> Self {
        crate::storage::read(FILE)
            .map(|t| Self::from_text(&t))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        crate::storage::write(FILE, &self.to_text());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigning_moves_a_command_between_slots_and_toggles_it_off() {
        let mut s = Shortcuts::default();
        s.assign("kfm", 0, "fireball");
        s.assign("kfm", 2, "fireball");
        assert_eq!(s.command("kfm", 0), None);
        assert_eq!(s.command("kfm", 2), Some("fireball"));
        assert_eq!(s.slot_of("kfm", "fireball"), Some(2));
        s.assign("kfm", 2, "fireball");
        assert_eq!(s, Shortcuts::default());
    }

    #[test]
    fn text_round_trip_and_garbage_lines() {
        let mut s = Shortcuts::default();
        s.assign("kfm", 1, "kamehameha");
        s.assign("goku", 3, "spirit_bomb");
        let back = Shortcuts::from_text(&format!("{}junk\n9\tx\ty\n", s.to_text()));
        assert_eq!(back, s);
    }

    #[test]
    fn out_of_range_slot_is_ignored() {
        let mut s = Shortcuts::default();
        s.assign("kfm", SLOT_COUNT, "x");
        assert_eq!(s, Shortcuts::default());
    }
}
