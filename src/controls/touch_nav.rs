use super::nav;
use super::touch_layout;
use super::touch_style;
use crate::gfx::*;

const ARROWS: [(&str, u32, Vec2); 4] = [
    ("^", nav::UP, vec2(0.0, -1.0)),
    ("v", nav::DOWN, vec2(0.0, 1.0)),
    ("<", nav::LEFT, vec2(-1.0, 0.0)),
    (">", nav::RIGHT, vec2(1.0, 0.0)),
];

#[derive(Default)]
pub struct TouchNav {
    pub active: bool,
    held: u32,
    touch_seen: bool,
}

struct Button {
    bit: u32,
    center: Vec2,
    radius: f32,
    label: &'static str,
    color: Color,
}

fn buttons() -> Vec<Button> {
    let lay = touch_layout::compute();
    let r = lay.stick_radius * 0.5;
    let mut out: Vec<Button> = ARROWS
        .iter()
        .map(|(label, bit, dir)| Button {
            bit: *bit,
            center: lay.stick + *dir * r * 1.7,
            radius: r * 0.85,
            label,
            color: Color::new(0.8, 0.8, 0.9, 0.5),
        })
        .collect();
    out.push(Button {
        bit: nav::OK,
        center: lay.pos[5],
        radius: lay.radius[5],
        label: "OK",
        color: Color::new(0.24, 0.9, 0.6, 0.5),
    });
    out.push(Button {
        bit: nav::BACK,
        center: lay.pos[3],
        radius: lay.radius[3],
        label: crate::i18n::tr3("VOLTAR", "BACK", "VOLVER"),
        color: Color::new(1.0, 0.3, 0.4, 0.5),
    });
    out
}

fn hit(b: &Button, p: Vec2) -> bool {
    b.center.distance(p) < b.radius * 1.25
}

impl TouchNav {
    pub fn held(&self) -> u32 {
        self.held
    }

    pub fn update(&mut self, enabled: bool) {
        self.held = 0;
        if !enabled || !self.active {
            return;
        }
        let dpi = dpi_scale().max(0.01);
        let touched = touches();
        self.touch_seen |= !touched.is_empty();
        let mut points: Vec<(Vec2, bool)> = touched
            .iter()
            .filter(|t| t.phase != TouchPhase::Cancelled)
            .map(|t| (t.position / dpi, t.phase != TouchPhase::Moved))
            .collect();
        if touched.is_empty() && !self.touch_seen && is_mouse_button_down(MouseButton::Left) {
            points.push((mouse_position().into(), is_mouse_button_pressed(MouseButton::Left)));
        }
        let pad = buttons();
        let mut swallow = false;
        for (p, started) in points {
            for b in pad.iter().filter(|b| hit(b, p)) {
                self.held |= b.bit;
                swallow |= started;
            }
        }
        if swallow {
            swallow_pointer();
        }
    }

    pub fn draw(&self, enabled: bool) {
        if !enabled || !self.active {
            return;
        }
        let opacity = crate::tweaks::current().touch_opacity;
        ume_shell::alpha_scale::with(opacity, || {
            for b in buttons() {
                touch_style::draw_button(b.center, b.radius, b.label, b.color, self.held & b.bit != 0);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrows_cover_the_four_directions_once() {
        let bits = ARROWS.iter().fold(0, |m, (_, b, _)| m | b);
        assert_eq!(bits, nav::UP | nav::DOWN | nav::LEFT | nav::RIGHT);
        assert_eq!(ARROWS.iter().map(|(_, b, _)| b.count_ones()).sum::<u32>(), 4);
    }
}
