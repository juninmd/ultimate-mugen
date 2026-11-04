use crate::chardef::Controller;
use crate::matchs::Match;

#[derive(Clone, Debug, PartialEq)]
pub struct PalFx {
    pub time: i32, pub add: [f32; 3], pub mul: [f32; 3], pub sin_add: [f32; 3], pub sin_period: i32,
    pub invert_all: bool, pub color: f32, pub elapsed: i32, pub hue: f32, pub sin_hue: f32,
}

impl Default for PalFx {
    fn default() -> Self { PalFx { time: 0, add: [0.0; 3], mul: [256.0; 3], sin_add: [0.0; 3], sin_period: 0, invert_all: false, color: 256.0, elapsed: 0, hue: 0.0, sin_hue: 0.0 } }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalParams { pub add: [f32; 3], pub mul: [f32; 3], pub color: f32, pub invert: bool, pub hue: f32 }

impl Default for PalParams {
    fn default() -> Self { PalParams { add: [0.0; 3], mul: [256.0; 3], color: 256.0, invert: false, hue: 0.0 } }
}

impl PalFx {
    pub fn new(time: i32, add: [f32; 3], mul: [f32; 3]) -> Self { PalFx { time, add, mul, ..Default::default() } }

    pub fn expired(&self) -> bool { self.time > 0 && self.elapsed >= self.time }

    pub fn params(&self) -> PalParams {
        let wave = if self.sin_period > 0 { (std::f32::consts::TAU * self.elapsed as f32 / self.sin_period as f32).sin() } else { 0.0 };
        let add = [0, 1, 2].map(|k| self.add[k] + self.sin_add[k] * wave);
        PalParams { add, mul: self.mul, color: self.color, invert: self.invert_all, hue: self.hue + self.sin_hue * wave }
    }

    pub fn from_controller(m: &Match, i: usize, sc: &Controller, prefix: &str) -> PalFx {
        let k = |name: &str| if prefix.is_empty() { name.to_string() } else { format!("{prefix}.{name}") };
        let n = |name: &str, d: f64, idx: usize| m.num(i, sc, &k(name), d, idx) as f32;
        PalFx {
            time: n("time", 0.0, 0) as i32,
            add: [n("add", 0.0, 0), n("add", 0.0, 1), n("add", 0.0, 2)],
            mul: [n("mul", 256.0, 0), n("mul", 256.0, 1), n("mul", 256.0, 2)],
            sin_add: [n("sinadd", 0.0, 0), n("sinadd", 0.0, 1), n("sinadd", 0.0, 2)],
            sin_period: n("sinadd", 0.0, 3) as i32,
            invert_all: n("invertall", 0.0, 0) != 0.0,
            color: n("color", 256.0, 0).clamp(0.0, 256.0),
            elapsed: 0,
            hue: n("hue", 0.0, 0),
            sin_hue: n("sinhue", 0.0, 0),
        }
    }
}

impl PalParams {
    pub fn combine(self, o: PalParams) -> PalParams {
        PalParams {
            add: [0, 1, 2].map(|k| self.add[k] + o.add[k]),
            mul: [0, 1, 2].map(|k| self.mul[k] * o.mul[k] / 256.0),
            color: self.color * o.color / 256.0,
            invert: self.invert != o.invert,
            hue: self.hue + o.hue,
        }
    }

    pub fn is_identity(&self) -> bool { *self == PalParams::default() }

    pub fn apply(&self, rgb: [u8; 3]) -> [u8; 3] {
        let mut c = rgb.map(|v| v as f32);
        if self.invert { c = c.map(|v| 255.0 - v); }
        if self.color < 256.0 {
            let gray = (c[0] + c[1] + c[2]) / 3.0;
            let t = self.color / 256.0;
            c = c.map(|v| gray + (v - gray) * t);
        }
        if self.hue.rem_euclid(360.0).abs() > 0.0001 {
            let (sin_a, cos_a) = self.hue.to_radians().sin_cos();
            let k = 0.577_350_26;
            let dot = (c[0] + c[1] + c[2]) * k;
            let cross = [(c[2] - c[1]) * k, (c[0] - c[2]) * k, (c[1] - c[0]) * k];
            c = [0, 1, 2].map(|i| (c[i] * cos_a + cross[i] * sin_a + k * dot * (1.0 - cos_a)).clamp(0.0, 255.0));
        }
        [0, 1, 2].map(|k| ((c[k] + self.add[k]) * self.mul[k] / 256.0).clamp(0.0, 255.0) as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode { None, Add, Add1, Sub, AddAlpha }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trans { pub mode: BlendMode, pub src: f32, pub dst: f32 }

impl Trans {
    pub fn parse(mode: &str, alpha: Option<(f32, f32)>) -> Trans {
        let m = mode.trim().to_ascii_lowercase();
        let mode = match m.as_str() { "add" => BlendMode::Add, "add1" => BlendMode::Add1, "sub" => BlendMode::Sub, "addalpha" => BlendMode::AddAlpha, _ => BlendMode::None };
        let (src, dst) = match (mode, alpha) {
            (BlendMode::AddAlpha, Some(a)) => a,
            (BlendMode::AddAlpha, None) => (256.0, 0.0),
            (BlendMode::Add1, _) => (256.0, 128.0),
            (BlendMode::Add, Some(a)) => a,
            _ => (256.0, 256.0),
        };
        Trans { mode, src: src.clamp(0.0, 256.0), dst: dst.clamp(0.0, 256.0) }
    }
}

#[derive(Clone, Debug)]
pub struct AfterImageParams {
    pub time_gap: i32, pub frame_gap: i32, pub trans: Trans,
    pub pal_color: f32, pub pal_invert: bool, pub pal_bright: [f32; 3], pub pal_contrast: [f32; 3],
    pub pal_post_bright: [f32; 3], pub pal_add: [f32; 3], pub pal_mul: [f32; 3],
}

impl Default for AfterImageParams {
    fn default() -> Self {
        AfterImageParams { time_gap: 1, frame_gap: 4, trans: Trans { mode: BlendMode::Add, src: 256.0, dst: 256.0 }, pal_color: 256.0, pal_invert: false,
            pal_bright: [30.0; 3], pal_contrast: [120.0, 120.0, 220.0], pal_post_bright: [0.0; 3], pal_add: [10.0, 10.0, 25.0], pal_mul: [0.65, 0.65, 0.75] }
    }
}

impl AfterImageParams {
    pub fn image_params(&self, image: usize) -> PalParams {
        let mut mul = [0, 1, 2].map(|k| self.pal_contrast[k].max(1.0));
        let mut add = [0, 1, 2].map(|k| self.pal_bright[k] + self.pal_post_bright[k] * 256.0 / mul[k]);
        for _ in 0..image {
            for k in 0..3 { add[k] += self.pal_add[k] * 256.0 / mul[k].max(1.0); mul[k] *= self.pal_mul[k]; }
        }
        PalParams { add, mul, color: self.pal_color, invert: self.pal_invert, hue: 0.0 }
    }

    pub fn shown(&self, trail_len: usize, length: usize) -> Vec<usize> {
        let gap = self.frame_gap.max(1) as usize;
        (1..trail_len.min(length)).filter(|k| k % gap == 0).collect()
    }
}
