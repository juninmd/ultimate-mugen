//! Input hub: keyboard maps, gamepads (with rumble), touch controls; menu navigation.
use crate::gamepad::{PadInfo, Pads};
use macroquad::prelude::*;
use ume_engine::input::k;

pub struct TouchPad { pub enabled: bool, pub bits: u32, pub macro_q: std::collections::VecDeque<u32>, stick_center: Option<Vec2>, pub stick_pos: Vec2, pub pressed: [bool; 8], hold: [u8; 8] }

pub const TOUCH_BUTTONS: [(&str, u32); 8] = [("LP", k::X), ("HP", k::Y), ("SP1", k::D2), ("LK", k::A), ("HK", k::B), ("SP2", k::W), ("SP3", k::E), ("SUPER", k::D2 | k::W)];

impl TouchPad {
    pub fn new(enabled: bool) -> Self { TouchPad { enabled, bits: 0, macro_q: Default::default(), stick_center: None, stick_pos: Vec2::ZERO, pressed: [false; 8], hold: [0; 8] } }
    /// Button centers in screen space (right side), radius.
    pub fn layout() -> (Vec<Vec2>, f32, Vec2, f32) {
        let (w, h) = (screen_width(), screen_height());
        let r = (h * 0.075).min(46.0).max(22.0);
        let gap = r * 2.35;
        let bx = w - r * 1.4 - gap * 2.0; let by = h - r * 1.4 - gap;
        let v = (0..8).map(|i| vec2(bx + (i % 3) as f32 * gap, by + ((i / 3) as f32 - 1.0) * gap)).collect();
        let stick = vec2(r * 3.2, h - r * 3.2);
        (v, r, stick, r * 2.3)
    }
    pub fn update(&mut self) {
        if !self.enabled { self.bits = 0; return; }
        let (btns, r, stick, sr) = Self::layout();
        // macroquad reports touches in physical pixels while layout/screen_width() are logical (dpi-scaled)
        let dpi = macroquad::miniquad::window::dpi_scale().max(0.01);
        let all: Vec<(Vec2, bool)> = touches().iter().map(|t| (t.position / dpi, matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled))).collect();
        let mut points: Vec<(Vec2, bool)> = all;
        if points.is_empty() && is_mouse_button_down(MouseButton::Left) { points.push((mouse_position().into(), false)); }
        let mut bits = 0;
        self.pressed = [false; 8];
        let mut stick_touch = None;
        for h in self.hold.iter_mut() { *h = h.saturating_sub(1); }
        for (p, ended) in &points {
            if p.x < screen_width() * 0.45 && p.y > screen_height() * 0.3 { if !ended { stick_touch = Some(*p); } continue; }
            for (i, c) in btns.iter().enumerate() {
                if c.distance(*p) < r * 1.15 {
                    // latch: a tap that starts and ends within one frame still registers for a few frames
                    if *ended { self.hold[i] = self.hold[i].max(3); } else { self.hold[i] = self.hold[i].max(1); }
                }
            }
        }
        for i in 0..8 { if self.hold[i] > 0 { bits |= TOUCH_BUTTONS[i].1; self.pressed[i] = true; } }
        if let Some(p) = stick_touch {
            let c = *self.stick_center.get_or_insert(stick);
            let d = p - c;
            let len = d.length();
            self.stick_pos = if len > sr * 0.6 { d / len * sr * 0.6 } else { d };
            if len > sr * 0.2 {
                let a = d.y.atan2(d.x);
                let oct = (a / (std::f32::consts::PI / 4.0)).round() as i32;
                bits |= match oct { 0 => k::R, 1 => k::R | k::D, 2 => k::D, 3 => k::D | k::L, 4 | -4 => k::L, -3 => k::U | k::L, -2 => k::U, -1 => k::U | k::R, _ => 0 };
            }
        } else { self.stick_center = None; self.stick_pos = Vec2::ZERO; }
        self.bits = bits;
    }
    pub fn draw(&self) {
        if !self.enabled { return; }
        let (btns, r, stick, sr) = Self::layout();
        let c = self.stick_center.unwrap_or(stick);
        draw_circle(c.x, c.y, sr, Color::new(1.0, 1.0, 1.0, 0.06));
        draw_circle_lines(c.x, c.y, sr, 2.0, Color::new(1.0, 1.0, 1.0, 0.25));
        draw_circle(c.x + self.stick_pos.x, c.y + self.stick_pos.y, sr * 0.42, Color::new(1.0, 1.0, 1.0, 0.3));
        for (i, p) in btns.iter().enumerate() {
            let (label, _) = TOUCH_BUTTONS[i];
            let base = match label { "LP" | "HP" => Color::new(0.25, 0.66, 1.0, 0.3), "LK" | "HK" => Color::new(1.0, 0.3, 0.43, 0.3), "SUPER" => Color::new(0.6, 0.2, 0.9, 0.45), _ => Color::new(1.0, 0.8, 0.2, 0.25) };
            draw_circle(p.x, p.y, r, if self.pressed[i] { Color::new(1.0, 1.0, 1.0, 0.5) } else { base });
            draw_circle_lines(p.x, p.y, r, 2.0, Color::new(1.0, 1.0, 1.0, 0.4));
            let fs = (r * 0.55) as u16;
            let d = measure_text(label, None, fs, 1.0);
            draw_text(label, p.x - d.width / 2.0, p.y + d.height / 3.0, fs as f32, WHITE);
        }
    }
}

pub struct Controls { pub pads: Pads, pub pad_info: Vec<PadInfo>, prev_menu: u32, pub menu_pressed: u32, pub touch: TouchPad }

/// Keyboard maps
fn kb_bits(p2_layout: bool, solo: bool) -> u32 {
    let mut b = 0;
    let mut map = |keys: &[KeyCode], bit: u32| { if keys.iter().any(|kc| is_key_down(*kc)) { b |= bit; } };
    if !p2_layout {
        map(&[KeyCode::W], k::U); map(&[KeyCode::S], k::D); map(&[KeyCode::A], k::L); map(&[KeyCode::D], k::R);
        map(&[KeyCode::U], k::X); map(&[KeyCode::I], k::Y); map(&[KeyCode::O], k::Z); map(&[KeyCode::J], k::A); map(&[KeyCode::K], k::B); map(&[KeyCode::L], k::C);
        map(&[KeyCode::Key1], k::D2); map(&[KeyCode::Key2], k::W); map(&[KeyCode::Key3], k::E);
        if solo {
            map(&[KeyCode::Up], k::U); map(&[KeyCode::Down], k::D); map(&[KeyCode::Left], k::L); map(&[KeyCode::Right], k::R);
            map(&[KeyCode::Z], k::A); map(&[KeyCode::X], k::B); map(&[KeyCode::C], k::C); map(&[KeyCode::V], k::X); map(&[KeyCode::B], k::Y); map(&[KeyCode::N], k::Z);
        }
    } else {
        map(&[KeyCode::Up], k::U); map(&[KeyCode::Down], k::D); map(&[KeyCode::Left], k::L); map(&[KeyCode::Right], k::R);
        map(&[KeyCode::Kp4], k::X); map(&[KeyCode::Kp5], k::Y); map(&[KeyCode::Kp6], k::Z); map(&[KeyCode::Kp1], k::A); map(&[KeyCode::Kp2], k::B); map(&[KeyCode::Kp3], k::C);
        map(&[KeyCode::Kp7], k::D2); map(&[KeyCode::Kp8], k::W); map(&[KeyCode::Kp9], k::E);
    }
    b
}

/// Menu actions
pub mod nav { pub const UP: u32 = 1; pub const DOWN: u32 = 2; pub const LEFT: u32 = 4; pub const RIGHT: u32 = 8; pub const OK: u32 = 16; pub const BACK: u32 = 32; pub const START: u32 = 64; pub const AUX: u32 = 128; }

impl Controls {
    pub fn new(touch: bool) -> Self { Controls { pads: Pads::new(), pad_info: vec![], prev_menu: 0, menu_pressed: 0, touch: TouchPad::new(touch) } }
    pub fn update(&mut self) {
        self.pad_info = self.pads.update();
        self.touch.update();
        let mut m = 0;
        let key = |ks: &[KeyCode]| ks.iter().any(|kc| is_key_down(*kc));
        if key(&[KeyCode::Up, KeyCode::W]) { m |= nav::UP; }
        if key(&[KeyCode::Down, KeyCode::S]) { m |= nav::DOWN; }
        if key(&[KeyCode::Left, KeyCode::A]) { m |= nav::LEFT; }
        if key(&[KeyCode::Right, KeyCode::D]) { m |= nav::RIGHT; }
        if key(&[KeyCode::Enter, KeyCode::Space, KeyCode::J, KeyCode::U, KeyCode::Kp1]) { m |= nav::OK; }
        if key(&[KeyCode::Escape, KeyCode::Backspace, KeyCode::K]) { m |= nav::BACK; }
        if key(&[KeyCode::F2]) { m |= nav::AUX; }
        for p in &self.pad_info {
            let b = p.bits;
            if b & k::U != 0 { m |= nav::UP; } if b & k::D != 0 { m |= nav::DOWN; } if b & k::L != 0 { m |= nav::LEFT; } if b & k::R != 0 { m |= nav::RIGHT; }
            if b & k::A != 0 { m |= nav::OK; } if b & k::B != 0 { m |= nav::BACK; } if b & k::S != 0 { m |= nav::START; }
        }
        self.menu_pressed = m & !self.prev_menu;
        self.prev_menu = m;
    }
    pub fn pressed(&self, a: u32) -> bool { self.menu_pressed & a != 0 }
    /// Fighting input bits for a human player slot (0/1). `duo` = two humans share the keyboard.
    pub fn player_bits(&mut self, slot: usize, duo: bool) -> u32 {
        let mut b = kb_bits(slot == 1, !duo);
        if let Some(p) = self.pad_info.get(slot) { b |= p.bits; }
        if slot == 0 {
            b |= self.touch.bits;
            if let Some(m) = self.touch.macro_q.pop_front() { b |= m; }
        }
        b
    }
    pub fn rumble(&mut self, slot: usize, strong: f32, weak: f32, ms: u32) {
        if slot < self.pad_info.len() { self.pads.rumble(slot, strong, weak, ms); }
        #[cfg(target_arch = "wasm32")]
        if slot == 0 && self.touch.enabled { self.pads.rumble(0, strong, weak, ms); }
    }
}
