//! Keyboard bindings for the fighting inputs (P1 / P2), persisted next to the executable.
//! Menu navigation (arrows / WASD / Enter / Esc) is deliberately not rebindable so a bad mapping can never lock the player out.
use crate::gfx::KeyCode;
use ume_engine::input::k;

mod keys;
use keys::KEYS;

/// Rebindable fighting actions: label and engine input bit.
pub const ACTIONS: [(&str, u32); 19] = [
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
    ("Carregar ki (Start)", k::S),
    ("Andar p/ o fundo (Story)", k::LANE_UP),
    ("Andar p/ a frente (Story)", k::LANE_DOWN),
    ("Defender (Story)", k::GUARD),
];
const START: usize = 15;
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
                    Up, Down, Left, Right, Q, W, E, A, S, D, Key1, Key2, Key3, T, F, R, Z, X, C,
                ],
                [
                    I, K, J, L, Kp4, Kp5, Kp6, Kp1, Kp2, Kp3, Kp7, Kp8, Kp9, Kp0, KpEnter, KpDecimal, KpSubtract, KpAdd, KpMultiply,
                ],
            ],
        }
    }
}

fn previous_defaults() -> [[KeyCode; N]; 2] {
    use KeyCode::*;
    [
        [W, S, A, D, U, I, O, J, K, L, Key1, Key2, Key3, T, F, R, Z, X, C],
        [
            Up, Down, Left, Right, Kp4, Kp5, Kp6, Kp1, Kp2, Kp3, Kp7, Kp8, Kp9, Kp0, KpEnter, KpDecimal, KpSubtract, KpAdd, KpMultiply,
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
        let mut has_start = [false; 2];
        let mut present = [[false; N]; 2];
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
                present[p - 1][i] = true;
                if i == START { has_start[p - 1] = true; }
            }
        }
        for p in 0..2 {
            if !has_start[p] {
                if p == 0 && m.slots[p][14] == KeyCode::R {
                    if let Some(key) = [KeyCode::F].into_iter().chain(KEYS.iter().map(|entry| entry.0)).find(|key| *key != KeyCode::R && !m.slots[p][..14].contains(key)) {
                        m.slots[p][14] = key;
                    }
                }
                if m.slots[p][..START].contains(&m.slots[p][START]) {
                    if let Some(key) = KEYS.iter().map(|entry| entry.0).find(|key| !m.slots[p][..START].contains(key)) { m.slots[p][START] = key; }
                }
            }
            for i in START + 1..N {
                if !present[p][i] && m.slots[p][..i].contains(&m.slots[p][i]) {
                    if let Some(key) = KEYS.iter().map(|entry| entry.0).find(|key| !m.slots[p].contains(key)) { m.slots[p][i] = key; }
                }
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
mod tests;
