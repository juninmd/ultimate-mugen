use crate::gfx::*;
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
