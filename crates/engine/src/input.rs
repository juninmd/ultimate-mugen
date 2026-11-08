pub mod k {
    pub const U: u32 = 1; pub const D: u32 = 2; pub const L: u32 = 4; pub const R: u32 = 8;
    pub const A: u32 = 16; pub const B: u32 = 32; pub const C: u32 = 64; pub const X: u32 = 128; pub const Y: u32 = 256; pub const Z: u32 = 512;
    pub const S: u32 = 1024; pub const D2: u32 = 2048; pub const W: u32 = 4096; pub const E: u32 = 8192;
    pub const T: u32 = 16384;
    pub const TAG: u32 = 32768;
    pub const LANE_UP: u32 = 65536; pub const LANE_DOWN: u32 = 131072; pub const LANE_MASK: u32 = LANE_UP | LANE_DOWN;
    pub const GUARD: u32 = 262144; pub const BRAWLER_MASK: u32 = LANE_MASK | GUARD;
}

pub fn button_bit(s: &str) -> Option<u32> {
    let b = s.as_bytes();
    if b.len() == 1 {
        Some(match b[0] {
            b'a' | b'A' => k::A,
            b'b' | b'B' => k::B,
            b'c' | b'C' => k::C,
            b'x' | b'X' => k::X,
            b'y' | b'Y' => k::Y,
            b'z' | b'Z' => k::Z,
            b's' | b'S' => k::S,
            b'd' | b'D' => k::D2,
            b'w' | b'W' => k::W,
            b'e' | b'E' => k::E,
            _ => return None,
        })
    } else {
        None
    }
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
        self.face[i] = 1 - (((facing < 0) as i8) << 1);
    }
    pub fn at(&self, t: i64) -> u32 { if t < 0 || t > self.tick || self.tick - t >= (HIST as i64) - 1 { 0 } else { self.buf[(t as usize) & (HIST - 1)] } }
    pub fn rewind(&mut self, n: i32) { self.tick = (self.tick - n.abs() as i64).max(-1); }
    pub fn face_at(&self, t: i64) -> i32 { if t < 0 { 1 } else { self.face[(t as usize) & (HIST - 1)] as i32 } }
}

pub fn rel_dirs(bits: u32, facing: i32) -> u32 {
    let ud = bits & 3;
    let lr = bits & (k::L | k::R);
    if lr == 0 || lr == (k::L | k::R) {
        return ud;
    }
    let is_l = (lr == k::L) as u32;
    let is_neg = (facing < 0) as u32;
    let is_back = is_l ^ is_neg;
    ud | (4 << (is_back ^ 1))
}

pub trait InputSource { fn poll(&mut self, m: &crate::matchs::Match, me: usize) -> u32; }
