use crate::gfx::*;
use ume_engine::input::k;
use super::{touch_layout, touch_style};

pub struct TouchPad {
    pub enabled: bool,
    pub bits: u32,
    pub macro_q: [std::collections::VecDeque<u32>; 2],
    touch_seen: bool,
    stick_center: Option<Vec2>,
    pub stick_pos: Vec2,
    pub began: Vec<Vec2>,
    pub pressed: [bool; BUTTONS],
    hold: [u8; BUTTONS],
    pub more_open: bool,
    toggle_down: bool,
    idle_frames: u32,
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

impl TouchPad {
    pub fn new(enabled: bool) -> Self {
        TouchPad {
            enabled,
            bits: 0,
            macro_q: Default::default(),
            touch_seen: false,
            stick_center: None,
            stick_pos: Vec2::ZERO,
            began: Vec::new(),
            pressed: [false; BUTTONS],
            hold: [0; BUTTONS],
            more_open: false,
            toggle_down: false,
            idle_frames: IDLE_FADE_FRAMES,
        }
    }
    fn visible(&self, i: usize) -> bool {
        i < CORE_BUTTONS || self.more_open
    }
    fn reset(&mut self) {
        self.began.clear();
        self.bits = 0;
        self.stick_center = None;
        self.stick_pos = Vec2::ZERO;
        self.pressed = [false; BUTTONS];
        self.hold = [0; BUTTONS];
        self.toggle_down = false;
    }
    pub fn update(&mut self) {
        if !touches().is_empty() {
            self.enabled = true;
        }
        if !self.enabled {
            self.reset();
            return;
        }
        let lay = touch_layout::compute();
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
        self.began = touches()
            .iter()
            .filter(|t| matches!(t.phase, TouchPhase::Started | TouchPhase::Ended))
            .map(|t| t.position / dpi)
            .collect();
        let mut points = all;
        if points.is_empty() && !self.touch_seen && is_mouse_button_down(MouseButton::Left) {
            points.push((mouse_position().into(), false));
        }
        self.idle_frames = if points.is_empty() { self.idle_frames.saturating_add(1) } else { 0 };
        let mut bits = 0;
        self.pressed = [false; BUTTONS];
        let mut stick_touch = None;
        let mut toggle_held = false;
        for h in self.hold.iter_mut() {
            *h = h.saturating_sub(1);
        }
        for (p, ended) in &points {
            if lay.toggle.distance(*p) < lay.toggle_radius * 1.6 {
                toggle_held |= !ended;
                continue;
            }
            let hit = (0..BUTTONS)
                .filter(|i| self.visible(*i))
                .map(|i| (i, lay.pos[i].distance(*p) / lay.radius[i]))
                .filter(|(_, d)| *d < 1.25)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match hit {
                Some((i, _)) => self.hold[i] = self.hold[i].max(if *ended { 3 } else { 1 }),
                None if lay.in_stick_zone(*p) && !ended => stick_touch = Some(*p),
                None => {}
            }
        }
        if toggle_held && !self.toggle_down {
            self.more_open = !self.more_open;
            self.hold = [0; BUTTONS];
        }
        self.toggle_down = toggle_held;
        for i in 0..BUTTONS {
            if self.hold[i] > 0 {
                bits |= TOUCH_BUTTONS[i].1;
                self.pressed[i] = true;
            }
        }
        let sr = lay.stick_radius;
        if let Some(p) = stick_touch {
            let c = *self.stick_center.get_or_insert(lay.stick);
            let d = p - c;
            let len = d.length();
            self.stick_pos = if len > sr * 0.6 { d / len * sr * 0.6 } else { d };
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
    fn fade(&self) -> f32 {
        let over = self.idle_frames.saturating_sub(IDLE_FADE_FRAMES) as f32 / 30.0;
        1.0 - (1.0 - IDLE_ALPHA) * over.min(1.0)
    }
    pub fn draw(&self) {
        if !self.enabled {
            return;
        }
        let opacity = crate::tweaks::current().touch_opacity * self.fade();
        ume_shell::alpha_scale::with(opacity, || self.draw_pad());
    }

    fn draw_pad(&self) {
        let lay = touch_layout::compute();
        let c = self.stick_center.unwrap_or(lay.stick);
        touch_style::draw_stick(c, lay.stick_radius, self.stick_pos);
        for i in (0..BUTTONS).filter(|i| self.visible(*i)) {
            let base = touch_style::base_color(i);
            touch_style::draw_button(lay.pos[i], lay.radius[i], TOUCH_BUTTONS[i].0, base, self.pressed[i]);
        }
        let label = if self.more_open { "x" } else { "+" };
        let base = Color::new(0.6, 0.6, 0.65, 0.5);
        touch_style::draw_button(lay.toggle, lay.toggle_radius, label, base, self.toggle_down);
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
