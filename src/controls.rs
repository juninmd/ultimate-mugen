//! Input hub: keyboard maps, gamepads (with rumble), touch controls; menu navigation.
use crate::gamepad::{PadInfo, PadKind, Pads};
use crate::gfx::*;
use crate::keybinds::KeyMap;
use crate::shortcuts::Shortcuts;
use crate::ui::{hex, Face, Mark};
use ume_engine::input::k;

pub struct TouchPad {
    pub enabled: bool,
    pub bits: u32,
    pub macro_q: [std::collections::VecDeque<u32>; 2],
    touch_seen: bool,
    stick_center: Option<Vec2>,
    pub stick_pos: Vec2,
    pub pressed: [bool; 7],
    hold: [u8; 7],
}

pub const TOUCH_BUTTONS: [(&str, u32); 7] = [
    ("SOCO", k::X),
    ("SOCO+", k::Y),
    ("ESPECIAL", k::D2),
    ("CHUTE", k::A),
    ("CHUTE+", k::B),
    ("SUPER", k::D2 | k::W),
    ("DESPERTAR", k::T),
];

impl TouchPad {
    pub fn new(enabled: bool) -> Self {
        TouchPad {
            enabled,
            bits: 0,
            macro_q: Default::default(),
            touch_seen: false,
            stick_center: None,
            stick_pos: Vec2::ZERO,
            pressed: [false; 7],
            hold: [0; 7],
        }
    }
    pub fn layout() -> (Vec<Vec2>, f32, Vec2, f32) {
        let (w, h) = (screen_width(), screen_height());
        let r = (h * 0.09).clamp(24.0, 52.0);
        let gap = r * 2.25;
        let (bx, by) = (w - r * 1.3 - gap * 2.0, h - r * 1.3);
        let mut v: Vec<Vec2> = (0..6)
            .map(|i| vec2(bx + (i % 3) as f32 * gap, by - (1 - i / 3) as f32 * gap))
            .collect();
        v.push(vec2(bx + gap, by - gap * 2.0));
        let stick = vec2(r * 3.0, h - r * 3.0);
        (v, r, stick, r * 2.2)
    }
    fn radius(i: usize, r: f32) -> f32 {
        if i == 6 {
            r * 0.7
        } else {
            r
        }
    }
    pub fn update(&mut self) {
        if !self.enabled {
            self.bits = 0;
            self.stick_center = None;
            self.stick_pos = Vec2::ZERO;
            self.pressed = [false; 7];
            self.hold = [0; 7];
            return;
        }
        let (btns, r, stick, sr) = Self::layout();
        let dpi = dpi_scale().max(0.01);
        let all: Vec<(Vec2, bool)> = touches()
            .iter()
            .map(|t| {
                (
                    t.position / dpi,
                    matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled),
                )
            })
            .collect();
        if !all.is_empty() {
            self.touch_seen = true;
        }
        let mut points: Vec<(Vec2, bool)> = all;
        if points.is_empty() && !self.touch_seen && is_mouse_button_down(MouseButton::Left) {
            points.push((mouse_position().into(), false));
        }
        let mut bits = 0;
        self.pressed = [false; 7];
        let mut stick_touch = None;
        for h in self.hold.iter_mut() {
            *h = h.saturating_sub(1);
        }
        for (p, ended) in &points {
            if p.x < screen_width() * 0.45 && p.y > screen_height() * 0.3 {
                if !ended {
                    stick_touch = Some(*p);
                }
                continue;
            }
            for (i, c) in btns.iter().enumerate() {
                if c.distance(*p) < Self::radius(i, r) * 1.1 {
                    if *ended {
                        self.hold[i] = self.hold[i].max(3);
                    } else {
                        self.hold[i] = self.hold[i].max(1);
                    }
                }
            }
        }
        for i in 0..7 {
            if self.hold[i] > 0 {
                bits |= TOUCH_BUTTONS[i].1;
                self.pressed[i] = true;
            }
        }
        if let Some(p) = stick_touch {
            let c = *self.stick_center.get_or_insert(stick);
            let d = p - c;
            let len = d.length();
            self.stick_pos = if len > sr * 0.6 {
                d / len * sr * 0.6
            } else {
                d
            };
            if len > sr * 0.2 {
                let a = d.y.atan2(d.x);
                let oct = (a / (std::f32::consts::PI / 4.0)).round() as i32;
                bits |= match oct {
                    0 => k::R,
                    1 => k::R | k::D,
                    2 => k::D,
                    3 => k::D | k::L,
                    4 | -4 => k::L,
                    -3 => k::U | k::L,
                    -2 => k::U,
                    -1 => k::U | k::R,
                    _ => 0,
                };
            }
        } else {
            self.stick_center = None;
            self.stick_pos = Vec2::ZERO;
        }
        self.bits = bits;
    }
    pub fn draw(&self) {
        if !self.enabled {
            return;
        }
        let (btns, r, stick, sr) = Self::layout();
        let c = self.stick_center.unwrap_or(stick);
        draw_circle(c.x, c.y, sr, Color::new(1.0, 1.0, 1.0, 0.08));
        draw_circle_lines(c.x, c.y, sr, 2.0, Color::new(1.0, 1.0, 1.0, 0.35));
        draw_circle(
            c.x + self.stick_pos.x,
            c.y + self.stick_pos.y,
            sr * 0.42,
            Color::new(1.0, 1.0, 1.0, 0.35),
        );
        for (i, p) in btns.iter().enumerate() {
            let (label, _) = TOUCH_BUTTONS[i];
            let rr = Self::radius(i, r);
            let base = match i {
                0 | 1 => Color::new(0.2, 0.55, 1.0, 0.45),
                3 | 4 => Color::new(1.0, 0.27, 0.4, 0.45),
                2 => Color::new(1.0, 0.75, 0.15, 0.45),
                5 => Color::new(0.6, 0.2, 0.95, 0.55),
                _ => Color::new(1.0, 0.45, 0.1, 0.5),
            };
            draw_circle(
                p.x,
                p.y,
                rr,
                if self.pressed[i] {
                    Color::new(1.0, 1.0, 1.0, 0.6)
                } else {
                    base
                },
            );
            draw_circle_lines(p.x, p.y, rr, 2.0, Color::new(1.0, 1.0, 1.0, 0.55));
            let mut fs = (rr * 0.5) as u16;
            while fs > 8 && measure_text(label, None, fs, 1.0).width > rr * 1.8 {
                fs -= 1;
            }
            let d = measure_text(label, None, fs, 1.0);
            draw_text(
                label,
                p.x - d.width / 2.0,
                p.y + d.offset_y / 2.0,
                fs as f32,
                WHITE,
            );
        }
    }
}

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
                k::X => "SOCO",
                k::Y => "SOCO+",
                k::Z => "SOCO 3",
                k::A => "CHUTE",
                k::B => "CHUTE+",
                k::C => "CHUTE 3",
                k::D2 => "ESPECIAL",
                k::W => "SUPER",
                k::T => "DESPERTAR",
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
    pub slot_prev: [u32; 2],
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
        for (kc, bit) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
            .into_iter()
            .zip(k::SLOTS)
        {
            if is_key_down(kc) {
                b |= bit;
            }
        }
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
            slot_prev: [0; 2],
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
