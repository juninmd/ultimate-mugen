//! Keyboard bindings for the fighting inputs (P1 / P2), persisted next to the executable.
//! Menu navigation (arrows / WASD / Enter / Esc) is deliberately not rebindable so a bad mapping can never lock the player out.
use crate::gfx::KeyCode;
use ume_engine::input::k;

mod keys;
use keys::KEYS;

/// Rebindable fighting actions: label and engine input bit.
pub const ACTIONS: [(&str, u32); 16] = [
    ("Cima", k::U),
    ("Baixo", k::D),
    ("Esquerda", k::L),
    ("Direita", k::R),
    ("Soco fraco (LP)", k::X),
    ("Soco forte (HP)", k::Y),
    ("Soco 3", k::Z),
    ("Chute fraco (LK)", k::A),
    ("Chute forte (HK)", k::B),
    ("Chute 3", k::C),
    ("Especial 1", k::D2),
    ("Especial 2", k::W),
    ("Especial 3", k::E),
    ("Despertar", k::T),
    ("Troca (Tag)", k::TAG),
    ("Start (carregar ki)", k::S),
];
const N: usize = ACTIONS.len();

pub fn id(key: KeyCode) -> Option<&'static str> {
    KEYS.iter().find(|e| e.0 == key).map(|e| e.1)
}
pub fn label(key: KeyCode) -> &'static str {
    KEYS.iter().find(|e| e.0 == key).map(|e| e.2).unwrap_or("?")
}
fn from_id(s: &str) -> Option<KeyCode> {
    KEYS.iter().find(|e| e.1 == s).map(|e| e.0)
}

#[derive(Clone, PartialEq, Debug)]
pub struct KeyMap {
    pub slots: [[KeyCode; N]; 2],
}

impl Default for KeyMap {
    fn default() -> Self {
        use KeyCode::*;
        KeyMap {
            slots: [
                [
                    Up, Down, Left, Right, Q, W, E, A, S, D, Key1, Key2, Key3, T, R, Enter,
                ],
                [
                    I, K, J, L, Kp4, Kp5, Kp6, Kp1, Kp2, Kp3, Kp7, Kp8, Kp9, Kp0, KpEnter, KpDecimal,
                ],
            ],
        }
    }
}

fn previous_defaults() -> [[KeyCode; N]; 2] {
    use KeyCode::*;
    [
        [W, S, A, D, U, I, O, J, K, L, Key1, Key2, Key3, T, R, Enter],
        [
            Up, Down, Left, Right, Kp4, Kp5, Kp6, Kp1, Kp2, Kp3, Kp7, Kp8, Kp9, Kp0, KpEnter, KpDecimal,
        ],
    ]
}

impl KeyMap {
    /// Input bits held by `slot` given a key-down predicate.
    pub fn bits(&self, slot: usize, down: impl Fn(KeyCode) -> bool) -> u32 {
        self.slots[slot.min(1)]
            .iter()
            .zip(ACTIONS.iter())
            .fold(0, |b, (kc, a)| if down(*kc) { b | a.1 } else { b })
    }
    /// Bind `key` to action `i` of `slot`; if the slot already uses it, the two actions swap keys (never two actions on one key).
    pub fn assign(&mut self, slot: usize, i: usize, key: KeyCode) {
        let s = &mut self.slots[slot.min(1)];
        if let Some(j) = s.iter().position(|k| *k == key) {
            s[j] = s[i];
        }
        s[i] = key;
    }
    pub fn reset_slot(&mut self, slot: usize) {
        self.slots[slot.min(1)] = KeyMap::default().slots[slot.min(1)];
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for (p, s) in self.slots.iter().enumerate() {
            for (i, kc) in s.iter().enumerate() {
                out += &format!(
                    "p{}.{}={}
",
                    p + 1,
                    i,
                    id(*kc).unwrap_or("")
                );
            }
        }
        out
    }
    /// Unknown names / out-of-range lines are ignored; a slot that ends up with a duplicate key falls back to its defaults.
    pub fn from_text(t: &str) -> Self {
        let mut m = KeyMap::default();
        for l in t.lines() {
            let Some((k, v)) = l.split_once('=') else {
                continue;
            };
            let Some((p, i)) = k.trim().split_once('.') else {
                continue;
            };
            let (Some(p), Ok(i), Some(kc)) = (
                p.strip_prefix('p').and_then(|p| p.parse::<usize>().ok()),
                i.parse::<usize>(),
                from_id(v.trim()),
            ) else {
                continue;
            };
            if (1..=2).contains(&p) && i < N {
                m.slots[p - 1][i] = kc;
            }
        }
        let old = previous_defaults();
        for p in 0..2 {
            if m.slots[p] == old[p] {
                m.reset_slot(p);
            }
        }
        for p in 0..2 {
            let s = m.slots[p];
            if (0..N).any(|a| (a + 1..N).any(|b| s[a] == s[b])) {
                m.reset_slot(p);
            }
        }
        m
    }
    pub fn load() -> Self {
        crate::storage::read("ultimate-mugen.keys")
            .map(|t| Self::from_text(&t))
            .unwrap_or_default()
    }
    pub fn save(&self) {
        crate::storage::write("ultimate-mugen.keys", &self.to_text());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut old = KeyMap::default();
        old.slots = previous_defaults();
        assert_eq!(KeyMap::from_text(&old.to_text()), KeyMap::default());
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

    #[test]
    fn start_button_is_rebindable_and_defaults_to_enter() {
        let mut m = KeyMap::default();
        assert_eq!(m.bits(0, |k| k == KeyCode::Enter), k::S);
        m.assign(0, ACTIONS.len() - 1, KeyCode::Space);
        assert_eq!(m.bits(0, |k| k == KeyCode::Space), k::S);
        assert_eq!(m.bits(0, |k| k == KeyCode::Enter), 0);
        assert_eq!(KeyMap::from_text(&m.to_text()).slots[0][ACTIONS.len() - 1], KeyCode::Space);
    }

    #[test]
    fn saved_file_without_the_start_entry_keeps_the_default_start_key() {
        let m = KeyMap::from_text("p1.0=Up\np1.14=R\n");
        assert_eq!(m.slots[0][ACTIONS.len() - 1], KeyCode::Enter);
    }}
