//! Input hub: keyboard maps, gamepads (with rumble), touch controls; menu navigation.
use crate::gamepad::{PadInfo, PadKind, Pads};
mod touch;
pub mod shortcuts;
use crate::gfx::*;
use crate::keybinds::KeyMap;
use shortcuts::Shortcuts;
use crate::ui::{hex, Face, Mark};
use ume_engine::input::k;

pub use touch::TouchPad;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Glyphs {
    Keyboard,
    Touch,
    Pad(PadKind),
}

pub fn face(g: Glyphs, keys: &KeyMap, bit: u32) -> Face {
    let blue = hex("#3a8cff");
    let red = hex("#ff4d6d");
    let dark = hex("#3b3552");
    let punch = bit & (k::X | k::Y | k::Z) != 0;
    match g {
        Glyphs::Touch => {
            let t = match bit {
                k::X => crate::i18n::tr3("SOCO", "PUNCH", "PUÑO"),
                k::Y => crate::i18n::tr3("SOCO+", "PUNCH+", "PUÑO+"),
                k::Z => crate::i18n::tr3("SOCO 3", "PUNCH 3", "PUÑO 3"),
                k::A => crate::i18n::tr3("CHUTE", "KICK", "PATA"),
                k::B => crate::i18n::tr3("CHUTE+", "KICK+", "PATA+"),
                k::C => crate::i18n::tr3("CHUTE 3", "KICK 3", "PATA 3"),
                k::D2 => crate::i18n::tr3("ESPECIAL", "SPECIAL", "ESPECIAL"),
                k::W => "SUPER",
                k::T => crate::i18n::tr3("DESPERTAR", "AWAKEN", "DESPERTAR"),
                _ => "II",
            };
            Face::chip(
                t,
                if punch {
                    blue
                } else if bit & (k::A | k::B | k::C) != 0 {
                    red
                } else {
                    hex("#ffb703")
                },
            )
        }
        Glyphs::Keyboard => {
            let i = crate::keybinds::ACTIONS.iter().position(|a| a.1 == bit);
            Face::key(
                i.map(|i| crate::keybinds::label(keys.slots[0][i]))
                    .unwrap_or("Esc"),
            )
        }
        Glyphs::Pad(PadKind::PlayStation) => match bit {
            k::X => Face::mark(Mark::Square, hex("#ff7ac8")),
            k::Y => Face::mark(Mark::Triangle, hex("#3ddc97")),
            k::A => Face::mark(Mark::Cross, hex("#7aa7ff")),
            k::B => Face::mark(Mark::Circle, hex("#ff5a6e")),
            k::Z => Face::chip("R1", dark),
            k::C => Face::chip("R2", dark),
            k::D2 => Face::chip("L1", dark),
            k::W => Face::chip("L2", dark),
            k::T => Face::chip("R3", dark),
            _ => Face::chip("OPTIONS", dark),
        },
        Glyphs::Pad(_) => match bit {
            k::X => Face::round("X", hex("#2f7bff")),
            k::Y => Face::round("Y", hex("#f5c400")),
            k::A => Face::round("A", hex("#3cb043")),
            k::B => Face::round("B", hex("#e53935")),
            k::Z => Face::chip("RB", dark),
            k::C => Face::chip("RT", dark),
            k::D2 => Face::chip("LB", dark),
            k::W => Face::chip("LT", dark),
            k::T => Face::chip("RS", dark),
            _ => Face::chip("MENU", dark),
        },
    }
}

pub struct Controls {
    pub keys: KeyMap,
    pub pads: Pads,
    pub pad_info: Vec<PadInfo>,
    prev_menu: u32,
    pub menu_pressed: u32,
    pub touch: TouchPad,
    pub glyphs: Glyphs,
    pub shortcuts: Shortcuts,
}

/// Keyboard bits for a slot from the (rebindable) map; a lone player also gets the fixed arrows + ZXCVBN layout as aliases.
fn kb_bits(keys: &KeyMap, slot: usize, solo: bool) -> u32 {
    let mut b = keys.bits(slot, is_key_down);
    if slot == 0 && solo {
        let mut map = |kc: KeyCode, bit: u32| {
            if is_key_down(kc) {
                b |= bit;
            }
        };
        map(KeyCode::Up, k::U);
        map(KeyCode::Down, k::D);
        map(KeyCode::Left, k::L);
        map(KeyCode::Right, k::R);
    }
    if slot == 0 {
        b |= shortcuts::key_bits(is_key_down);
    }
    b
}

/// Menu actions
pub mod nav {
    pub const UP: u32 = 1;
    pub const DOWN: u32 = 2;
    pub const LEFT: u32 = 4;
    pub const RIGHT: u32 = 8;
    pub const OK: u32 = 16;
    pub const BACK: u32 = 32;
    pub const START: u32 = 64;
    pub const AUX: u32 = 128;
}

impl Controls {
    pub fn new(touch: bool) -> Self {
        Controls {
            keys: KeyMap::load(),
            pads: Pads::new(),
            pad_info: vec![],
            prev_menu: 0,
            menu_pressed: 0,
            touch: TouchPad::new(touch),
            shortcuts: Shortcuts::load(),
            glyphs: if touch {
                Glyphs::Touch
            } else {
                Glyphs::Keyboard
            },
        }
    }
    pub fn face(&self, bit: u32) -> Face {
        face(self.glyphs, &self.keys, bit)
    }
    pub fn update(&mut self) {
        let had_pads = !self.pad_info.is_empty();
        self.pad_info = self.pads.update();
        self.touch.update();
        let fallback = if self.touch.enabled {
            Glyphs::Touch
        } else {
            Glyphs::Keyboard
        };
        if let Some(p) = self.pad_info.iter().find(|p| p.bits != 0) {
            self.glyphs = Glyphs::Pad(p.kind);
        } else if !had_pads && !self.pad_info.is_empty() {
            self.glyphs = Glyphs::Pad(self.pad_info[0].kind);
        } else if self.keys.bits(0, is_key_down) != 0
            || is_key_down(KeyCode::Enter)
            || is_key_down(KeyCode::Escape)
        {
            self.glyphs = Glyphs::Keyboard;
        } else if self.touch.enabled
            && (self.touch.bits != 0 || is_mouse_button_pressed(MouseButton::Left))
        {
            self.glyphs = Glyphs::Touch;
        } else if self.pad_info.is_empty() && matches!(self.glyphs, Glyphs::Pad(_)) {
            self.glyphs = fallback;
        }
        let mut m = 0;
        let key = |ks: &[KeyCode]| ks.iter().any(|kc| is_key_down(*kc));
        if key(&[KeyCode::Up, KeyCode::W]) {
            m |= nav::UP;
        }
        if key(&[KeyCode::Down, KeyCode::S]) {
            m |= nav::DOWN;
        }
        if key(&[KeyCode::Left, KeyCode::A]) {
            m |= nav::LEFT;
        }
        if key(&[KeyCode::Right, KeyCode::D]) {
            m |= nav::RIGHT;
        }
        if key(&[KeyCode::Enter, KeyCode::Space, KeyCode::Q, KeyCode::Kp1]) {
            m |= nav::OK;
        }
        if key(&[KeyCode::Escape, KeyCode::Backspace, KeyCode::W]) {
            m |= nav::BACK;
        }
        if key(&[KeyCode::F2]) {
            m |= nav::AUX;
        }
        for p in &self.pad_info {
            let b = p.bits;
            if b & k::U != 0 {
                m |= nav::UP;
            }
            if b & k::D != 0 {
                m |= nav::DOWN;
            }
            if b & k::L != 0 {
                m |= nav::LEFT;
            }
            if b & k::R != 0 {
                m |= nav::RIGHT;
            }
            if b & k::A != 0 {
                m |= nav::OK;
            }
            if b & k::B != 0 {
                m |= nav::BACK;
            }
            if b & k::S != 0 {
                m |= nav::START;
            }
        }
        self.menu_pressed = m & !self.prev_menu;
        self.prev_menu = m;
    }
    pub fn clear_macros(&mut self) {
        for q in self.touch.macro_q.iter_mut() {
            q.clear();
        }
    }
    pub fn pressed(&self, a: u32) -> bool {
        self.menu_pressed & a != 0
    }
    /// Fighting input bits for a human player slot (0/1). `duo` = two humans share the keyboard.
    pub fn player_bits(&mut self, slot: usize, duo: bool) -> u32 {
        let mut b = kb_bits(&self.keys, slot, !duo);
        if let Some(p) = self.pad_info.get(slot) {
            b |= p.bits;
        }
        if slot == 0 {
            b |= self.touch.bits;
        }
        if let Some(m) = self.touch.macro_q[slot.min(1)].pop_front() {
            b |= m;
        }
        b
    }
    pub fn rumble(&mut self, slot: usize, strong: f32, weak: f32, ms: u32) {
        if slot < self.pad_info.len() {
            self.pads.rumble(slot, strong, weak, ms);
        }
        #[cfg(target_arch = "wasm32")]
        if slot == 0 && self.touch.enabled {
            self.pads.rumble(0, strong, weak, ms);
        }
    }
}
