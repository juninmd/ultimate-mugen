use crate::gfx::*;
use std::collections::VecDeque;
use ume_engine::input::k;

pub struct TouchPad {
    pub enabled: bool,
    pub bits: u32,
    pub nav: u32,
    pub began: Vec<Vec2>,
    pub macro_q: [VecDeque<u32>; 2],
}

#[cfg(target_arch = "wasm32")]
fn poll() -> Option<(u32, u32)> {
    crate::web_input::vpad()
}

#[cfg(not(target_arch = "wasm32"))]
fn poll() -> Option<(u32, u32)> {
    None
}

impl TouchPad {
    pub fn new(enabled: bool) -> Self {
        TouchPad { enabled, bits: 0, nav: 0, began: Vec::new(), macro_q: Default::default() }
    }

    pub fn update(&mut self) {
        let dpi = dpi_scale().max(0.01);
        self.began = touches()
            .iter()
            .filter(|t| matches!(t.phase, TouchPhase::Started | TouchPhase::Ended))
            .map(|t| t.position / dpi)
            .collect();
        (self.bits, self.nav) = match poll() {
            Some((bits, nav)) => {
                self.enabled = true;
                (bits, nav)
            }
            None => (0, 0),
        };
    }
}

pub fn label(bit: u32) -> &'static str {
    match bit {
        k::X => "X",
        k::Y => "Y",
        k::Z => "Z",
        k::A => "A",
        k::B => "B",
        k::C => "C",
        k::D2 => crate::i18n::tr3("ESP", "SP", "ESP"),
        k::W => "SUPER",
        k::T => crate::i18n::tr3("DESP.", "AWAKE", "DESP."),
        _ => "II",
    }
}
