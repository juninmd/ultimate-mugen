//! Projectile and super-move effects: a small tiny-skia canvas emulation that draws the `effect` frames of a fighter.
//! Fighters themselves are never drawn here; they come from pixel-art PNG sprites (see `pixel.rs`).
#[path = "effects/dragon.rs"] mod dragon;
use super::types::Box4;
type B4 = [f64; 4];
use crate::chardef::ImageData;
use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tiny_skia::{
    BlendMode, Color, FillRule, GradientStop, LineCap, LineJoin, Mask, Paint, Path,
    PathBuilder, Pixmap, Point, RadialGradient, Rect, Shader, SpreadMode, Stroke, Transform,
};

pub struct FrameArt {
    pub image: Option<Arc<ImageData>>,
    pub width: f32,
    pub height: f32,
    pub axis_x: f32,
    pub axis_y: f32,
    pub scale: f32,
    pub hurt: Vec<Box4>,
    pub hit: Vec<Box4>,
}

/// Art quality: sprites are rasterized at `res()` pixels per game unit (3 = default, 6 for 4K displays).
static ART_RES_X100: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(300);
pub fn set_art_res(r: f32) {
    ART_RES_X100.store(
        (r.clamp(1.0, 8.0) * 100.0) as u32,
        std::sync::atomic::Ordering::Relaxed,
    );
}
pub fn art_res() -> f32 {
    ART_RES_X100.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.0
}
fn res() -> f64 {
    art_res() as f64
}
const PI: f64 = std::f64::consts::PI;

// ------------------------------------------------------------------ math
fn hash(n: f64) -> f64 {
    let s = (n * 127.1 + 311.7).sin() * 43758.5453;
    s - s.floor()
}
/// JavaScript `Math.round` (halves round towards +inf)
fn jround(x: f64) -> f64 {
    (x + 0.5).floor()
}

// ------------------------------------------------------------------ JSON helpers (JS semantics)
fn gv<'a>(v: &'a Value, k: &str) -> Option<&'a Value> {
    v.get(k).filter(|x| !x.is_null())
}
fn num(v: Option<&Value>, d: f64) -> f64 {
    v.and_then(|x| x.as_f64())
        .filter(|x| x.is_finite())
        .unwrap_or(d)
}
fn gnum(v: &Value, k: &str, d: f64) -> f64 {
    num(gv(v, k), d)
}
fn gstr<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    gv(v, k).and_then(|x| x.as_str())
}
fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}
fn gt(v: &Value, k: &str) -> bool {
    truthy(gv(v, k))
}

// ------------------------------------------------------------------ colour
type Rgb = [u8; 3];
fn parse_rgb(c: &str) -> Rgb {
    let c = if c.is_empty() { "#888" } else { c };
    let mut h: Vec<char> = c.trim().chars().collect();
    if h.first() == Some(&'#') {
        h.remove(0);
    }
    if h.len() == 3 {
        h = h.iter().flat_map(|&ch| [ch, ch]).collect();
    }
    let mut v: u32 = 0;
    let mut any = false;
    for ch in h.iter().take(6) {
        match ch.to_digit(16) {
            Some(d) => {
                v = v * 16 + d;
                any = true;
            }
            None => break,
        }
    }
    if !any {
        return [136, 136, 136];
    }
    [
        ((v >> 16) & 255) as u8,
        ((v >> 8) & 255) as u8,
        (v & 255) as u8,
    ]
}
fn ch(x: f64) -> u8 {
    jround(x).clamp(0.0, 255.0) as u8
}
fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let f = |i: usize| ch(a[i] as f64 + (b[i] as f64 - a[i] as f64) * t);
    [f(0), f(1), f(2)]
}
fn light_of(c: Rgb) -> Rgb {
    mix(c, [0xff, 0xfa, 0xf0], 0.38)
}
fn rgba(c: Rgb, a: f64) -> Color {
    Color::from_rgba(
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        a.clamp(0.0, 1.0) as f32,
    )
    .unwrap_or(Color::BLACK)
}
const WHITE: Rgb = [255, 255, 255];

const DEFAULT_COLORS: &[(&str, &str)] = &[
    ("skin", "#f2c49b"),
    ("hair", "#2a2230"),
    ("eye", "#3a6fd8"),
    ("top", "#e8e4dc"),
    ("top2", "#b02a2a"),
    ("trim", "#1f1f2a"),
    ("pants", "#e8e4dc"),
    ("pants2", "#2a2a3a"),
    ("belt", "#1c1c22"),
    ("glove", "#b02a2a"),
    ("boot", "#3a2a22"),
    ("weapon", "#c9d3e0"),
    ("weapon2", "#3a3a48"),
    ("hilt", "#5a1f1f"),
    ("aura", "#7fd4ff"),
    ("energy", "#7fd4ff"),
    ("accent", "#ffcc33"),
    ("outline", "#160e1e"),
    ("horn", "#efe2c4"),
    ("metal", "#9aa6b8"),
    ("visor", "#59f0ff"),
    ("cloth", "#b02a2a"),
    ("mask", "#2a2a33"),
    ("hat", "#2a2030"),
    ("lips", "#c0606a"),
];

struct Colors(HashMap<String, String>);
impl Colors {
    fn new(extra: Option<&Value>, palette: Option<&Value>) -> Colors {
        let mut m: HashMap<String, String> = DEFAULT_COLORS
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let mut put = |v: Option<&Value>| {
            if let Some(Value::Object(o)) = v {
                for (k, x) in o {
                    if let Some(s) = x.as_str() {
                        m.insert(k.clone(), s.to_string());
                    }
                }
            }
        };
        put(extra);
        if let Some(p) = palette {
            put(Some(gv(p, "colors").unwrap_or(p)));
        }
        Colors(m)
    }
    fn get(&self, k: &str) -> Option<Rgb> {
        self.0.get(k).map(|s| parse_rgb(s))
    }
}

// ------------------------------------------------------------------ canvas emulation
#[derive(Clone, Copy, Debug)]
struct Aff {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}
impl Aff {
    const I: Aff = Aff {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    fn apply(&self, x: f64, y: f64) -> (f32, f32) {
        (
            (self.a * x + self.c * y + self.e) as f32,
            (self.b * x + self.d * y + self.f) as f32,
        )
    }
    fn ts(&self) -> Transform {
        Transform::from_row(
            self.a as f32,
            self.b as f32,
            self.c as f32,
            self.d as f32,
            self.e as f32,
            self.f as f32,
        )
    }
    fn scale_factor(&self) -> f64 {
        (self.a * self.d - self.b * self.c).abs().sqrt()
    }
}

#[derive(Clone)]
enum Fill {
    Solid(Color),
    Rad(f64, f64, f64, f64, Vec<(f32, Color)>),
}
fn sol(c: Rgb, a: f64) -> Fill {
    Fill::Solid(rgba(c, a))
}
fn stops(v: &[(f32, Color)]) -> Vec<GradientStop> {
    v.iter().map(|(o, c)| GradientStop::new(*o, *c)).collect()
}

/// clip mask whose buffer is recycled (only the rectangle that was written is cleared again)
struct Clip {
    m: Option<Mask>,
    rect: (usize, usize, usize, usize),
}
impl std::ops::Deref for Clip {
    type Target = Mask;
    fn deref(&self) -> &Mask {
        self.m.as_ref().expect("clip mask")
    }
}
thread_local! { static MASKS: std::cell::RefCell<Vec<Mask>> = const { std::cell::RefCell::new(Vec::new()) }; }
impl Drop for Clip {
    fn drop(&mut self) {
        if let Some(mut m) = self.m.take() {
            let (x0, y0, w, h) = self.rect;
            let pw = m.width() as usize;
            let d = m.data_mut();
            for y in y0..y0 + h {
                d[y * pw + x0..y * pw + x0 + w].fill(0);
            }
            MASKS.with(|p| {
                let mut p = p.borrow_mut();
                if p.len() < 6 {
                    p.push(m);
                }
            });
        }
    }
}

#[derive(Clone)]
struct St {
    m: Aff,
    alpha: f32,
    blend: BlendMode,
    shadow: Option<(Color, f32)>,
    lw: f64,
    clip: Option<Rc<Clip>>,
    butt: bool,
}

struct Ctx {
    pm: Pixmap,
    st: St,
    stack: Vec<St>,
    pb: PathBuilder,
    open: bool,
}

fn arc_sweep(a0: f64, a1: f64) -> f64 {
    let tau = 2.0 * PI;
    if a1 - a0 >= tau {
        tau
    } else {
        let s = (a1 - a0) % tau;
        if s < 0.0 {
            s + tau
        } else {
            s
        }
    }
}

impl Ctx {
    fn new(w: u32, h: u32) -> Option<Ctx> {
        Some(Ctx {
            pm: Pixmap::new(w.max(1), h.max(1))?,
            st: St {
                m: Aff::I,
                alpha: 1.0,
                blend: BlendMode::SourceOver,
                shadow: None,
                lw: 1.0,
                clip: None,
                butt: false,
            },
            stack: vec![],
            pb: PathBuilder::new(),
            open: false,
        })
    }
    fn save(&mut self) {
        self.stack.push(self.st.clone());
    }
    fn restore(&mut self) {
        if let Some(s) = self.stack.pop() {
            self.st = s;
        }
    }
    fn translate(&mut self, x: f64, y: f64) {
        let m = &mut self.st.m;
        m.e += m.a * x + m.c * y;
        m.f += m.b * x + m.d * y;
    }
    fn scale(&mut self, sx: f64, sy: f64) {
        let m = &mut self.st.m;
        m.a *= sx;
        m.b *= sx;
        m.c *= sy;
        m.d *= sy;
    }
    fn rotate(&mut self, t: f64) {
        let (s, c) = t.sin_cos();
        let m = self.st.m;
        self.st.m = Aff {
            a: m.a * c + m.c * s,
            b: m.b * c + m.d * s,
            c: -m.a * s + m.c * c,
            d: -m.b * s + m.d * c,
            ..m
        };
    }
    fn set_shadow(&mut self, c: Color, blur: f64) {
        self.st.shadow = Some((c, blur as f32));
    }
    fn begin(&mut self) {
        self.pb = PathBuilder::new();
        self.open = false;
    }
    fn move_to(&mut self, x: f64, y: f64) {
        let (u, v) = self.st.m.apply(x, y);
        self.pb.move_to(u, v);
        self.open = true;
    }
    fn line_to(&mut self, x: f64, y: f64) {
        if !self.open {
            return self.move_to(x, y);
        }
        let (u, v) = self.st.m.apply(x, y);
        self.pb.line_to(u, v);
    }
    fn quad_to(&mut self, cx: f64, cy: f64, x: f64, y: f64) {
        if !self.open {
            self.move_to(cx, cy);
        }
        let (a, b) = self.st.m.apply(cx, cy);
        let (u, v) = self.st.m.apply(x, y);
        self.pb.quad_to(a, b, u, v);
    }
    fn close(&mut self) {
        if self.open {
            self.pb.close();
        }
    }
    fn arc(&mut self, cx: f64, cy: f64, r: f64, a0: f64, a1: f64) {
        self.ellipse(cx, cy, r, r, 0.0, a0, a1)
    }
    #[allow(clippy::too_many_arguments)]
    fn ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, rot: f64, a0: f64, a1: f64) {
        let (sr, cr) = rot.sin_cos();
        let map = |ux: f64, uy: f64| {
            (
                cx + rx * ux * cr - ry * uy * sr,
                cy + rx * ux * sr + ry * uy * cr,
            )
        };
        let st = map(a0.cos(), a0.sin());
        if self.open {
            self.line_to(st.0, st.1)
        } else {
            self.move_to(st.0, st.1)
        }
        let sweep = arc_sweep(a0, a1);
        if sweep <= 0.0 {
            return;
        }
        let n = ((sweep / (PI / 2.0)) - 1e-9).ceil().max(1.0) as usize;
        let dt = sweep / n as f64;
        let k = 4.0 / 3.0 * (dt / 4.0).tan();
        for i in 0..n {
            let t1 = a0 + i as f64 * dt;
            let t2 = t1 + dt;
            let (s1, c1) = t1.sin_cos();
            let (s2, c2) = t2.sin_cos();
            let p1 = map(c1 - k * s1, s1 + k * c1);
            let p2 = map(c2 + k * s2, s2 - k * c2);
            let p3 = map(c2, s2);
            let m = self.st.m;
            let (x1, y1) = m.apply(p1.0, p1.1);
            let (x2, y2) = m.apply(p2.0, p2.1);
            let (x3, y3) = m.apply(p3.0, p3.1);
            self.pb.cubic_to(x1, y1, x2, y2, x3, y3);
        }
    }
    fn path(&self) -> Option<Path> {
        self.pb.clone().finish()
    }
    fn paint(&self, f: &Fill, blend: BlendMode) -> Option<Paint<'static>> {
        let ts = self.st.m.ts();
        let mut shader = match f {
            Fill::Solid(c) => {
                let mut c = *c;
                c.apply_opacity(self.st.alpha);
                return Some(Paint {
                    shader: Shader::SolidColor(c),
                    blend_mode: blend,
                    anti_alias: true,
                    force_hq_pipeline: false,
                });
            }
            Fill::Rad(x, y, r0, r1, s) => {
                let mut st: Vec<(f32, Color)> = s
                    .iter()
                    .map(|(o, c)| (((r0 + *o as f64 * (r1 - r0)) / r1) as f32, *c))
                    .collect();
                if *r0 > 0.0 {
                    if let Some(first) = s.first() {
                        st.insert(0, (0.0, first.1));
                    }
                }
                let c = Point::from_xy(*x as f32, *y as f32);
                RadialGradient::new(c, c, *r1 as f32, stops(&st), SpreadMode::Pad, ts)?
            }
        };
        if self.st.alpha < 1.0 {
            shader.apply_opacity(self.st.alpha);
        }
        Some(Paint {
            shader,
            blend_mode: blend,
            anti_alias: true,
            force_hq_pipeline: false,
        })
    }
    fn stroke_style(&self) -> Stroke {
        Stroke {
            width: (self.st.lw * self.st.m.scale_factor()) as f32,
            line_cap: if self.st.butt {
                LineCap::Butt
            } else {
                LineCap::Round
            },
            line_join: if self.st.butt {
                LineJoin::Bevel
            } else {
                LineJoin::Round
            },
            ..Default::default()
        }
    }
    fn fill(&mut self, f: &Fill) {
        if let Some(p) = self.path() {
            self.draw(&p, f, false);
        }
    }
    fn stroke(&mut self, f: &Fill) {
        if let Some(p) = self.path() {
            self.draw(&p, f, true);
        }
    }
    fn stroke_w(&mut self, f: &Fill, w: f64) {
        self.st.lw = w;
        self.stroke(f);
    }
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, f: &Fill) {
        let saved = (std::mem::take(&mut self.pb), self.open);
        self.begin();
        self.move_to(x, y);
        self.line_to(x + w, y);
        self.line_to(x + w, y + h);
        self.line_to(x, y + h);
        self.close();
        self.fill(f);
        self.pb = saved.0;
        self.open = saved.1;
    }
    fn draw(&mut self, path: &Path, f: &Fill, stroke: bool) {
        let outline;
        let geom: &Path = if stroke {
            outline = match path.stroke(&self.stroke_style(), 1.0) {
                Some(p) => p,
                None => return,
            };
            &outline
        } else {
            path
        };
        if let Some((sc, blur)) = self.st.shadow {
            let alpha = match f {
                Fill::Solid(c) => c.alpha() * self.st.alpha,
                _ => self.st.alpha,
            };
            self.shadow_paths(&[geom], alpha, sc, blur, 1.0);
        }
        match f {
            Fill::Solid(c) => self.fill_solid(geom, *c),
            _ => {
                let Some(paint) = self.paint(f, self.st.blend) else {
                    return;
                };
                let clip = self.st.clip.clone();
                self.pm.fill_path(
                    geom,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    clip.as_deref().map(|c| &**c),
                );
            }
        }
    }
    /// non-zero fill of a device-space path with a solid colour (fast rasterizer + own compositing)
    fn fill_solid(&mut self, geom: &Path, c: Color) {
        let Some((x0, y0, w, h)) = region(geom.bounds(), 1.0, self.pm.width(), self.pm.height())
        else {
            return;
        };
        let mut acc = Acc::new(w, h);
        acc.path(geom, x0 as f32, y0 as f32, 1.0);
        acc.prefix();
        let clip = self.st.clip.clone();
        composite(
            &mut self.pm,
            &acc.a,
            acc.stride,
            x0,
            y0,
            w,
            h,
            [c.red() * 255.0, c.green() * 255.0, c.blue() * 255.0],
            c.alpha() * self.st.alpha,
            self.st.blend,
            clip.as_deref(),
            &acc.rows,
        );
    }
    /// canvas `shadowBlur` for filled geometry: coverage (union of `geoms`, at reduced resolution
    /// for wide blurs), gaussian-ish 3-pass box blur (sigma = blur / 2), tinted and composited
    /// with bilinear upsampling.
    fn shadow_paths(&mut self, geoms: &[&Path], alpha: f32, col: Color, blur: f32, gain: f32) {
        if col.alpha() <= 0.0 || blur <= 0.0 || alpha <= 0.0 || geoms.is_empty() {
            return;
        }
        let sigma = blur / 2.0;
        let m = (sigma * 2.6).ceil() + 1.0;
        let mut bd = geoms[0].bounds();
        for g in &geoms[1..] {
            let b = g.bounds();
            bd = Rect::from_ltrb(
                bd.left().min(b.left()),
                bd.top().min(b.top()),
                bd.right().max(b.right()),
                bd.bottom().max(b.bottom()),
            )
            .unwrap_or(bd);
        }
        let Some((x0, y0, w, h)) = region(bd, m, self.pm.width(), self.pm.height()) else {
            return;
        };
        let f = (sigma / 4.0).floor().clamp(1.0, 4.0) as usize;
        let (sw, sh) = (w.div_ceil(f), h.div_ceil(f));
        let k = 1.0 / f as f32;
        let mut a = coverage(geoms, x0, y0, sw, sh, k, true);
        box_blur(&mut a, sw, sh, sigma / f as f32);
        let (r, g, b, ca) = (
            col.red(),
            col.green(),
            col.blue(),
            col.alpha() * gain * alpha,
        );
        let blend = self.st.blend;
        let clip = self.st.clip.clone();
        let stride = self.pm.width() as usize;
        let data = self.pm.data_mut();
        // separable bilinear upsampling: rows first, then columns
        let map = |x: usize, n: usize| -> (usize, usize, f32) {
            let u = ((x as f32 + 0.5) * k - 0.5).max(0.0);
            let i = (u as usize).min(n - 1);
            (i, (i + 1).min(n - 1), u - i as f32)
        };
        let cols: Vec<(usize, usize, f32)> = (0..w).map(|x| map(x, sw)).collect();
        let mut hz = vec![0f32; sh * w];
        for j in 0..sh {
            let row = &a[j * sw..(j + 1) * sw];
            for (x, &(i0, i1, fu)) in cols.iter().enumerate() {
                hz[j * w + x] = row[i0] + (row[i1] - row[i0]) * fu;
            }
        }
        for y in 0..h {
            let (j0, j1, fv) = map(y, sh);
            let (r0, r1) = (&hz[j0 * w..(j0 + 1) * w], &hz[j1 * w..(j1 + 1) * w]);
            let drow = (y0 + y) * stride + x0;
            for x in 0..w {
                let mut al = ((r0[x] + (r1[x] - r0[x]) * fv) * ca).min(1.0);
                if al < 1.0 / 512.0 {
                    continue;
                }
                let di = drow + x;
                if let Some(cm) = &clip {
                    al *= cm.data()[di] as f32 * (1.0 / 255.0);
                }
                blend_px(
                    &mut data[di * 4..di * 4 + 4],
                    [r * al * 255.0, g * al * 255.0, b * al * 255.0, al * 255.0],
                    blend,
                );
            }
        }
    }
}

/// fast round-to-nearest of a value in [0, 255] (float magic-number trick, no saturating cast)
#[inline(always)]
fn q8(x: f32) -> u8 {
    ((x.min(255.0) + 12582912.0).to_bits() & 0xff) as u8
}

/// composite one premultiplied source pixel (0..255 floats) onto a premultiplied RGBA8 pixel
#[inline]
fn blend_px(d: &mut [u8], s: [f32; 4], mode: BlendMode) {
    match mode {
        BlendMode::Plus => {
            for i in 0..4 {
                d[i] = q8(d[i] as f32 + s[i]);
            }
        }
        BlendMode::SourceAtop => {
            let (da, ia) = (d[3] as f32 * (1.0 / 255.0), 1.0 - s[3] * (1.0 / 255.0));
            for i in 0..3 {
                d[i] = q8(s[i] * da + d[i] as f32 * ia);
            }
        }
        _ => {
            let ia = 1.0 - s[3] * (1.0 / 255.0);
            for i in 0..4 {
                d[i] = q8(s[i] + d[i] as f32 * ia);
            }
        }
    }
}

/// composite a coverage buffer (region x0,y0,w,h of the pixmap) with a solid colour
#[allow(clippy::too_many_arguments)]
fn composite(
    pm: &mut Pixmap,
    cov: &[f32],
    cstride: usize,
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    rgb: [f32; 3],
    alpha: f32,
    mode: BlendMode,
    clip: Option<&Clip>,
    rows: &[(u32, u32)],
) {
    if alpha <= 0.0 {
        return;
    }
    let stride = pm.width() as usize;
    let data = pm.data_mut();
    let [r, g, b] = rgb;
    for y in 0..h {
        let (lo, hi) = rows[y];
        let (xs, xe) = (lo as usize, (hi as usize + 1).min(w));
        if lo > hi || xs >= xe {
            continue;
        }
        let crow = &cov[y * cstride + xs..y * cstride + xe];
        let drow = (y0 + y) * stride + x0 + xs;
        let dst = &mut data[drow * 4..(drow + xe - xs) * 4];
        let mrow = clip.map(|m| &m.data()[drow..drow + xe - xs]);
        match (mode, mrow) {
            (BlendMode::SourceOver, None) => {
                for (d, &c) in dst.as_chunks_mut::<4>().0.iter_mut().zip(crow) {
                    if c <= 0.0 {
                        continue;
                    }
                    let a = c * alpha;
                    let ia = 1.0 - a;
                    d[0] = q8(r * a + d[0] as f32 * ia);
                    d[1] = q8(g * a + d[1] as f32 * ia);
                    d[2] = q8(b * a + d[2] as f32 * ia);
                    d[3] = q8(255.0 * a + d[3] as f32 * ia);
                }
            }
            _ => {
                for (x, (d, &c)) in dst.as_chunks_mut::<4>().0.iter_mut().zip(crow).enumerate() {
                    if c <= 0.0 {
                        continue;
                    }
                    let mut a = c * alpha;
                    if let Some(m) = mrow {
                        a *= m[x] as f32 * (1.0 / 255.0);
                    }
                    if a < 1.0 / 512.0 {
                        continue;
                    }
                    blend_px(d, [r * a, g * a, b * a, a * 255.0], mode);
                }
            }
        }
    }
}

/// integer pixel region covering `b` expanded by `m`, clamped to the pixmap
fn region(b: Rect, m: f32, pw: u32, ph: u32) -> Option<(usize, usize, usize, usize)> {
    let x0 = (b.left() - m).floor().max(0.0);
    let y0 = (b.top() - m).floor().max(0.0);
    let x1 = (b.right() + m).ceil().min(pw as f32);
    let y1 = (b.bottom() + m).ceil().min(ph as f32);
    if !(x1 > x0 && y1 > y0) {
        return None;
    }
    Some((
        x0 as usize,
        y0 as usize,
        (x1 - x0) as usize,
        (y1 - y0) as usize,
    ))
}

// ------------------------------------------------------------------ fast AA rasterizer
/// Signed-area accumulation rasterizer (font-rs style): exact-area anti-aliasing, non-zero rule.
/// Rows are padded by 2 cells so geometry clamped to the right edge never leaks into the next row.
/// `rows` keeps the touched cell range of every row so that prefix-summing / compositing / clearing only visit
/// cells a shape actually reached (sparse shapes in big boxes: thin strokes, hatching, seams). Pool buffers are all-zero.
struct Acc {
    w: usize,
    h: usize,
    stride: usize,
    a: Vec<f32>,
    rows: Vec<(u32, u32)>,
}
thread_local! { static POOL: std::cell::RefCell<Vec<Vec<f32>>> = const { std::cell::RefCell::new(Vec::new()) }; }
impl Drop for Acc {
    fn drop(&mut self) {
        let mut a = std::mem::take(&mut self.a);
        if a.capacity() > 0 {
            for (y, &(lo, hi)) in self.rows.iter().enumerate() {
                if lo <= hi {
                    let s = y * self.stride;
                    let e = (s + hi as usize + 1).min(a.len());
                    a[s + lo as usize..e].fill(0.0);
                }
            }
            POOL.with(|p| {
                let mut p = p.borrow_mut();
                if p.len() < 8 {
                    p.push(a);
                }
            });
        }
    }
}
impl Acc {
    fn new(w: usize, h: usize) -> Acc {
        let n = (w + 2) * h;
        let mut a = POOL.with(|p| p.borrow_mut().pop()).unwrap_or_default();
        if a.len() < n {
            a.resize(n, 0.0);
        }
        Acc {
            w,
            h,
            stride: w + 2,
            a,
            rows: vec![(u32::MAX, 0); h],
        }
    }
    fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        if y0 == y1 || !(x0.is_finite() && y0.is_finite() && x1.is_finite() && y1.is_finite()) {
            return;
        }
        let (dir, ax, ay, bx, by) = if y0 < y1 {
            (1.0, x0, y0, x1, y1)
        } else {
            (-1.0, x1, y1, x0, y0)
        };
        let hf = self.h as f32;
        if by <= 0.0 || ay >= hf {
            return;
        }
        let wf = self.w as f32;
        let dxdy = (bx - ax) / (by - ay);
        let ys = ay.max(0.0);
        let ye = by.min(hf);
        let mut x = ax + (ys - ay) * dxdy;
        let mut yi = ys.floor() as usize;
        while (yi as f32) < ye && yi < self.h {
            let top = (yi as f32).max(ys);
            let bot = ((yi + 1) as f32).min(ye);
            let dy = bot - top;
            let xn = x + dxdy * dy;
            if dy > 0.0 {
                self.span(yi, x.clamp(0.0, wf), xn.clamp(0.0, wf), dy * dir);
            }
            x = xn;
            yi += 1;
        }
    }
    #[inline]
    fn span(&mut self, y: usize, xa: f32, xb: f32, d: f32) {
        let ls = y * self.stride;
        let (x0, x1) = if xa < xb { (xa, xb) } else { (xb, xa) };
        // x0, x1 >= 0 (clamped): truncation is floor; avoids libm calls on targets without SSE4.1
        let x0i = x0 as usize;
        let x0f = x0i as f32;
        let x1t = x1 as usize;
        let x1i = if (x1t as f32) < x1 { x1t + 1 } else { x1t };
        let r = &mut self.rows[y];
        r.0 = r.0.min(x0i as u32);
        r.1 = r.1.max(x1i.max(x0i + 1) as u32);
        let a = &mut self.a;
        let x1c = x1i as f32;
        if x1i <= x0i + 1 {
            let xmf = 0.5 * (xa + xb) - x0f;
            a[ls + x0i] += d - d * xmf;
            a[ls + x0i + 1] += d * xmf;
        } else {
            let s = 1.0 / (x1 - x0);
            let fr0 = x0 - x0f;
            let a0 = 0.5 * s * (1.0 - fr0) * (1.0 - fr0);
            let fr1 = x1 - x1c + 1.0;
            let am = 0.5 * s * fr1 * fr1;
            a[ls + x0i] += d * a0;
            if x1i == x0i + 2 {
                a[ls + x0i + 1] += d * (1.0 - a0 - am);
            } else {
                let a1 = s * (1.5 - fr0);
                a[ls + x0i + 1] += d * (a1 - a0);
                for xi in x0i + 2..x1i - 1 {
                    a[ls + xi] += d * s;
                }
                let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                a[ls + x1i - 1] += d * (1.0 - a2 - am);
            }
            a[ls + x1i] += d * am;
        }
    }
    fn path(&mut self, path: &Path, ox: f32, oy: f32, k: f32) {
        use tiny_skia::PathSegment as Seg;
        let tf = |p: Point| ((p.x - ox) * k, (p.y - oy) * k);
        let (mut start, mut last, mut open) = ((0.0, 0.0), (0.0, 0.0), false);
        const TOL: f32 = 0.15;
        for seg in path.segments() {
            match seg {
                Seg::MoveTo(p) => {
                    if open {
                        self.line(last.0, last.1, start.0, start.1);
                    }
                    start = tf(p);
                    last = start;
                    open = true;
                }
                Seg::LineTo(p) => {
                    let q = tf(p);
                    self.line(last.0, last.1, q.0, q.1);
                    last = q;
                }
                Seg::QuadTo(c, p) => {
                    let (c, q) = (tf(c), tf(p));
                    let dd = ((last.0 - 2.0 * c.0 + q.0).powi(2)
                        + (last.1 - 2.0 * c.1 + q.1).powi(2))
                    .sqrt();
                    let n = ((dd / (8.0 * TOL)).sqrt().ceil() as usize).clamp(1, 64);
                    let p0 = last;
                    for i in 1..=n {
                        let t = i as f32 / n as f32;
                        let u = 1.0 - t;
                        let nx = (
                            u * u * p0.0 + 2.0 * u * t * c.0 + t * t * q.0,
                            u * u * p0.1 + 2.0 * u * t * c.1 + t * t * q.1,
                        );
                        self.line(last.0, last.1, nx.0, nx.1);
                        last = nx;
                    }
                }
                Seg::CubicTo(c1, c2, p) => {
                    let (c1, c2, q) = (tf(c1), tf(c2), tf(p));
                    let p0 = last;
                    let d1 = ((p0.0 - 2.0 * c1.0 + c2.0).powi(2)
                        + (p0.1 - 2.0 * c1.1 + c2.1).powi(2))
                    .sqrt();
                    let d2 = ((c1.0 - 2.0 * c2.0 + q.0).powi(2)
                        + (c1.1 - 2.0 * c2.1 + q.1).powi(2))
                    .sqrt();
                    let n = ((0.75 * d1.max(d2) / TOL).sqrt().ceil() as usize).clamp(1, 64);
                    for i in 1..=n {
                        let t = i as f32 / n as f32;
                        let u = 1.0 - t;
                        let (b0, b1, b2, b3) =
                            (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                        let nx = (
                            b0 * p0.0 + b1 * c1.0 + b2 * c2.0 + b3 * q.0,
                            b0 * p0.1 + b1 * c1.1 + b2 * c2.1 + b3 * q.1,
                        );
                        self.line(last.0, last.1, nx.0, nx.1);
                        last = nx;
                    }
                }
                Seg::Close => {
                    self.line(last.0, last.1, start.0, start.1);
                    last = start;
                }
            }
        }
        if open {
            self.line(last.0, last.1, start.0, start.1);
        }
    }
    /// prefix-sum the accumulation rows into coverage in place (row stride kept)
    fn prefix(&mut self) {
        let (w, st) = (self.w, self.stride);
        for y in 0..self.h {
            let (lo, hi) = self.rows[y];
            let end = (hi as usize + 1).min(w);
            if lo > hi || lo as usize >= end {
                continue;
            }
            let mut acc = 0.0f32;
            for v in self.a[y * st + lo as usize..y * st + end].iter_mut() {
                acc += *v;
                *v = acc.abs().min(1.0);
            }
        }
    }
    /// prefix-sum the accumulation rows into coverage, compacting to a dense w*h buffer in place
    fn into_coverage(mut self) -> Vec<f32> {
        let (w, st) = (self.w, self.stride);
        for y in 0..self.h {
            let mut acc = 0.0f32;
            for x in 0..w {
                acc += self.a[y * st + x];
                self.a[y * w + x] = acc.abs().min(1.0);
            }
        }
        let mut a = std::mem::take(&mut self.a);
        a.truncate(w * self.h);
        a
    }
}

/// coverage (0..1, w*h) of device-space paths over the region at (x0, y0), scaled by `k`
/// (k < 1: low resolution grid). Several paths are combined as a union ("source-over").
fn coverage(
    paths: &[&Path],
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    k: f32,
    union: bool,
) -> Vec<f32> {
    let mut out: Option<Vec<f32>> = None;
    for p in paths {
        let mut acc = Acc::new(w, h);
        acc.path(p, x0 as f32, y0 as f32, k);
        let c = acc.into_coverage();
        out = Some(match out {
            Some(mut o) if union => {
                for (d, v) in o.iter_mut().zip(&c) {
                    *d = *d + v - *d * v;
                }
                o
            }
            _ => c,
        });
    }
    out.unwrap_or_else(|| vec![0.0; w * h])
}

/// box sizes approximating a gaussian with `n` passes (Kovesi)
fn box_blur(a: &mut [f32], w: usize, h: usize, sigma: f32) {
    if sigma < 0.25 || w == 0 || h == 0 {
        return;
    }
    let n = 3.0f32;
    let wideal = (12.0 * sigma * sigma / n + 1.0).sqrt();
    let mut wl = wideal.floor() as i32;
    if wl % 2 == 0 {
        wl -= 1;
    }
    let wu = wl + 2;
    let mideal = (12.0 * sigma * sigma - n * (wl * wl) as f32 - 4.0 * n * wl as f32 - 3.0 * n)
        / (-4.0 * wl as f32 - 4.0);
    let m = mideal.round() as i32;
    let mut tmp = vec![0.0f32; a.len()];
    for i in 0..3 {
        let size = if i < m { wl } else { wu };
        let r = ((size - 1) / 2).max(0) as usize;
        if r == 0 {
            continue;
        }
        let inv = 1.0 / (2 * r + 1) as f32;
        // horizontal a -> tmp
        for y in 0..h {
            let row = &a[y * w..(y + 1) * w];
            let out = &mut tmp[y * w..(y + 1) * w];
            let mut acc = 0.0f32;
            for v in row.iter().take(r.min(w - 1) + 1) {
                acc += v;
            }
            for x in 0..w {
                out[x] = acc * inv;
                if x + r + 1 < w {
                    acc += row[x + r + 1];
                }
                if x >= r {
                    acc -= row[x - r];
                }
            }
        }
        // vertical tmp -> a
        for x in 0..w {
            let mut acc = 0.0f32;
            for y in 0..=r.min(h - 1) {
                acc += tmp[y * w + x];
            }
            for y in 0..h {
                a[y * w + x] = acc * inv;
                if y + r + 1 < h {
                    acc += tmp[(y + r + 1) * w + x];
                }
                if y >= r {
                    acc -= tmp[(y - r) * w + x];
                }
            }
        }
    }
}

/// straight-alpha RGBA image of a premultiplied pixmap, demultiplied in place
fn to_image(p: Pixmap) -> Arc<ImageData> {
    let (w, h) = (p.width(), p.height());
    let mut data = p.take();
    for d in data.as_chunks_mut::<4>().0 {
        let a = d[3] as u32;
        if a != 0 && a != 255 {
            for v in d.iter_mut().take(3) {
                *v = ((*v as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    ImageData::new(w, h, data)
}

fn to_box4(b: &B4) -> Box4 {
    [b[0] as f32, b[1] as f32, b[2] as f32, b[3] as f32]
}

// ------------------------------------------------------------------ effects
/// effect: { type, size?, length?, color?, color2?, phase?, angle? }
pub fn render_effect(effect: &Value, with_images: bool, palette: Option<&Value>) -> FrameArt {
    let e = effect;
    let c = Colors::new(None, palette);
    let ty = gstr(e, "type").unwrap_or("orb");
    let size = gnum(e, "size", 14.0);
    let ph = gnum(e, "phase", 0.0);
    let col = match gstr(e, "color") {
        Some(s) => parse_rgb(s),
        None => c.get("energy").unwrap_or([0x7f, 0xd4, 0xff]),
    };
    let col2 = match gstr(e, "color2") {
        Some(s) => parse_rgb(s),
        None => light_of(col),
    };
    let s = size;
    let (body, draw): (B4, B4) = match ty {
        "beam" => {
            let l = gnum(e, "length", 200.0);
            ([0.0, -s, l, s], [-s * 1.5, -s * 2.2, l + s * 1.8, s * 2.2])
        }
        "pillar" => {
            let h = gnum(e, "length", 90.0);
            ([-s, -h, s, 0.0], [-s * 2.0, -h - s, s * 2.0, s * 0.6])
        }
        "shockwave" => (
            [-s, -s * 1.4, s * 1.1, 0.0],
            [-s * 2.4, -s * 2.2, s * 1.8, s * 0.5],
        ),
        "slash" | "water" | "wave" => (
            [-s * 0.35, -s, s * 0.55, s],
            [-s * 1.6, -s * 1.4, s * 1.2, s * 1.4],
        ),
        "kunai" | "shuriken" => (
            [-s * 0.6, -s * 0.45, s * 0.8, s * 0.45],
            [-s * 3.4, -s * 1.4, s * 1.6, s * 1.4],
        ),
        "ice" => (
            [-s * 0.8, -s * 0.6, s * 1.1, s * 0.6],
            [-s * 2.6, -s * 1.6, s * 2.0, s * 1.6],
        ),
        "burst" => (
            [-s * 0.6, -s * 0.6, s * 0.6, s * 0.6],
            [-s * 1.8, -s * 1.8, s * 1.8, s * 1.8],
        ),
        _ => (
            [-s * 0.8, -s * 0.8, s * 0.9, s * 0.8],
            [-s * 3.2, -s * 1.9, s * 1.9, s * 1.9],
        ),
    };
    let body = body.map(jround);
    let (x1, y1, x2, y2) = (
        (draw[0] - 4.0).floor(),
        (draw[1] - 4.0).floor(),
        (draw[2] + 4.0).ceil(),
        (draw[3] + 4.0).ceil(),
    );
    let (width, height, axis_x, axis_y) = (
        (x2 - x1) * res(),
        (y2 - y1) * res(),
        -x1 * res(),
        -y1 * res(),
    );
    let nohit = gt(e, "nohit");
    let boxes = if nohit { vec![] } else { vec![to_box4(&body)] };
    let mut image = None;
    if with_images {
        if let Some(mut ctx) = Ctx::new(width as u32, height as u32) {
            ctx.translate(axis_x, axis_y);
            ctx.scale(res(), res());
            draw_effect(&mut ctx, ty, size, ph, col, col2, e);
            image = Some(to_image(ctx.pm));
        }
    }
    FrameArt {
        image,
        width: width as f32,
        height: height as f32,
        axis_x: axis_x as f32,
        axis_y: axis_y as f32,
        scale: (1.0 / res()) as f32,
        hit: boxes.clone(),
        hurt: boxes,
    }
}

fn radial(ctx: &mut Ctx, x: f64, y: f64, r: f64, st: &[(f32, Color)]) {
    ctx.begin();
    ctx.arc(x, y, r, 0.0, PI * 2.0);
    ctx.fill(&Fill::Rad(x, y, 0.0, r, st.to_vec()));
}
fn glow_stroke(ctx: &mut Ctx, col: Color, w: f64, blur: f64, f: impl FnOnce(&mut Ctx)) {
    ctx.save();
    ctx.set_shadow(col, blur * res());
    ctx.st.lw = w;
    ctx.begin();
    f(ctx);
    ctx.stroke(&Fill::Solid(col));
    ctx.restore();
}
fn glow_fill(ctx: &mut Ctx, col: Color, blur: f64, f: impl FnOnce(&mut Ctx)) {
    ctx.save();
    ctx.set_shadow(col, blur * res());
    ctx.begin();
    f(ctx);
    ctx.fill(&Fill::Solid(col));
    ctx.restore();
}

#[allow(clippy::too_many_arguments)]
fn draw_effect(ctx: &mut Ctx, ty: &str, s: f64, ph: f64, col: Rgb, col2: Rgb, e: &Value) {
    let w = WHITE;
    ctx.st.blend = BlendMode::Plus;
    match ty {
        "slash" | "water" | "wave" => {
            let water = ty != "slash";
            let h = s;
            let crescent = |ctx: &mut Ctx, sc: f64| {
                ctx.move_to(0.0, -h * sc);
                ctx.quad_to(h * 0.95 * sc, 0.0, 0.0, h * sc);
                ctx.quad_to(h * 0.35 * sc, 0.0, 0.0, -h * sc);
                ctx.close();
            };
            // after images
            for i in (1..=3).rev() {
                let fi = i as f64;
                ctx.save();
                ctx.translate(-fi * s * 0.28, 0.0);
                ctx.st.alpha = (0.18 * (4.0 - fi)) as f32;
                glow_fill(ctx, rgba(col, 1.0), 4.0, |c| crescent(c, 1.0 - fi * 0.07));
                ctx.restore();
            }
            glow_fill(ctx, rgba(col, 0.95), 10.0, |c| crescent(c, 1.05));
            ctx.save();
            ctx.translate(h * 0.06, 0.0);
            glow_fill(ctx, rgba(col2, 1.0), 3.0, |c| crescent(c, 0.85));
            ctx.restore();
            ctx.save();
            ctx.translate(h * 0.12, 0.0);
            ctx.begin();
            crescent(ctx, 0.62);
            ctx.fill(&sol(w, 1.0));
            ctx.restore();
            if water {
                for i in 0..9 {
                    let fi = i as f64;
                    let y = (hash(fi + 1.0) * 2.0 - 1.0) * h * 0.9;
                    let x = -s * (0.2 + hash(fi + 5.0 + ph) * 1.1);
                    radial(
                        ctx,
                        x,
                        y,
                        1.2 + hash(fi * 3.0) * 2.2,
                        &[(0.0, rgba(w, 0.95)), (1.0, rgba(col, 0.0))],
                    );
                }
                glow_stroke(ctx, rgba(w, 0.8), 0.8, 3.0, |c| {
                    c.move_to(-s * 0.6, -h * 0.6);
                    c.quad_to(-s * 0.2 + ph.sin() * 3.0, 0.0, -s * 0.6, h * 0.6);
                });
            } else {
                for i in 0..6 {
                    let fi = i as f64;
                    let y = (fi / 5.0 - 0.5) * h * 1.6;
                    glow_stroke(ctx, rgba(col2, 0.6), 0.7, 2.0, |c| {
                        c.move_to(-s * (0.3 + hash(fi + ph) * 0.4), y);
                        c.line_to(-s * (1.0 + hash(fi * 7.0 + ph) * 0.6), y * 0.9);
                    });
                }
            }
        }
        "beam" => {
            let l = gnum(e, "length", 200.0);
            let wob = |x: f64, k: f64| (x * 0.12 + ph * 3.0 + k).sin() * s * 0.12;
            let band = |ctx: &mut Ctx, th: f64, k: f64| {
                ctx.move_to(0.0, -th);
                let mut x = 0.0;
                while x <= l {
                    ctx.line_to(x, -th - wob(x, k));
                    x += 8.0;
                }
                ctx.arc(l, 0.0, th, -PI / 2.0, PI / 2.0);
                let mut x = l;
                while x >= 0.0 {
                    ctx.line_to(x, th + wob(x, k + 1.0));
                    x -= 8.0;
                }
                ctx.close();
            };
            glow_fill(ctx, rgba(col, 0.55), 14.0, |c| band(c, s * 1.25, 0.0));
            glow_fill(ctx, rgba(col, 0.9), 6.0, |c| band(c, s * 0.85, 2.0));
            ctx.begin();
            band(ctx, s * 0.55, 4.0);
            ctx.fill(&sol(col2, 1.0));
            ctx.begin();
            band(ctx, s * 0.28, 6.0);
            ctx.fill(&sol(w, 1.0));
            // origin flare and head
            radial(
                ctx,
                0.0,
                0.0,
                s * 2.1,
                &[
                    (0.0, rgba(w, 1.0)),
                    (0.35, rgba(col2, 0.8)),
                    (1.0, rgba(col, 0.0)),
                ],
            );
            radial(
                ctx,
                l,
                0.0,
                s * 1.8,
                &[
                    (0.0, rgba(w, 1.0)),
                    (0.4, rgba(col2, 0.7)),
                    (1.0, rgba(col, 0.0)),
                ],
            );
            // energy streaks
            for i in 0..10 {
                let fi = i as f64;
                let x = (hash(fi + 2.0) * l + ph * 40.0) % l;
                let y = (hash(fi + 9.0) * 2.0 - 1.0) * s * 0.9;
                glow_stroke(ctx, rgba(w, 0.7), 0.7, 2.0, |c| {
                    c.move_to(x, y);
                    c.line_to(x + 14.0 + hash(fi) * 20.0, y);
                });
            }
        }
        "ice" => {
            let shard = |ctx: &mut Ctx, x: f64, y: f64, l: f64, wd: f64, a: f64| {
                ctx.save();
                ctx.translate(x, y);
                ctx.rotate(a);
                glow_fill(ctx, rgba(col, 0.75), 6.0, |c| {
                    c.move_to(l, 0.0);
                    c.line_to(0.0, -wd);
                    c.line_to(-l * 0.5, 0.0);
                    c.line_to(0.0, wd);
                    c.close();
                });
                ctx.begin();
                ctx.move_to(l, 0.0);
                ctx.line_to(0.0, -wd);
                ctx.line_to(-l * 0.1, 0.0);
                ctx.close();
                ctx.fill(&sol(w, 0.85));
                ctx.begin();
                ctx.move_to(l, 0.0);
                ctx.line_to(0.0, -wd);
                ctx.line_to(-l * 0.5, 0.0);
                ctx.line_to(0.0, wd);
                ctx.close();
                ctx.stroke_w(&sol(w, 0.9), 0.5);
                ctx.restore();
            };
            for i in 0..5 {
                let t = i as f64 / 5.0;
                radial(
                    ctx,
                    -s * (0.8 + t * 1.6),
                    (i as f64 * 2.0 + ph).sin() * s * 0.3,
                    s * (0.5 - t * 0.3),
                    &[(0.0, rgba(col, 0.5)), (1.0, rgba(col, 0.0))],
                );
            }
            radial(
                ctx,
                0.0,
                0.0,
                s * 1.5,
                &[(0.0, rgba(col, 0.6)), (1.0, rgba(col, 0.0))],
            );
            shard(
                ctx,
                -s * 0.3,
                -s * 0.45,
                s * 0.9,
                s * 0.25,
                -0.25 + ph.sin() * 0.05,
            );
            shard(ctx, -s * 0.3, s * 0.45, s * 0.9, s * 0.25, 0.25);
            shard(ctx, 0.0, 0.0, s * 1.15, s * 0.4, 0.0);
            for i in 0..6 {
                let fi = i as f64;
                radial(
                    ctx,
                    -s * (0.6 + hash(fi + ph * 3.0) * 1.6),
                    (hash(fi * 5.0) * 2.0 - 1.0) * s * 0.8,
                    1.4,
                    &[(0.0, rgba(w, 1.0)), (1.0, rgba(col, 0.0))],
                );
            }
        }
        "lightning" => {
            radial(
                ctx,
                0.0,
                0.0,
                s * 1.8,
                &[
                    (0.0, rgba(col, 0.7)),
                    (0.5, rgba(col, 0.25)),
                    (1.0, rgba(col, 0.0)),
                ],
            );
            radial(
                ctx,
                0.0,
                0.0,
                s * 0.8,
                &[
                    (0.0, rgba(w, 1.0)),
                    (0.6, rgba(col2, 0.8)),
                    (1.0, rgba(col, 0.0)),
                ],
            );
            for k in 0..6 {
                let fk = k as f64;
                let a0 = (fk / 6.0) * PI * 2.0 + ph * 1.7;
                glow_stroke(ctx, rgba(col2, 1.0), 0.9, 5.0, |c| {
                    let (mut x, mut y) = (0.0, 0.0);
                    c.move_to(0.0, 0.0);
                    for i in 1..=5 {
                        let a = a0 + (hash(fk * 10.0 + i as f64 + (ph * 7.0).floor()) - 0.5) * 1.3;
                        x += a.cos() * s * 0.33;
                        y += a.sin() * s * 0.33;
                        c.line_to(x, y);
                    }
                });
            }
            for i in 0..4 {
                let fi = i as f64;
                glow_stroke(ctx, rgba(w, 0.9), 0.6, 2.0, |c| {
                    let (mut x, mut y) = (-s * 0.3, (fi - 1.5) * s * 0.3);
                    c.move_to(x, y);
                    for j in 0..5 {
                        x -= s * 0.45;
                        y += (hash(fi * 9.0 + j as f64 + ph * 5.0) - 0.5) * s * 0.5;
                        c.line_to(x, y);
                    }
                });
            }
        }
        "kunai" | "shuriken" => {
            // motion streaks
            for i in 0..3 {
                let fi = i as f64;
                glow_stroke(ctx, rgba(col, 0.6 - fi * 0.15), 1.2 - fi * 0.3, 3.0, |c| {
                    c.move_to(-s * 0.4, (fi - 1.0) * s * 0.25);
                    c.line_to(-s * (2.6 - fi * 0.4), (fi - 1.0) * s * 0.3);
                });
            }
            ctx.st.blend = BlendMode::SourceOver;
            let dark = [0x14, 0x10, 0x18];
            if ty == "kunai" {
                ctx.save();
                ctx.set_shadow(rgba(col, 1.0), 5.0 * res());
                ctx.st.lw = 0.6;
                ctx.begin();
                ctx.move_to(s * 0.9, 0.0);
                ctx.line_to(0.0, -s * 0.28);
                ctx.line_to(-s * 0.1, 0.0);
                ctx.line_to(0.0, s * 0.28);
                ctx.close();
                ctx.fill(&sol([0xdf, 0xe6, 0xf0], 1.0));
                ctx.stroke(&sol(dark, 1.0));
                ctx.restore();
                ctx.begin();
                ctx.move_to(s * 0.9, 0.0);
                ctx.line_to(0.0, s * 0.28);
                ctx.line_to(-s * 0.1, 0.0);
                ctx.close();
                ctx.fill(&sol([0x7a, 0x84, 0x96], 1.0));
                ctx.fill_rect(
                    -s * 0.55,
                    -s * 0.08,
                    s * 0.45,
                    s * 0.16,
                    &sol([0x2a, 0x22, 0x30], 1.0),
                );
                ctx.begin();
                ctx.arc(-s * 0.7, 0.0, s * 0.14, 0.0, PI * 2.0);
                ctx.stroke_w(&sol([0x2a, 0x22, 0x30], 1.0), 0.8);
            } else {
                ctx.save();
                ctx.rotate(ph * 4.0);
                ctx.set_shadow(rgba(col, 1.0), 5.0 * res());
                ctx.st.lw = 0.6;
                ctx.begin();
                for i in 0..8 {
                    let a = (i as f64 / 8.0) * PI * 2.0;
                    let rr = if i % 2 == 1 { s * 0.18 } else { s * 0.6 };
                    ctx.line_to(a.cos() * rr, a.sin() * rr);
                }
                ctx.close();
                ctx.fill(&sol([0xc9, 0xd2, 0xde], 1.0));
                ctx.stroke(&sol(dark, 1.0));
                ctx.restore();
                ctx.begin();
                ctx.arc(0.0, 0.0, s * 0.08, 0.0, PI * 2.0);
                ctx.fill(&sol([0x2a, 0x22, 0x30], 1.0));
            }
        }
        "pillar" => {
            let h = gnum(e, "length", 90.0);
            glow_fill(ctx, rgba(col, 0.55), 12.0, |c| {
                c.move_to(-s * 1.3, 0.0);
                let mut y = 0.0;
                while y >= -h {
                    c.line_to(
                        -s * (1.1 - (-y / h) * 0.4) + (y * 0.2 + ph * 4.0).sin() * 2.0,
                        y,
                    );
                    y -= 6.0;
                }
                c.line_to(0.0, -h - s * 0.8);
                let mut y = -h;
                while y <= 0.0 {
                    c.line_to(
                        s * (1.1 - (-y / h) * 0.4) + (y * 0.25 + ph * 4.0 + 1.0).sin() * 2.0,
                        y,
                    );
                    y += 6.0;
                }
                c.close();
            });
            ctx.begin();
            ctx.move_to(-s * 0.6, 0.0);
            ctx.quad_to(-s * 0.3, -h * 0.6, 0.0, -h);
            ctx.quad_to(s * 0.3, -h * 0.6, s * 0.6, 0.0);
            ctx.fill(&sol(col2, 0.95));
            ctx.begin();
            ctx.move_to(-s * 0.25, 0.0);
            ctx.quad_to(0.0, -h * 0.5, 0.0, -h * 0.8);
            ctx.quad_to(0.0, -h * 0.5, s * 0.25, 0.0);
            ctx.fill(&sol(w, 0.9));
            radial(
                ctx,
                0.0,
                0.0,
                s * 2.0,
                &[
                    (0.0, rgba(w, 0.9)),
                    (0.4, rgba(col, 0.6)),
                    (1.0, rgba(col, 0.0)),
                ],
            );
        }
        "shockwave" => {
            for i in 0..3 {
                let fi = i as f64;
                let x = -fi * s * 0.6;
                glow_fill(ctx, rgba(col, 0.6 - fi * 0.15), 8.0, |c| {
                    c.move_to(x - s, 0.0);
                    c.quad_to(
                        x - s * 0.2,
                        -s * (1.6 - fi * 0.3) - (ph * 3.0).sin() * 3.0,
                        x + s,
                        0.0,
                    );
                    c.close();
                });
            }
            ctx.st.blend = BlendMode::SourceOver;
            for i in 0..5 {
                let fi = i as f64;
                let x = (hash(fi + 4.0) - 0.6) * s * 2.0;
                let y = -hash(fi + 8.0 + ph) * s * 1.4;
                let rr = 1.5 + hash(fi) * 2.5;
                ctx.begin();
                ctx.move_to(x - rr, y);
                ctx.line_to(x, y - rr);
                ctx.line_to(x + rr, y + rr * 0.3);
                ctx.line_to(x - rr * 0.2, y + rr);
                ctx.close();
                ctx.fill(&sol([0x6a, 0x58, 0x48], 1.0));
                ctx.stroke_w(&sol([0x1a, 0x12, 0x10], 1.0), 0.5);
            }
        }
        "dragon" => dragon::draw(ctx, s, ph, col, col2),
        "burst" => {
            let t = ph.clamp(0.0, 1.0);
            let rr = s * (0.6 + t * 1.1);
            radial(
                ctx,
                0.0,
                0.0,
                rr,
                &[
                    (0.0, rgba(w, 1.0 - t * 0.7)),
                    (0.4, rgba(col2, 0.8 * (1.0 - t))),
                    (1.0, rgba(col, 0.0)),
                ],
            );
            glow_stroke(ctx, rgba(col2, 1.0 - t), 1.6 * (1.0 - t) + 0.3, 6.0, |c| {
                c.arc(0.0, 0.0, rr * 0.85, 0.0, PI * 2.0)
            });
            for i in 0..8 {
                let a = (i as f64 / 8.0) * PI * 2.0 + 0.3;
                glow_stroke(ctx, rgba(w, 1.0 - t), 0.9, 3.0, |c| {
                    c.move_to(a.cos() * rr * 0.5, a.sin() * rr * 0.5);
                    c.line_to(a.cos() * rr * 1.3, a.sin() * rr * 1.3);
                });
            }
        }
        "orb" | "fire" | "darkfire" => {
            let fire = ty != "orb";
            // trail
            for i in 0..6 {
                let t = i as f64 / 6.0;
                let x = -s * (0.6 + t * 2.2);
                let y = (ph * 2.0 + i as f64 * 1.3).sin() * s * 0.18 * t;
                radial(
                    ctx,
                    x,
                    y,
                    s * (0.85 - t * 0.55),
                    &[(0.0, rgba(col, 0.55 * (1.0 - t))), (1.0, rgba(col, 0.0))],
                );
            }
            if fire {
                for i in 0..9 {
                    let a = (i as f64 / 9.0) * PI * 2.0 + ph;
                    let fl = s * (1.0 + 0.5 * hash(i as f64 + ph * 10.0));
                    ctx.save();
                    ctx.translate(a.cos() * s * 0.3, a.sin() * s * 0.3);
                    glow_fill(ctx, rgba(col, 0.75), 6.0, |c| {
                        c.move_to(0.0, -s * 0.45);
                        c.quad_to(
                            -fl * 1.3,
                            -s * 0.5 + a.sin() * 4.0,
                            -fl * 1.9,
                            (a * 2.0).sin() * s * 0.4,
                        );
                        c.quad_to(-fl * 1.1, s * 0.5, 0.0, s * 0.45);
                        c.close();
                    });
                    ctx.restore();
                }
            }
            radial(
                ctx,
                0.0,
                0.0,
                s * 1.9,
                &[
                    (0.0, rgba(col, 0.8)),
                    (0.45, rgba(col, 0.35)),
                    (1.0, rgba(col, 0.0)),
                ],
            );
            radial(
                ctx,
                0.0,
                0.0,
                s * 0.95,
                &[
                    (0.0, rgba(w, 1.0)),
                    (0.5, rgba(col2, 0.95)),
                    (1.0, rgba(col, 0.2)),
                ],
            );
            // swirl
            for i in 0..3 {
                let a = ph * 2.0 + i as f64 * 2.1;
                let rr = s * (0.75 + i as f64 * 0.22);
                glow_stroke(ctx, rgba(col2, 0.9), 1.1, 4.0, |c| {
                    c.arc(0.0, 0.0, rr, a, a + 2.2)
                });
            }
        }
        _ => {
            // lotus / petal burst: trailing petals, a ring of petals and a hot core
            for i in 0..7 {
                let fi = i as f64;
                let x = -s * (0.8 + hash(fi + 1.0) * 1.8);
                let y = (hash(fi + 7.0) * 2.0 - 1.0) * s * 0.8;
                let a = ph * 3.0 + fi;
                ctx.save();
                ctx.translate(x, y);
                ctx.rotate(a);
                glow_fill(ctx, rgba(col, 0.7 - fi * 0.07), 3.0, |c| {
                    c.move_to(-s * 0.2, 0.0);
                    c.quad_to(0.0, -s * 0.16, s * 0.2, 0.0);
                    c.quad_to(0.0, s * 0.16, -s * 0.2, 0.0);
                });
                ctx.restore();
            }
            radial(
                ctx,
                0.0,
                0.0,
                s * 1.5,
                &[(0.0, rgba(col, 0.55)), (1.0, rgba(col, 0.0))],
            );
            for i in 0..6 {
                let a = (i as f64 / 6.0) * PI * 2.0 + ph * 1.5;
                ctx.save();
                ctx.rotate(a);
                glow_fill(ctx, rgba(col, 0.9), 5.0, |c| {
                    c.move_to(s * 0.15, 0.0);
                    c.quad_to(s * 0.55, -s * 0.38, s * 1.05, 0.0);
                    c.quad_to(s * 0.55, s * 0.38, s * 0.15, 0.0);
                });
                ctx.begin();
                ctx.move_to(s * 0.2, 0.0);
                ctx.quad_to(s * 0.5, -s * 0.16, s * 0.85, 0.0);
                ctx.quad_to(s * 0.5, s * 0.16, s * 0.2, 0.0);
                ctx.fill(&sol(col2, 0.9));
                ctx.restore();
            }
            radial(
                ctx,
                0.0,
                0.0,
                s * 0.55,
                &[
                    (0.0, rgba(w, 1.0)),
                    (0.6, rgba(col2, 0.9)),
                    (1.0, rgba(col, 0.0)),
                ],
            );
        }
    }
}
