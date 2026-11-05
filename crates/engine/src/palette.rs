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
        if self.sin_period > 0 {
            let wave = (std::f32::consts::TAU * self.elapsed as f32 / self.sin_period as f32).sin();
            let add = [
                self.add[0] + self.sin_add[0] * wave,
                self.add[1] + self.sin_add[1] * wave,
                self.add[2] + self.sin_add[2] * wave,
            ];
            PalParams { add, mul: self.mul, color: self.color, invert: self.invert_all, hue: self.hue + self.sin_hue * wave }
        } else {
            PalParams { add: self.add, mul: self.mul, color: self.color, invert: self.invert_all, hue: self.hue }
        }
    }

    pub fn from_controller(m: &Match, i: usize, sc: &Controller, prefix: &str) -> PalFx {
        let n = |name: &str, d: f64, idx: usize| -> f32 {
            if prefix.is_empty() {
                m.num(i, sc, name, d, idx) as f32
            } else {
                let mut buf = [0u8; 64];
                let key = if prefix.len() + 1 + name.len() <= buf.len() {
                    let p_bytes = prefix.as_bytes();
                    let n_bytes = name.as_bytes();
                    buf[..p_bytes.len()].copy_from_slice(p_bytes);
                    buf[p_bytes.len()] = b'.';
                    buf[p_bytes.len() + 1..p_bytes.len() + 1 + n_bytes.len()].copy_from_slice(n_bytes);
                    std::str::from_utf8(&buf[..p_bytes.len() + 1 + n_bytes.len()]).unwrap_or(name)
                } else {
                    return m.num(i, sc, &format!("{prefix}.{name}"), d, idx) as f32;
                };
                m.num(i, sc, key, d, idx) as f32
            }
        };
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
        if self.is_identity() { return o; }
        if o.is_identity() { return self; }
        PalParams {
            add: [self.add[0] + o.add[0], self.add[1] + o.add[1], self.add[2] + o.add[2]],
            mul: [self.mul[0] * o.mul[0] * (1.0 / 256.0), self.mul[1] * o.mul[1] * (1.0 / 256.0), self.mul[2] * o.mul[2] * (1.0 / 256.0)],
            color: self.color * o.color * (1.0 / 256.0),
            invert: self.invert != o.invert,
            hue: self.hue + o.hue,
        }
    }

    pub fn is_identity(&self) -> bool { *self == PalParams::default() }

    pub fn apply(&self, rgb: [u8; 3]) -> [u8; 3] {
        if self.is_identity() {
            return rgb;
        }
        if !self.invert && self.color >= 256.0 && self.hue.rem_euclid(360.0).abs() <= 0.0001 {
            return [
                ((rgb[0] as f32 + self.add[0]) * (self.mul[0] * (1.0 / 256.0))).clamp(0.0, 255.0) as u8,
                ((rgb[1] as f32 + self.add[1]) * (self.mul[1] * (1.0 / 256.0))).clamp(0.0, 255.0) as u8,
                ((rgb[2] as f32 + self.add[2]) * (self.mul[2] * (1.0 / 256.0))).clamp(0.0, 255.0) as u8,
            ];
        }
        let mut c = [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32];
        if self.invert { c = [255.0 - c[0], 255.0 - c[1], 255.0 - c[2]]; }
        if self.color < 256.0 {
            let gray = (c[0] + c[1] + c[2]) * (1.0 / 3.0);
            let t = self.color * (1.0 / 256.0);
            c = [
                gray + (c[0] - gray) * t,
                gray + (c[1] - gray) * t,
                gray + (c[2] - gray) * t,
            ];
        }
        if self.hue.rem_euclid(360.0).abs() > 0.0001 {
            let (sin_a, cos_a) = self.hue.to_radians().sin_cos();
            let k = 0.577_350_26;
            let dot = (c[0] + c[1] + c[2]) * k;
            let cross = [(c[2] - c[1]) * k, (c[0] - c[2]) * k, (c[1] - c[0]) * k];
            c = [
                (c[0] * cos_a + cross[0] * sin_a + k * dot * (1.0 - cos_a)).clamp(0.0, 255.0),
                (c[1] * cos_a + cross[1] * sin_a + k * dot * (1.0 - cos_a)).clamp(0.0, 255.0),
                (c[2] * cos_a + cross[2] * sin_a + k * dot * (1.0 - cos_a)).clamp(0.0, 255.0),
            ];
        }
        [
            ((c[0] + self.add[0]) * (self.mul[0] * (1.0 / 256.0))).clamp(0.0, 255.0) as u8,
            ((c[1] + self.add[1]) * (self.mul[1] * (1.0 / 256.0))).clamp(0.0, 255.0) as u8,
            ((c[2] + self.add[2]) * (self.mul[2] * (1.0 / 256.0))).clamp(0.0, 255.0) as u8,
        ]
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
