use super::touch::{BUTTONS, CORE_BUTTONS};
use crate::gfx::*;

pub struct Layout {
    pub pos: [Vec2; BUTTONS],
    pub radius: [f32; BUTTONS],
    pub stick: Vec2,
    pub stick_radius: f32,
    pub toggle: Vec2,
    pub toggle_radius: f32,
    pub left_handed: bool,
}

#[cfg(target_arch = "wasm32")]
fn safe_insets() -> [f32; 4] {
    crate::web_input::safe_insets()
}

#[cfg(not(target_arch = "wasm32"))]
fn safe_insets() -> [f32; 4] {
    [0.0; 4]
}

pub fn compute() -> Layout {
    let (w, h) = (screen_width(), screen_height());
    let tweaks = crate::tweaks::current();
    let left_handed = tweaks.touch_left_handed;
    let [_, right, bottom, left] = safe_insets();
    let (cluster_edge, stick_edge) = if left_handed {
        (left, right)
    } else {
        (right, left)
    };
    let r = (h * 0.075).clamp(20.0, 44.0) * tweaks.touch_scale;
    let gap = r * 2.1;
    let small = r * 0.72;
    let small_gap = small * 2.3;
    let base_x = w - cluster_edge - r * 1.4 - gap * 2.0;
    let base_y = h - bottom - r * 2.3;
    let mut pos = [Vec2::ZERO; BUTTONS];
    let mut radius = [small; BUTTONS];
    for i in 0..CORE_BUTTONS {
        pos[i] = vec2(
            base_x + (i % 3) as f32 * gap,
            base_y - (1 - i / 3) as f32 * gap,
        );
        radius[i] = r;
    }
    let extras_right = base_x - gap * 0.85;
    for (n, i) in (CORE_BUTTONS..BUTTONS).enumerate() {
        let (col, row) = ((n % 4) as f32, (n / 4) as f32);
        pos[i] = vec2(
            extras_right - (3.0 - col) * small_gap,
            base_y - row * small_gap,
        );
    }
    let mut toggle = vec2(base_x + gap * 2.0, base_y - gap * 2.0 + r * 0.2);
    let mut stick = vec2(stick_edge + r * 2.8, h - bottom - r * 3.1);
    if left_handed {
        pos.iter_mut().for_each(|p| p.x = w - p.x);
        toggle.x = w - toggle.x;
        stick.x = w - stick.x;
    }
    Layout {
        pos,
        radius,
        stick,
        stick_radius: r * 2.0,
        toggle,
        toggle_radius: r * 0.5,
        left_handed,
    }
}

impl Layout {
    pub fn in_stick_zone(&self, p: Vec2) -> bool {
        let (w, h) = (screen_width(), screen_height());
        let side = if self.left_handed {
            p.x > w * 0.55
        } else {
            p.x < w * 0.45
        };
        side && p.y > h * 0.25
    }
}
