use crate::gfx::*;
use ume_engine::input::k;
use super::{touch_layout, touch_style};

mod fingers;
use fingers::{Fingers, Sample};

pub struct TouchPad {
    pub enabled: bool,
    pub bits: u32,
    pub macro_q: [std::collections::VecDeque<u32>; 2],
    touch_seen: bool,
    fingers: Fingers,
    stick_center: Option<Vec2>,
    pub stick_pos: Vec2,
    pub began: Vec<Vec2>,
    pub pressed: [bool; BUTTONS],
    hold: [u8; BUTTONS],
    pub more_open: bool,
    idle_frames: u32,
    age: [u8; BUTTONS],
    probe: Option<String>,
    probe_on: bool,
    pub suspended: bool,
}

pub const QUICK_SKILLS: usize = 4;
pub const CHARGE: usize = 7;
pub const QUICK_START: usize = 8;
pub const EXTRA_START: usize = QUICK_START + QUICK_SKILLS;
pub const BUTTONS: usize = EXTRA_START + 5;
pub const CORE_BUTTONS: usize = 6;
const IDLE_FADE_FRAMES: u32 = 150;
const IDLE_ALPHA: f32 = 0.3;

pub const TOUCH_BUTTONS: [(&str, u32); BUTTONS] = [
    ("SOCO", k::X),
    ("SOCO+", k::Y),
    ("ESPECIAL", k::D2),
    ("CHUTE", k::A),
    ("CHUTE+", k::B),
    ("SUPER", k::D2 | k::W),
    ("DESPERTAR", k::T),
    ("CARREGAR", k::S),
    ("1", k::SLOT1),
    ("2", k::SLOT2),
    ("3", k::SLOT3),
    ("4", k::SLOT4),
    ("TROCA", k::TAG),
    ("ESPECIAL 3", k::E),
    ("SOCO 3", k::Z),
    ("CHUTE 3", k::C),
    ("DEFESA", k::GUARD),
];

const MOUSE_ID: u64 = u64::MAX;
const TAP_FRAMES: u8 = 3;

impl TouchPad {
    pub fn new(enabled: bool) -> Self {
        TouchPad {
            enabled,
            bits: 0,
            macro_q: Default::default(),
            touch_seen: false,
            fingers: Fingers::default(),
            stick_center: None,
            stick_pos: Vec2::ZERO,
            began: Vec::new(),
            pressed: [false; BUTTONS],
            hold: [0; BUTTONS],
            more_open: false,
            idle_frames: IDLE_FADE_FRAMES,
            age: [0; BUTTONS],
            probe: None,
            probe_on: crate::storage::read("touchdebug").is_some(),
            suspended: false,
        }
    }
    fn reset(&mut self) {
        self.began.clear();
        self.bits = 0;
        self.fingers = Fingers::default();
        self.stick_center = None;
        self.stick_pos = Vec2::ZERO;
        self.pressed = [false; BUTTONS];
        self.hold = [0; BUTTONS];
        self.age = [0; BUTTONS];
    }
    fn samples(&mut self, dpi: f32) -> Vec<Sample> {
        let touched = touches();
        if !touched.is_empty() {
            self.touch_seen = true;
        }
        self.began = touched
            .iter()
            .filter(|t| matches!(t.phase, TouchPhase::Started | TouchPhase::Ended))
            .map(|t| t.position / dpi)
            .collect();
        let mut samples: Vec<Sample> = touched
            .iter()
            .map(|t| Sample {
                id: t.id,
                pos: t.position / dpi,
                ended: matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled),
            })
            .collect();
        if samples.is_empty() && !self.touch_seen && is_mouse_button_down(MouseButton::Left) {
            samples.push(Sample { id: MOUSE_ID, pos: mouse_position().into(), ended: false });
        }
        samples
    }
    pub fn update(&mut self) {
        if !touches().is_empty() {
            self.enabled = true;
        }
        if !self.enabled {
            self.reset();
            return;
        }
        let samples = self.samples(dpi_scale().max(0.01));
        if self.suspended {
            let began = std::mem::take(&mut self.began);
            self.reset();
            self.began = began;
            return;
        }
        let lay = touch_layout::compute();
        let floating = crate::tweaks::current().touch_floating_stick;
        let out = self.fingers.update(&samples, &lay, self.more_open, floating);
        self.idle_frames = if samples.is_empty() { self.idle_frames.saturating_add(1) } else { 0 };
        if out.toggle {
            self.more_open = !self.more_open;
            self.hold = [0; BUTTONS];
        }
        let mut bits = 0;
        for i in 0..BUTTONS {
            self.hold[i] = self.hold[i].saturating_sub(1);
            self.age[i] = if out.held[i] { self.age[i].saturating_add(1) } else { 0 };
            if out.held[i] {
                self.hold[i] = self.hold[i].max(1);
            }
            if out.released[i] {
                self.hold[i] = self.hold[i].max(TAP_FRAMES.saturating_sub(self.age[i]));
            }
            self.pressed[i] = self.hold[i] > 0;
            if self.pressed[i] {
                bits |= TOUCH_BUTTONS[i].1;
            }
        }
        match out.stick {
            Some(s) => {
                self.stick_center = Some(s.origin);
                self.stick_pos = s.offset;
                bits |= s.bits;
            }
            None => {
                self.stick_center = None;
                self.stick_pos = Vec2::ZERO;
            }
        }
        self.bits = bits;
        self.publish_probe();
    }
    fn publish_probe(&mut self) {
        if !self.probe_on {
            return;
        }
        let mask = self.pressed.iter().enumerate().fold(0u32, |m, (i, p)| m | (*p as u32) << i);
        let line = format!("{} {} {}", self.bits, mask, self.more_open as u8);
        if self.probe.as_deref() != Some(line.as_str()) {
            crate::storage::write("touchdebug.out", &line);
            self.probe = Some(line);
        }
    }
    fn fade(&self) -> f32 {
        let over = self.idle_frames.saturating_sub(IDLE_FADE_FRAMES) as f32 / 30.0;
        1.0 - (1.0 - IDLE_ALPHA) * over.min(1.0)
    }
    pub fn draw(&self) {
        if !self.enabled || self.suspended {
            return;
        }
        let opacity = crate::tweaks::current().touch_opacity * self.fade();
        ume_shell::alpha_scale::with(opacity, || self.draw_pad());
    }

    fn draw_pad(&self) {
        let lay = touch_layout::compute();
        let c = self.stick_center.unwrap_or(lay.stick);
        touch_style::draw_stick(c, lay.stick_radius, self.stick_pos, self.stick_center.is_some());
        for i in (0..BUTTONS).filter(|i| i < &CORE_BUTTONS || self.more_open) {
            let base = touch_style::base_color(i);
            touch_style::draw_button(lay.pos[i], lay.radius[i], super::touch_label::button(TOUCH_BUTTONS[i]), base, self.pressed[i]);
        }
        let label = if self.more_open { "x" } else { "+" };
        let base = Color::new(0.6, 0.6, 0.65, 0.5);
        touch_style::draw_button(lay.toggle, lay.toggle_radius, label, base, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charge_button_presses_the_ki_charge_key() {
        assert_eq!(TOUCH_BUTTONS[CHARGE].1, k::S);
    }

    #[test]
    fn quick_skill_buttons_press_the_shortcut_slots() {
        let bits: Vec<u32> = TOUCH_BUTTONS[QUICK_START..EXTRA_START].iter().map(|(_, b)| *b).collect();
        assert_eq!(bits, k::SLOTS.to_vec(), "quick skills must press the shortcut slots the pause screen binds");
    }
}
