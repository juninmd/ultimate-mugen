//! Input bits and history buffer.
pub mod k {
    pub const U: u32 = 1; pub const D: u32 = 2; pub const L: u32 = 4; pub const R: u32 = 8;
    pub const A: u32 = 16; pub const B: u32 = 32; pub const C: u32 = 64; pub const X: u32 = 128; pub const Y: u32 = 256; pub const Z: u32 = 512;
    pub const S: u32 = 1024; pub const D2: u32 = 2048; pub const W: u32 = 4096; pub const E: u32 = 8192;
    /// transformation (awakening) button
    pub const T: u32 = 16384;
    pub const TAG: u32 = 32768;
}

/// Button symbol (as used in CMD files) to bit.
pub fn button_bit(s: &str) -> Option<u32> {
    Some(match s { "a" => k::A, "b" => k::B, "c" => k::C, "x" => k::X, "y" => k::Y, "z" => k::Z, "s" => k::S, "d" => k::D2, "w" => k::W, "e" => k::E, _ => return None })
}

pub const HIST: usize = 256;

#[derive(Clone)]
pub struct InputHistory { buf: [u32; HIST], face: [i8; HIST], pub tick: i64 }
impl Default for InputHistory { fn default() -> Self { Self { buf: [0; HIST], face: [1; HIST], tick: -1 } } }
impl InputHistory {
    pub fn push(&mut self, bits: u32, facing: i32) {
        self.tick += 1;
        let i = (self.tick as usize) & (HIST - 1);
        self.buf[i] = bits;
        self.face[i] = if facing < 0 { -1 } else { 1 };
    }
    pub fn at(&self, t: i64) -> u32 { if t < 0 || t > self.tick || self.tick - t >= (HIST as i64) - 1 { 0 } else { self.buf[(t as usize) & (HIST - 1)] } }
    /// Drop the last `n` recorded ticks (negative input shifting); the current tick never goes below -1.
    pub fn rewind(&mut self, n: i32) { self.tick = (self.tick - n.abs() as i64).max(-1); }
    pub fn face_at(&self, t: i64) -> i32 { if t < 0 { 1 } else { self.face[(t as usize) & (HIST - 1)] as i32 } }
}

/// Relative direction bits: U=1 D=2 B=4 F=8
pub fn rel_dirs(bits: u32, facing: i32) -> u32 {
    let mut r = bits & 3;
    let (l, rr) = (bits & k::L != 0, bits & k::R != 0);
    if l && rr { return r; }
    if facing >= 0 { if l { r |= 4 } if rr { r |= 8 } } else { if rr { r |= 4 } if l { r |= 8 } }
    r
}

/// Source of input bits for a player (keyboard/gamepad/touch/AI/replay).
pub trait InputSource { fn poll(&mut self, m: &crate::matchs::Match, me: usize) -> u32; }
