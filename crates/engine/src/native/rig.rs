//! Procedural character art: a 2D skeletal rig rendered with an anime / fighting-game look
//! (bold outlines, two-tone cel shading with a rim light, flowing hair & cloth), plus an
//! effect renderer for projectiles and super moves. Port of `src/native/rig.ts` to tiny-skia.
//!
//! Conventions (engine units): the feet / ground point is the axis (0,0), the fighter faces RIGHT,
//! y grows downwards (negative = up). Images are rendered at RES x and returned with scale 1/RES.
//! All geometry (hit/hurt boxes, size, axis) is computed without rasterizing so that headless code
//! (tests, servers) gets identical collision data.
//!
//! Canvas2D features missing from tiny-skia are emulated by a tiny canvas-like context (`Ctx`):
//! path building in device space (arcs/ellipses as cubics), save/restore, AA clip masks,
//! global alpha, 'lighter' (Plus) / 'source-atop' compositing and `shadowBlur` glows
//! (coverage rendered to a temporary pixmap, 3-pass box blur, tinted and composited).
use super::types::{ActiveSpec, Box4};
use crate::chardef::ImageData;
use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tiny_skia::{
    BlendMode, Color, FillRule, GradientStop, LineCap, LineJoin, LinearGradient, Mask, Paint, Path, PathBuilder, Pixmap, Point,
    RadialGradient, Rect, Shader, SpreadMode, Stroke, Transform,
};

pub struct RigRender { pub image: Option<Arc<ImageData>>, pub width: f32, pub height: f32, pub axis_x: f32, pub axis_y: f32, pub scale: f32, pub hurt: Vec<Box4>, pub hit: Vec<Box4> }

const RES: f64 = 3.0;
const PAD: f64 = 7.0;
const PI: f64 = std::f64::consts::PI;
const D2R: f64 = PI / 180.0;

// ------------------------------------------------------------------ math
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct P { x: f64, y: f64 }
const fn pt(x: f64, y: f64) -> P { P { x, y } }
/// direction for an angle in degrees: 0 = down, 90 = forward (+x), 180 = up
fn dir(a: f64) -> P { let r = a * D2R; pt(r.sin(), r.cos()) }
fn add(p: P, q: P, s: f64) -> P { pt(p.x + q.x * s, p.y + q.y * s) }
fn sub(p: P, q: P) -> P { pt(p.x - q.x, p.y - q.y) }
fn len(p: P) -> f64 { p.x.hypot(p.y) }
fn norm(p: P) -> P { let l = len(p); let l = if l == 0.0 || l.is_nan() { 1.0 } else { l }; pt(p.x / l, p.y / l) }
fn lerp(a: P, b: P, t: f64) -> P { pt(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t) }
/// perpendicular (rotated +90 deg on screen: (x,y) -> (-y,x))
fn perp(p: P) -> P { pt(-p.y, p.x) }
fn ang_of(p: P) -> f64 { p.x.atan2(p.y) / D2R }
fn hash(n: f64) -> f64 { let s = (n * 127.1 + 311.7).sin() * 43758.5453; s - s.floor() }
/// JavaScript `Math.round` (halves round towards +inf)
fn jround(x: f64) -> f64 { (x + 0.5).floor() }

// ------------------------------------------------------------------ JSON helpers (JS semantics)
fn gv<'a>(v: &'a Value, k: &str) -> Option<&'a Value> { v.get(k).filter(|x| !x.is_null()) }
fn num(v: Option<&Value>, d: f64) -> f64 { v.and_then(|x| x.as_f64()).filter(|x| x.is_finite()).unwrap_or(d) }
fn gnum(v: &Value, k: &str, d: f64) -> f64 { num(gv(v, k), d) }
fn gstr<'a>(v: &'a Value, k: &str) -> Option<&'a str> { gv(v, k).and_then(|x| x.as_str()) }
fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}
fn gt(v: &Value, k: &str) -> bool { truthy(gv(v, k)) }
fn is_num(v: Option<&Value>) -> bool { matches!(v, Some(Value::Number(_))) }

// ------------------------------------------------------------------ colour
type Rgb = [u8; 3];
fn parse_rgb(c: &str) -> Rgb {
    let c = if c.is_empty() { "#888" } else { c };
    let mut h: Vec<char> = c.trim().chars().collect();
    if h.first() == Some(&'#') { h.remove(0); }
    if h.len() == 3 { h = h.iter().flat_map(|&ch| [ch, ch]).collect(); }
    let mut v: u32 = 0;
    let mut any = false;
    for ch in h.iter().take(6) {
        match ch.to_digit(16) { Some(d) => { v = v * 16 + d; any = true; } None => break }
    }
    if !any { return [136, 136, 136]; }
    [((v >> 16) & 255) as u8, ((v >> 8) & 255) as u8, (v & 255) as u8]
}
fn ch(x: f64) -> u8 { jround(x).clamp(0.0, 255.0) as u8 }
fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let f = |i: usize| ch(a[i] as f64 + (b[i] as f64 - a[i] as f64) * t);
    [f(0), f(1), f(2)]
}
fn lum(c: Rgb) -> f64 { (0.3 * c[0] as f64 + 0.59 * c[1] as f64 + 0.11 * c[2] as f64) / 255.0 }
fn shade_of(c: Rgb) -> Rgb { mix(c, [0x2b, 0x17, 0x48], 0.42) }
fn light_of(c: Rgb) -> Rgb { mix(c, [0xff, 0xfa, 0xf0], 0.38) }
fn back_of(c: Rgb) -> Rgb { mix(c, [0x12, 0x0a, 0x22], 0.3) }
fn rgba(c: Rgb, a: f64) -> Color { Color::from_rgba(c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, a.clamp(0.0, 1.0) as f32).unwrap_or(Color::BLACK) }
const WHITE: Rgb = [255, 255, 255];

const DEFAULT_COLORS: &[(&str, &str)] = &[
    ("skin", "#f2c49b"), ("hair", "#2a2230"), ("eye", "#3a6fd8"), ("top", "#e8e4dc"), ("top2", "#b02a2a"), ("trim", "#1f1f2a"),
    ("pants", "#e8e4dc"), ("pants2", "#2a2a3a"), ("belt", "#1c1c22"), ("glove", "#b02a2a"), ("boot", "#3a2a22"), ("weapon", "#c9d3e0"),
    ("weapon2", "#3a3a48"), ("hilt", "#5a1f1f"), ("aura", "#7fd4ff"), ("energy", "#7fd4ff"), ("accent", "#ffcc33"), ("outline", "#160e1e"),
    ("horn", "#efe2c4"), ("metal", "#9aa6b8"), ("visor", "#59f0ff"), ("cloth", "#b02a2a"), ("mask", "#2a2a33"), ("hat", "#2a2030"), ("lips", "#c0606a"),
];

struct Colors(HashMap<String, String>);
impl Colors {
    fn new(extra: Option<&Value>, palette: Option<&Value>) -> Colors {
        let mut m: HashMap<String, String> = DEFAULT_COLORS.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        let mut put = |v: Option<&Value>| {
            if let Some(Value::Object(o)) = v {
                for (k, x) in o { if let Some(s) = x.as_str() { m.insert(k.clone(), s.to_string()); } }
            }
        };
        put(extra);
        if let Some(p) = palette { put(Some(gv(p, "colors").unwrap_or(p))); }
        Colors(m)
    }
    fn get(&self, k: &str) -> Option<Rgb> { self.0.get(k).map(|s| parse_rgb(s)) }
    fn c(&self, k: &str) -> Rgb { self.get(k).unwrap_or([136, 136, 136]) }
    fn has(&self, k: &str) -> bool { self.0.get(k).is_some_and(|s| !s.is_empty()) }
    /// `C[v ?? def] ?? v`
    fn cref(&self, v: Option<&Value>, def: &str) -> Rgb {
        let vs = v.and_then(|x| x.as_str());
        let key = vs.unwrap_or(def);
        self.get(key).unwrap_or_else(|| parse_rgb(vs.unwrap_or("")))
    }
}

// ------------------------------------------------------------------ canvas emulation
#[derive(Clone, Copy, Debug)]
struct Aff { a: f64, b: f64, c: f64, d: f64, e: f64, f: f64 }
impl Aff {
    const I: Aff = Aff { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };
    fn apply(&self, x: f64, y: f64) -> (f32, f32) { ((self.a * x + self.c * y + self.e) as f32, (self.b * x + self.d * y + self.f) as f32) }
    fn ts(&self) -> Transform { Transform::from_row(self.a as f32, self.b as f32, self.c as f32, self.d as f32, self.e as f32, self.f as f32) }
    fn scale_factor(&self) -> f64 { (self.a * self.d - self.b * self.c).abs().sqrt() }
}

#[derive(Clone)]
enum Fill { Solid(Color), Lin(f64, f64, f64, f64, Vec<(f32, Color)>), Rad(f64, f64, f64, f64, Vec<(f32, Color)>) }
fn sol(c: Rgb, a: f64) -> Fill { Fill::Solid(rgba(c, a)) }
fn stops(v: &[(f32, Color)]) -> Vec<GradientStop> { v.iter().map(|(o, c)| GradientStop::new(*o, *c)).collect() }

#[derive(Clone)]
struct St { m: Aff, alpha: f32, blend: BlendMode, shadow: Option<(Color, f32)>, lw: f64, clip: Option<Rc<Mask>> }

struct Ctx { pm: Pixmap, st: St, stack: Vec<St>, pb: PathBuilder, open: bool }

fn arc_sweep(a0: f64, a1: f64) -> f64 {
    let tau = 2.0 * PI;
    if a1 - a0 >= tau { tau } else { let s = (a1 - a0) % tau; if s < 0.0 { s + tau } else { s } }
}

impl Ctx {
    fn new(w: u32, h: u32) -> Option<Ctx> {
        Some(Ctx { pm: Pixmap::new(w.max(1), h.max(1))?, st: St { m: Aff::I, alpha: 1.0, blend: BlendMode::SourceOver, shadow: None, lw: 1.0, clip: None }, stack: vec![], pb: PathBuilder::new(), open: false })
    }
    fn save(&mut self) { self.stack.push(self.st.clone()); }
    fn restore(&mut self) { if let Some(s) = self.stack.pop() { self.st = s; } }
    fn translate(&mut self, x: f64, y: f64) { let m = &mut self.st.m; m.e += m.a * x + m.c * y; m.f += m.b * x + m.d * y; }
    fn scale(&mut self, sx: f64, sy: f64) { let m = &mut self.st.m; m.a *= sx; m.b *= sx; m.c *= sy; m.d *= sy; }
    fn rotate(&mut self, t: f64) {
        let (s, c) = t.sin_cos();
        let m = self.st.m;
        self.st.m = Aff { a: m.a * c + m.c * s, b: m.b * c + m.d * s, c: -m.a * s + m.c * c, d: -m.b * s + m.d * c, ..m };
    }
    fn set_shadow(&mut self, c: Color, blur: f64) { self.st.shadow = Some((c, blur as f32)); }
    fn begin(&mut self) { self.pb = PathBuilder::new(); self.open = false; }
    fn move_to(&mut self, x: f64, y: f64) { let (u, v) = self.st.m.apply(x, y); self.pb.move_to(u, v); self.open = true; }
    fn line_to(&mut self, x: f64, y: f64) { if !self.open { return self.move_to(x, y); } let (u, v) = self.st.m.apply(x, y); self.pb.line_to(u, v); }
    fn mv(&mut self, p: P) { self.move_to(p.x, p.y) }
    fn ln(&mut self, p: P) { self.line_to(p.x, p.y) }
    fn quad_to(&mut self, cx: f64, cy: f64, x: f64, y: f64) {
        if !self.open { self.move_to(cx, cy); }
        let (a, b) = self.st.m.apply(cx, cy);
        let (u, v) = self.st.m.apply(x, y);
        self.pb.quad_to(a, b, u, v);
    }
    fn close(&mut self) { if self.open { self.pb.close(); } }
    fn arc(&mut self, cx: f64, cy: f64, r: f64, a0: f64, a1: f64) { self.ellipse(cx, cy, r, r, 0.0, a0, a1) }
    #[allow(clippy::too_many_arguments)]
    fn ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, rot: f64, a0: f64, a1: f64) {
        let (sr, cr) = rot.sin_cos();
        let map = |ux: f64, uy: f64| (cx + rx * ux * cr - ry * uy * sr, cy + rx * ux * sr + ry * uy * cr);
        let st = map(a0.cos(), a0.sin());
        if self.open { self.line_to(st.0, st.1) } else { self.move_to(st.0, st.1) }
        let sweep = arc_sweep(a0, a1);
        if sweep <= 0.0 { return; }
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
    fn path(&self) -> Option<Path> { self.pb.clone().finish() }
    fn paint(&self, f: &Fill, blend: BlendMode) -> Option<Paint<'static>> {
        let ts = self.st.m.ts();
        let mut shader = match f {
            Fill::Solid(c) => { let mut c = *c; c.apply_opacity(self.st.alpha); return Some(Paint { shader: Shader::SolidColor(c), blend_mode: blend, anti_alias: true, force_hq_pipeline: false }) }
            Fill::Lin(x0, y0, x1, y1, s) => LinearGradient::new(Point::from_xy(*x0 as f32, *y0 as f32), Point::from_xy(*x1 as f32, *y1 as f32), stops(s), SpreadMode::Pad, ts)?,
            Fill::Rad(x, y, r0, r1, s) => {
                let mut st: Vec<(f32, Color)> = s.iter().map(|(o, c)| (((r0 + *o as f64 * (r1 - r0)) / r1) as f32, *c)).collect();
                if *r0 > 0.0 { if let Some(first) = s.first() { st.insert(0, (0.0, first.1)); } }
                let c = Point::from_xy(*x as f32, *y as f32);
                RadialGradient::new(c, c, *r1 as f32, stops(&st), SpreadMode::Pad, ts)?
            }
        };
        if self.st.alpha < 1.0 { shader.apply_opacity(self.st.alpha); }
        Some(Paint { shader, blend_mode: blend, anti_alias: true, force_hq_pipeline: false })
    }
    fn stroke_style(&self) -> Stroke { Stroke { width: (self.st.lw * self.st.m.scale_factor()) as f32, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Default::default() } }
    fn fill(&mut self, f: &Fill) { if let Some(p) = self.path() { self.draw(&p, f, false); } }
    fn stroke(&mut self, f: &Fill) { if let Some(p) = self.path() { self.draw(&p, f, true); } }
    fn stroke_w(&mut self, f: &Fill, w: f64) { self.st.lw = w; self.stroke(f); }
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, f: &Fill) {
        let saved = (std::mem::take(&mut self.pb), self.open);
        self.begin();
        self.move_to(x, y); self.line_to(x + w, y); self.line_to(x + w, y + h); self.line_to(x, y + h); self.close();
        self.fill(f);
        self.pb = saved.0; self.open = saved.1;
    }
    fn draw(&mut self, path: &Path, f: &Fill, stroke: bool) {
        let outline;
        let geom: &Path = if stroke { outline = match path.stroke(&self.stroke_style(), 1.0) { Some(p) => p, None => return }; &outline } else { path };
        if let Some((sc, blur)) = self.st.shadow {
            let alpha = match f { Fill::Solid(c) => c.alpha() * self.st.alpha, _ => self.st.alpha };
            self.shadow_paths(&[geom], alpha, sc, blur, 1.0);
        }
        match f {
            Fill::Solid(c) => self.fill_solid(geom, *c),
            _ => {
                let Some(paint) = self.paint(f, self.st.blend) else { return };
                let clip = self.st.clip.clone();
                self.pm.fill_path(geom, &paint, FillRule::Winding, Transform::identity(), clip.as_deref());
            }
        }
    }
    /// non-zero fill of a device-space path with a solid colour (fast rasterizer + own compositing)
    fn fill_solid(&mut self, geom: &Path, c: Color) {
        let Some((x0, y0, w, h)) = region(geom.bounds(), 1.0, self.pm.width(), self.pm.height()) else { return };
        let cov = coverage(&[geom], x0, y0, w, h, 1.0, false);
        let clip = self.st.clip.clone();
        composite(&mut self.pm, &cov, x0, y0, w, h, [c.red() * 255.0, c.green() * 255.0, c.blue() * 255.0], c.alpha() * self.st.alpha, self.st.blend, clip.as_deref());
    }
    /// canvas `shadowBlur` for filled geometry: coverage (union of `geoms`, at reduced resolution
    /// for wide blurs), gaussian-ish 3-pass box blur (sigma = blur / 2), tinted and composited
    /// with bilinear upsampling.
    fn shadow_paths(&mut self, geoms: &[&Path], alpha: f32, col: Color, blur: f32, gain: f32) {
        if col.alpha() <= 0.0 || blur <= 0.0 || alpha <= 0.0 || geoms.is_empty() { return; }
        let sigma = blur / 2.0;
        let m = (sigma * 2.6).ceil() + 1.0;
        let mut bd = geoms[0].bounds();
        for g in &geoms[1..] { let b = g.bounds(); bd = Rect::from_ltrb(bd.left().min(b.left()), bd.top().min(b.top()), bd.right().max(b.right()), bd.bottom().max(b.bottom())).unwrap_or(bd); }
        let Some((x0, y0, w, h)) = region(bd, m, self.pm.width(), self.pm.height()) else { return };
        let f = (sigma / 4.0).floor().clamp(1.0, 4.0) as usize;
        let (sw, sh) = (w.div_ceil(f), h.div_ceil(f));
        let k = 1.0 / f as f32;
        let mut a = coverage(geoms, x0, y0, sw, sh, k, true);
        box_blur(&mut a, sw, sh, sigma / f as f32);
        let (r, g, b, ca) = (col.red(), col.green(), col.blue(), col.alpha() * gain * alpha);
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
            for (x, &(i0, i1, fu)) in cols.iter().enumerate() { hz[j * w + x] = row[i0] + (row[i1] - row[i0]) * fu; }
        }
        for y in 0..h {
            let (j0, j1, fv) = map(y, sh);
            let (r0, r1) = (&hz[j0 * w..(j0 + 1) * w], &hz[j1 * w..(j1 + 1) * w]);
            let drow = (y0 + y) * stride + x0;
            for x in 0..w {
                let mut al = ((r0[x] + (r1[x] - r0[x]) * fv) * ca).min(1.0);
                if al < 1.0 / 512.0 { continue; }
                let di = drow + x;
                if let Some(cm) = &clip { al *= cm.data()[di] as f32 * (1.0 / 255.0); }
                blend_px(&mut data[di * 4..di * 4 + 4], [r * al * 255.0, g * al * 255.0, b * al * 255.0, al * 255.0], blend);
            }
        }
    }
    fn clip(&mut self) {
        let (pw, ph) = (self.pm.width(), self.pm.height());
        let Some(mut m) = Mask::new(pw, ph) else { return };
        let path = self.path();
        if let Some((x0, y0, w, h)) = path.as_ref().and_then(|p| region(p.bounds(), 1.0, pw, ph)) {
            let cov = coverage(&[path.as_ref().unwrap()], x0, y0, w, h, 1.0, false);
            let old = self.st.clip.clone();
            let stride = pw as usize;
            let md = m.data_mut();
            for y in 0..h {
                for x in 0..w {
                    let di = (y0 + y) * stride + x0 + x;
                    let c = cov[y * w + x] * old.as_ref().map_or(1.0, |o| o.data()[di] as f32 * (1.0 / 255.0));
                    md[di] = (c * 255.0 + 0.5) as u8;
                }
            }
        }
        self.st.clip = Some(Rc::new(m));
    }
}

/// composite one premultiplied source pixel (0..255 floats) onto a premultiplied RGBA8 pixel
#[inline]
fn blend_px(d: &mut [u8], s: [f32; 4], mode: BlendMode) {
    match mode {
        BlendMode::Plus => { for i in 0..4 { d[i] = (d[i] as f32 + s[i] + 0.5).min(255.0) as u8; } }
        BlendMode::SourceAtop => {
            let (da, ia) = (d[3] as f32 * (1.0 / 255.0), 1.0 - s[3] * (1.0 / 255.0));
            for i in 0..3 { d[i] = (s[i] * da + d[i] as f32 * ia + 0.5).min(255.0) as u8; }
        }
        _ => {
            let ia = 1.0 - s[3] * (1.0 / 255.0);
            for i in 0..4 { d[i] = (s[i] + d[i] as f32 * ia + 0.5).min(255.0) as u8; }
        }
    }
}

/// composite a coverage buffer (region x0,y0,w,h of the pixmap) with a solid colour
#[allow(clippy::too_many_arguments)]
fn composite(pm: &mut Pixmap, cov: &[f32], x0: usize, y0: usize, w: usize, h: usize, rgb: [f32; 3], alpha: f32, mode: BlendMode, clip: Option<&Mask>) {
    if alpha <= 0.0 { return; }
    let stride = pm.width() as usize;
    let data = pm.data_mut();
    let [r, g, b] = rgb;
    for y in 0..h {
        let crow = &cov[y * w..(y + 1) * w];
        let drow = (y0 + y) * stride + x0;
        let dst = &mut data[drow * 4..(drow + w) * 4];
        let mrow = clip.map(|m| &m.data()[drow..drow + w]);
        match (mode, mrow) {
            (BlendMode::SourceOver, None) => {
                for (d, &c) in dst.chunks_exact_mut(4).zip(crow) {
                    if c <= 0.0 { continue; }
                    let a = c * alpha;
                    let ia = 1.0 - a;
                    d[0] = (r * a + d[0] as f32 * ia + 0.5) as u8;
                    d[1] = (g * a + d[1] as f32 * ia + 0.5) as u8;
                    d[2] = (b * a + d[2] as f32 * ia + 0.5) as u8;
                    d[3] = (255.0 * a + d[3] as f32 * ia + 0.5) as u8;
                }
            }
            _ => {
                for (x, (d, &c)) in dst.chunks_exact_mut(4).zip(crow).enumerate() {
                    if c <= 0.0 { continue; }
                    let mut a = c * alpha;
                    if let Some(m) = mrow { a *= m[x] as f32 * (1.0 / 255.0); }
                    if a < 1.0 / 512.0 { continue; }
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
    if !(x1 > x0 && y1 > y0) { return None; }
    Some((x0 as usize, y0 as usize, (x1 - x0) as usize, (y1 - y0) as usize))
}

// ------------------------------------------------------------------ fast AA rasterizer
/// Signed-area accumulation rasterizer (font-rs style): exact-area anti-aliasing, non-zero rule.
/// Rows are padded by 2 cells so geometry clamped to the right edge never leaks into the next row.
struct Acc { w: usize, h: usize, stride: usize, a: Vec<f32> }
impl Acc {
    fn new(w: usize, h: usize) -> Acc { Acc { w, h, stride: w + 2, a: vec![0.0; (w + 2) * h] } }
    fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        if y0 == y1 || !(x0.is_finite() && y0.is_finite() && x1.is_finite() && y1.is_finite()) { return; }
        let (dir, ax, ay, bx, by) = if y0 < y1 { (1.0, x0, y0, x1, y1) } else { (-1.0, x1, y1, x0, y0) };
        let hf = self.h as f32;
        if by <= 0.0 || ay >= hf { return; }
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
            if dy > 0.0 { self.span(yi, x.clamp(0.0, wf), xn.clamp(0.0, wf), dy * dir); }
            x = xn;
            yi += 1;
        }
    }
    #[inline]
    fn span(&mut self, y: usize, xa: f32, xb: f32, d: f32) {
        let ls = y * self.stride;
        let a = &mut self.a;
        let (x0, x1) = if xa < xb { (xa, xb) } else { (xb, xa) };
        let x0f = x0.floor();
        let x0i = x0f as usize;
        let x1c = x1.ceil();
        let x1i = x1c as usize;
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
                for xi in x0i + 2..x1i - 1 { a[ls + xi] += d * s; }
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
                Seg::MoveTo(p) => { if open { self.line(last.0, last.1, start.0, start.1); } start = tf(p); last = start; open = true; }
                Seg::LineTo(p) => { let q = tf(p); self.line(last.0, last.1, q.0, q.1); last = q; }
                Seg::QuadTo(c, p) => {
                    let (c, q) = (tf(c), tf(p));
                    let dd = ((last.0 - 2.0 * c.0 + q.0).powi(2) + (last.1 - 2.0 * c.1 + q.1).powi(2)).sqrt();
                    let n = ((dd / (8.0 * TOL)).sqrt().ceil() as usize).clamp(1, 64);
                    let p0 = last;
                    for i in 1..=n {
                        let t = i as f32 / n as f32;
                        let u = 1.0 - t;
                        let nx = (u * u * p0.0 + 2.0 * u * t * c.0 + t * t * q.0, u * u * p0.1 + 2.0 * u * t * c.1 + t * t * q.1);
                        self.line(last.0, last.1, nx.0, nx.1);
                        last = nx;
                    }
                }
                Seg::CubicTo(c1, c2, p) => {
                    let (c1, c2, q) = (tf(c1), tf(c2), tf(p));
                    let p0 = last;
                    let d1 = ((p0.0 - 2.0 * c1.0 + c2.0).powi(2) + (p0.1 - 2.0 * c1.1 + c2.1).powi(2)).sqrt();
                    let d2 = ((c1.0 - 2.0 * c2.0 + q.0).powi(2) + (c1.1 - 2.0 * c2.1 + q.1).powi(2)).sqrt();
                    let n = ((0.75 * d1.max(d2) / TOL).sqrt().ceil() as usize).clamp(1, 64);
                    for i in 1..=n {
                        let t = i as f32 / n as f32;
                        let u = 1.0 - t;
                        let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                        let nx = (b0 * p0.0 + b1 * c1.0 + b2 * c2.0 + b3 * q.0, b0 * p0.1 + b1 * c1.1 + b2 * c2.1 + b3 * q.1);
                        self.line(last.0, last.1, nx.0, nx.1);
                        last = nx;
                    }
                }
                Seg::Close => { self.line(last.0, last.1, start.0, start.1); last = start; }
            }
        }
        if open { self.line(last.0, last.1, start.0, start.1); }
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
        self.a.truncate(w * self.h);
        self.a
    }
}

/// coverage (0..1, w*h) of device-space paths over the region at (x0, y0), scaled by `k`
/// (k < 1: low resolution grid). Several paths are combined as a union ("source-over").
fn coverage(paths: &[&Path], x0: usize, y0: usize, w: usize, h: usize, k: f32, union: bool) -> Vec<f32> {
    let mut out: Option<Vec<f32>> = None;
    for p in paths {
        let mut acc = Acc::new(w, h);
        acc.path(p, x0 as f32, y0 as f32, k);
        let c = acc.into_coverage();
        out = Some(match out {
            Some(mut o) if union => { for (d, v) in o.iter_mut().zip(&c) { *d = *d + v - *d * v; } o }
            _ => c,
        });
    }
    out.unwrap_or_else(|| vec![0.0; w * h])
}

/// box sizes approximating a gaussian with `n` passes (Kovesi)
fn box_blur(a: &mut [f32], w: usize, h: usize, sigma: f32) {
    if sigma < 0.25 || w == 0 || h == 0 { return; }
    let n = 3.0f32;
    let wideal = (12.0 * sigma * sigma / n + 1.0).sqrt();
    let mut wl = wideal.floor() as i32;
    if wl % 2 == 0 { wl -= 1; }
    let wu = wl + 2;
    let mideal = (12.0 * sigma * sigma - n * (wl * wl) as f32 - 4.0 * n * wl as f32 - 3.0 * n) / (-4.0 * wl as f32 - 4.0);
    let m = mideal.round() as i32;
    let mut tmp = vec![0.0f32; a.len()];
    for i in 0..3 {
        let size = if i < m { wl } else { wu };
        let r = ((size - 1) / 2).max(0) as usize;
        if r == 0 { continue; }
        let inv = 1.0 / (2 * r + 1) as f32;
        // horizontal a -> tmp
        for y in 0..h {
            let row = &a[y * w..(y + 1) * w];
            let out = &mut tmp[y * w..(y + 1) * w];
            let mut acc = 0.0f32;
            for v in row.iter().take(r.min(w - 1) + 1) { acc += v; }
            for x in 0..w {
                out[x] = acc * inv;
                if x + r + 1 < w { acc += row[x + r + 1]; }
                if x >= r { acc -= row[x - r]; }
            }
        }
        // vertical tmp -> a
        for x in 0..w {
            let mut acc = 0.0f32;
            for y in 0..=r.min(h - 1) { acc += tmp[y * w + x]; }
            for y in 0..h {
                a[y * w + x] = acc * inv;
                if y + r + 1 < h { acc += tmp[(y + r + 1) * w + x]; }
                if y >= r { acc -= tmp[(y - r) * w + x]; }
            }
        }
    }
}

// ------------------------------------------------------------------ shapes
#[derive(Clone)]
enum Geo { Poly(Vec<P>, bool), Mixed(Vec<(P, bool)>), Capsule(P, P, f64, f64), Ellipse(P, f64, f64, f64) }
impl Geo {
    fn emit(&self, ctx: &mut Ctx, ox: f64, oy: f64) {
        let o = |p: P| pt(p.x + ox, p.y + oy);
        match self {
            Geo::Poly(pts, smooth) => {
                if pts.len() < 2 { return; }
                if !smooth || pts.len() < 3 {
                    ctx.mv(o(pts[0]));
                    for p in &pts[1..] { ctx.ln(o(*p)); }
                    ctx.close();
                    return;
                }
                let n = pts.len();
                ctx.mv(o(lerp(pts[n - 1], pts[0], 0.5)));
                for i in 0..n {
                    let (p, q) = (pts[i], pts[(i + 1) % n]);
                    let m = o(lerp(p, q, 0.5));
                    let p = o(p);
                    ctx.quad_to(p.x, p.y, m.x, m.y);
                }
                ctx.close();
            }
            Geo::Mixed(pts) => {
                let n = pts.len();
                if n < 3 { return; }
                let mid = |i: usize| lerp(pts[i % n].0, pts[(i + 1) % n].0, 0.5);
                let s = pts.iter().position(|q| q.1).unwrap_or(0);
                let st = if pts[s].1 { pts[s].0 } else { mid(s) };
                ctx.mv(o(st));
                for k in 1..=n {
                    let i = (s + k) % n;
                    let q = pts[i];
                    if q.1 { ctx.ln(o(q.0)); } else {
                        let nx = pts[(i + 1) % n];
                        let e = o(if nx.1 { nx.0 } else { mid(i) });
                        let c = o(q.0);
                        ctx.quad_to(c.x, c.y, e.x, e.y);
                    }
                }
                ctx.close();
            }
            Geo::Capsule(a, b, ra, rb) => {
                let th = (b.y - a.y).atan2(b.x - a.x);
                let (a, b) = (o(*a), o(*b));
                ctx.move_to(a.x + (th + PI / 2.0).cos() * ra, a.y + (th + PI / 2.0).sin() * ra);
                ctx.arc(a.x, a.y, *ra, th + PI / 2.0, th + PI * 1.5);
                ctx.arc(b.x, b.y, *rb, th - PI / 2.0, th + PI / 2.0);
                ctx.close();
            }
            Geo::Ellipse(c, rx, ry, rot) => {
                let c = o(*c);
                ctx.move_to(c.x + rot.cos() * rx, c.y + rot.sin() * rx);
                ctx.ellipse(c.x, c.y, *rx, *ry, *rot, 0.0, PI * 2.0);
            }
        }
    }
}

#[derive(Clone)]
struct Shape { geo: Geo, fill: Rgb, shade: Option<f64>, rim: Option<f64>, line: bool, no_outline: bool, flat: bool }
fn shp(geo: Geo, fill: Rgb, shade: f64) -> Shape { Shape { geo, fill, shade: Some(shade), rim: None, line: false, no_outline: false, flat: false } }
impl Shape {
    fn rim(mut self, r: f64) -> Self { self.rim = Some(r); self }
    fn line(mut self, l: bool) -> Self { self.line = l; self }
}
fn poly(pts: Vec<P>, smooth: bool) -> Geo { Geo::Poly(pts, smooth) }

enum After { Body, Head, Hair, Charge(Vec<P>, f64) }
struct Layer { shapes: Vec<Shape>, after: Option<After>, aura: Option<Rgb> }
fn layer(shapes: Vec<Shape>) -> Layer { Layer { shapes, after: None, aura: None } }

struct BBox { x1: f64, y1: f64, x2: f64, y2: f64 }
impl BBox {
    fn new() -> Self { BBox { x1: f64::INFINITY, y1: f64::INFINITY, x2: f64::NEG_INFINITY, y2: f64::NEG_INFINITY } }
    fn add_p(&mut self, p: P, r: f64) { self.x1 = self.x1.min(p.x - r); self.y1 = self.y1.min(p.y - r); self.x2 = self.x2.max(p.x + r); self.y2 = self.y2.max(p.y + r); }
}
fn reg(bb: &mut BBox, pts: Vec<P>, r: f64) -> Vec<P> { for p in &pts { bb.add_p(*p, r); } pts }

/// tapered ribbon along a polyline
fn ribbon_pts(spine: &[P], w0: f64, w1: f64) -> Vec<P> {
    let n = spine.len();
    let mut l = Vec::with_capacity(n * 2);
    let mut r = Vec::with_capacity(n);
    for i in 0..n {
        let a = spine[i.saturating_sub(1)];
        let b = spine[(i + 1).min(n - 1)];
        let nn = norm(perp(sub(b, a)));
        let t = i as f64 / (n as f64 - 1.0);
        let w = (w0 + (w1 - w0) * t) / 2.0;
        l.push(add(spine[i], nn, w));
        r.push(add(spine[i], nn, -w));
    }
    r.reverse();
    l.extend(r);
    l
}
/// waving spine from start in direction angle (deg, dir() convention)
fn wave_spine(start: P, ang: f64, length: f64, amp: f64, phase: f64, n: usize, curl: f64) -> Vec<P> {
    let mut out = vec![start];
    let mut p = start;
    for i in 1..=n {
        let t = i as f64 / n as f64;
        let a = ang + (phase + t * 4.2).sin() * amp * t + curl * t;
        p = add(p, dir(a), length / n as f64);
        out.push(p);
    }
    out
}

// ------------------------------------------------------------------ skeleton
struct Lens { thigh: f64, torso: f64, neck: f64, up: f64, fore: f64, hand: f64 }
struct Skel {
    r: f64, fem: bool, s: f64, l: Lens, rot: f64, head_th: f64,
    hip: P, hip_f: P, hip_b: P, neck: P, head: P, sh_f: P, sh_b: P,
    el_f: P, el_b: P, wr_f: P, wr_b: P, hand_f: P, hand_b: P, up_af: f64, fore_af: f64, fore_ab: f64,
    kn_f: P, kn_b: P, an_f: P, an_b: P, th_af: f64, th_ab: f64, sh_af: f64, sh_ab: f64,
    up: P, fwd: P, torso_pts: Vec<P>, foot_f: Vec<P>, foot_b: Vec<P>, w: f64,
}

fn torso_local(w: f64, fem: bool, lt: f64, shape: &str) -> Vec<(f64, f64)> {
    let k = lt / 29.0;
    let mut pts: Vec<(f64, f64)> = if fem {
        vec![(-7.6 * w, -3.0), (-6.4 * w, 5.0), (-5.6 * w, 12.0), (-6.8 * w, 19.0), (-7.4 * w, 25.0), (-4.8 * w, 29.5), (-1.5, 31.0), (2.5 * w, 30.5), (6.0 * w, 27.0), (8.6 * w, 22.0), (8.4 * w, 17.5), (5.6 * w, 14.5), (5.6 * w, 9.0), (7.2 * w, 2.0), (6.2 * w, -3.0)]
    } else {
        vec![(-7.6 * w, -3.0), (-7.0 * w, 6.0), (-8.2 * w, 15.0), (-9.2 * w, 22.0), (-8.2 * w, 27.5), (-5.0 * w, 30.8), (-1.5, 31.6), (3.0 * w, 31.0), (7.6 * w, 27.5), (9.6 * w, 21.5), (8.4 * w, 15.0), (6.8 * w, 8.0), (7.4 * w, 1.0), (6.2 * w, -3.0)]
    };
    if shape == "heavy" { for p in pts.iter_mut() { if p.1 > 2.0 && p.1 < 20.0 { p.0 *= 1.12; } } }
    pts.into_iter().map(|(v, u)| (v, u * k)).collect()
}

struct Arm { up_a: f64, el: P, fore_a: f64, wr: P, hand: P }

fn solve(style: &Value, pose: &Value) -> Skel {
    let s = gnum(style, "scale", 1.0);
    let b = gnum(style, "build", 1.0);
    let fem = gt(style, "female");
    let pr = gv(style, "proportions").cloned().unwrap_or(Value::Null);
    let l = Lens {
        thigh: gnum(&pr, "thigh", if fem { 23.5 } else { 23.0 }) * s, torso: gnum(&pr, "torso", if fem { 27.0 } else { 29.0 }) * s,
        neck: gnum(&pr, "neck", 3.5) * s, up: gnum(&pr, "upperArm", if fem { 15.5 } else { 16.5 }) * s, fore: gnum(&pr, "foreArm", if fem { 14.5 } else { 15.0 }) * s,
        hand: 3.9 * s * b.powf(0.4),
    };
    let shin = gnum(&pr, "shin", if fem { 22.0 } else { 21.5 }) * s;
    let r = gnum(&pr, "head", if fem { 8.9 } else { 9.3 }) * s;
    let w = b * s;
    let rot = gnum(pose, "rot", 0.0);
    let lean = gnum(pose, "lean", 0.0);
    let torso_a = 180.0 - lean - rot;
    let head_a = torso_a - gnum(pose, "head", 0.0);
    let up = dir(torso_a);
    let fwd = dir(torso_a - 90.0);
    let pelv_a = 180.0 - rot;
    let pf = dir(pelv_a - 90.0);
    let hip = pt(0.0, 0.0);
    let tf = |v: f64, u: f64| add(add(hip, fwd, v), up, u);
    let hip_f = add(add(hip, pf, 2.2 * w), dir(pelv_a), -1.0);
    let hip_b = add(add(hip, pf, -2.6 * w), dir(pelv_a), 0.5);
    let tl = torso_local(w, fem, l.torso, gstr(style, "torsoShape").unwrap_or(""));
    let torso_pts: Vec<P> = tl.iter().map(|&(v, u)| tf(v, u)).collect();
    let neck = tf(0.6 * s, l.torso * 0.98);
    let head = add(add(neck, dir(head_a), l.neck + r * 0.82), dir(head_a - 90.0), 1.4 * s);
    let sh_f = tf(2.6 * w, l.torso * 0.86);
    let sh_b = tf(-3.6 * w, l.torso * 0.88);
    let arm = |sh: P, sa: f64, ea: f64, wa: f64| {
        let up_a = -lean - rot + sa;
        let el = add(sh, dir(up_a), l.up);
        let fore_a = up_a + ea;
        let wr = add(el, dir(fore_a), l.fore);
        let hand = add(wr, dir(fore_a + wa), l.hand * 0.85);
        Arm { up_a, el, fore_a, wr, hand }
    };
    let a_f = arm(sh_f, gnum(pose, "shoulderF", 20.0), gnum(pose, "elbowF", 70.0), gnum(pose, "wristF", 0.0));
    let mut a_b = arm(sh_b, gnum(pose, "shoulderB", 25.0), gnum(pose, "elbowB", 80.0), gnum(pose, "wristB", 0.0));
    if let Some(wpn) = gv(style, "weapon") {
        if truthy(Some(wpn)) && gt(wpn, "twoHand") && !gt(pose, "oneHand") && pose.get("weapon") != Some(&Value::Bool(false)) && !gt(pose, "noWeapon") {
            // two-handed grip: back hand reaches the handle just behind the front hand (2-bone IK)
            let ang = if is_num(gv(pose, "blade")) { gnum(pose, "blade", 0.0) - rot } else { a_f.fore_a + gnum(pose, "weapon", gnum(wpn, "angle", 75.0)) };
            let tgt = add(a_f.hand, dir(ang), -6.5 * s);
            let a = l.up;
            let bl = l.fore + l.hand * 0.85;
            let dv = sub(tgt, sh_b);
            let dist = ((a - bl).abs() + 0.1).max((a + bl - 0.05).min(len(dv)));
            let base = ang_of(dv);
            let aa = ((a * a + dist * dist - bl * bl) / (2.0 * a * dist)).clamp(-1.0, 1.0).acos() / D2R;
            let mut best: Option<Arm> = None;
            for sgn in [1.0, -1.0] {
                let up_a = base + sgn * aa;
                let el = add(sh_b, dir(up_a), a);
                let fore_a = ang_of(sub(tgt, el));
                let cand = Arm { up_a, el, fore_a, wr: add(el, dir(fore_a), l.fore), hand: add(el, dir(fore_a), bl) };
                if best.as_ref().is_none_or(|bb| cand.el.y > bb.el.y) { best = Some(cand); }
            }
            a_b = best.unwrap();
        }
    }
    let leg = |hp: P, ta: f64, ka: f64, an: Option<&Value>| {
        let th_a = ta - rot;
        let kn = add(hp, dir(th_a), l.thigh);
        let sh_a = th_a - ka;
        let ank = add(kn, dir(sh_a), shin);
        let ft_a = if is_num(an) { sh_a + 90.0 - num(an, 0.0) } else { 90.0 - rot };
        (th_a, kn, sh_a, ank, ft_a)
    };
    let l_f = leg(hip_f, gnum(pose, "thighF", 8.0), gnum(pose, "kneeF", 8.0), gv(pose, "ankleF"));
    let l_b = leg(hip_b, gnum(pose, "thighB", -8.0), gnum(pose, "kneeB", 6.0), gv(pose, "ankleB"));
    let foot_pts = |an: P, fa: f64| {
        let f = dir(fa);
        let dn = perp(f);
        const LOC: [(f64, f64); 8] = [(-2.6, -1.6), (-3.2, 2.6), (-2.4, 4.4), (8.8, 4.4), (10.4, 3.6), (9.2, 2.2), (4.0, 0.2), (1.6, -2.2)];
        LOC.iter().map(|&(a, b)| add(add(an, f, a * s), dn, b * s)).collect::<Vec<P>>()
    };
    let foot_f = foot_pts(l_f.3, l_f.4);
    let foot_b = foot_pts(l_b.3, l_b.4);
    // ---- grounding: lowest body point sits on y = 0 (+ pose.y offset), x offset by pose.x
    let mut max_y = f64::NEG_INFINITY;
    let mut consider = |p: P, rr: f64| { max_y = max_y.max(p.y + rr); };
    for p in foot_f.iter().chain(foot_b.iter()).chain(torso_pts.iter()) { consider(*p, 0.0); }
    consider(head, r); consider(l_f.1, 4.0 * w); consider(l_b.1, 4.0 * w); consider(a_f.el, 3.0 * w); consider(a_b.el, 3.0 * w);
    consider(a_f.hand, l.hand); consider(a_b.hand, l.hand);
    let dy = if gt(pose, "free") { -gnum(pose, "hipH", 34.0) * s + gnum(pose, "y", 0.0) } else { -max_y + gnum(pose, "y", 0.0) };
    let o = pt(gnum(pose, "x", 0.0), dy);
    let sh = |p: P| add(p, o, 1.0);
    Skel {
        r, fem, s, rot, head_th: (180.0 - head_a) * D2R,
        hip: sh(hip), hip_f: sh(hip_f), hip_b: sh(hip_b), neck: sh(neck), head: sh(head), sh_f: sh(sh_f), sh_b: sh(sh_b),
        el_f: sh(a_f.el), el_b: sh(a_b.el), wr_f: sh(a_f.wr), wr_b: sh(a_b.wr), hand_f: sh(a_f.hand), hand_b: sh(a_b.hand),
        up_af: a_f.up_a, fore_af: a_f.fore_a, fore_ab: a_b.fore_a,
        kn_f: sh(l_f.1), kn_b: sh(l_b.1), an_f: sh(l_f.3), an_b: sh(l_b.3), th_af: l_f.0, th_ab: l_b.0, sh_af: l_f.2, sh_ab: l_b.2,
        up, fwd, torso_pts: torso_pts.iter().map(|p| sh(*p)).collect(), foot_f: foot_f.iter().map(|p| sh(*p)).collect(), foot_b: foot_b.iter().map(|p| sh(*p)).collect(),
        w, l,
    }
}

// ------------------------------------------------------------------ weapon geometry
struct WeaponGeo { blade: Vec<P>, handle: Vec<P>, guard: Vec<P>, tip: P, base: P, edge: Option<Vec<P>>, ty: String }
fn weapon_geo(style: &Value, pose: &Value, sk: &Skel) -> Option<WeaponGeo> {
    let w = gv(style, "weapon")?;
    let ty = gstr(w, "type")?.to_string();
    if !truthy(gv(w, "type")) { return None; }
    if pose.get("weapon") == Some(&Value::Bool(false)) || gt(pose, "noWeapon") { return None; }
    let in_back = gstr(w, "hand").unwrap_or("F") == "B";
    let hand = if in_back { sk.hand_b } else { sk.hand_f };
    let fa = if in_back { sk.fore_ab } else { sk.fore_af };
    let s = sk.s;
    let ang = if is_num(gv(pose, "blade")) { gnum(pose, "blade", 0.0) - sk.rot } else { fa + gnum(pose, "weapon", gnum(w, "angle", 75.0)) };
    let d = dir(ang);
    let n = perp(d);
    let at = |a: f64, t: f64| add(add(hand, d, a * s), n, t * s);
    let lb = gnum(w, "length", match ty.as_str() { "greatsword" => 70.0, "katana" => 46.0, "cutlass" => 38.0, "kunai" => 11.0, "staff" => 60.0, _ => 40.0 });
    match ty.as_str() {
        "greatsword" => {
            let wd = gnum(w, "width", 11.0);
            let blade = vec![at(4.0, -wd * 0.42), at(lb * 0.55, -wd * 0.5), at(lb - 2.0, -wd * 0.48), at(lb + 6.0, -wd * 0.1), at(lb, wd * 0.52), at(lb * 0.35, wd * 0.5), at(10.0, wd * 0.5), at(8.0, wd * 0.25), at(5.5, wd * 0.35), at(4.0, wd * 0.42)];
            let edge = vec![at(lb * 0.35, wd * 0.5), at(lb, wd * 0.52), at(lb + 6.0, -wd * 0.1), at(lb + 2.5, -wd * 0.05), at(lb - 2.0, wd * 0.3), at(lb * 0.35, wd * 0.3)];
            Some(WeaponGeo { blade, edge: Some(edge), handle: vec![at(-11.0, -1.6), at(2.0, -1.8), at(2.0, 1.8), at(-11.0, 1.6)], guard: vec![at(1.5, -wd * 0.62), at(4.5, -wd * 0.62), at(4.5, wd * 0.62), at(1.5, wd * 0.62)], tip: at(lb + 6.0, 0.0), base: at(6.0, 0.0), ty })
        }
        "katana" | "cutlass" => {
            let cut = ty == "cutlass";
            let wd = gnum(w, "width", if cut { 4.2 } else { 2.8 });
            let curve = if cut { 3.2 } else { 1.6 };
            let mut pts = vec![];
            const N: usize = 8;
            for i in 0..=N { let t = i as f64 / N as f64; pts.push(at(3.0 + t * (lb - 3.0), -wd * 0.5 * (if cut { 1.0 + t * 0.5 } else { 1.0 }) - (t * PI * 0.9).sin() * curve * t)); }
            pts.push(at(lb + 4.0, -curve * 1.1 - wd * 0.2));
            for i in (0..=N).rev() { let t = i as f64 / N as f64; pts.push(at(3.0 + t * (lb - 3.0), wd * 0.5 * (if cut { 1.0 + t * 0.2 } else { 1.0 }) - (t * PI * 0.9).sin() * curve * t)); }
            let gw = if cut { 4.6 } else { 3.4 };
            let hb = if cut { -7.0 } else { -12.0 };
            let guard = if cut { vec![at(1.0, -gw), at(4.0, -gw), at(4.0, gw + 3.0), at(-5.0, gw + 4.0), at(-7.0, 2.0), at(1.0, gw * 0.5)] } else { vec![at(1.5, -gw), at(3.5, -gw), at(3.5, gw), at(1.5, gw)] };
            Some(WeaponGeo { blade: pts, handle: vec![at(hb, -1.4), at(2.0, -1.6), at(2.0, 1.6), at(hb, 1.4)], guard, tip: at(lb + 4.0, -curve), base: at(4.0, 0.0), edge: None, ty })
        }
        "kunai" => Some(WeaponGeo {
            blade: vec![at(3.0, -1.6), at(5.0, -2.6), at(lb, -0.2), at(lb + 2.0, 0.0), at(lb, 0.2), at(5.0, 2.6), at(3.0, 1.6)],
            handle: vec![at(-5.0, -1.0), at(3.0, -1.1), at(3.0, 1.1), at(-5.0, 1.0)], guard: vec![at(-8.5, -2.0), at(-5.0, -2.0), at(-5.0, 2.0), at(-8.5, 2.0)],
            tip: at(lb + 2.0, 0.0), base: at(4.0, 0.0), edge: None, ty,
        }),
        "staff" => Some(WeaponGeo { blade: vec![at(-lb * 0.45, -1.6), at(lb * 0.55, -1.6), at(lb * 0.55, 1.6), at(-lb * 0.45, 1.6)], handle: vec![], guard: vec![], tip: at(lb * 0.55, 0.0), base: at(0.0, 0.0), edge: None, ty }),
        _ => None,
    }
}

// ------------------------------------------------------------------ boxes
fn part_alias(p: &str) -> &str {
    match p {
        "handR" => "handF", "handL" => "handB", "footR" => "footF", "footL" => "footB", "fistF" => "handF", "fistB" => "handB", "kneeR" => "kneeF", "kneeL" => "kneeB",
        "elbowR" => "elbowF", "elbowL" => "elbowB", "hand" => "handF", "foot" => "footF", "fist" => "handF", "blade" | "sword" | "tip" => "weapon",
        _ => p,
    }
}
fn part_point(sk: &Skel, part: &str) -> Option<P> {
    Some(match part_alias(part) {
        "handF" => sk.hand_f, "handB" => sk.hand_b,
        "footF" => lerp(sk.foot_f[3], sk.foot_f[4], 0.5), "footB" => lerp(sk.foot_b[3], sk.foot_b[4], 0.5),
        "heelF" => sk.foot_f[1], "heelB" => sk.foot_b[1], "kneeF" => sk.kn_f, "kneeB" => sk.kn_b, "elbowF" => sk.el_f, "elbowB" => sk.el_b,
        "head" => sk.head, "shoulder" | "shoulderF" => sk.sh_f, "body" | "torso" => lerp(sk.hip, sk.neck, 0.55), "hip" => sk.hip,
        _ => return None,
    })
}
type B4 = [f64; 4];
fn bx(pts: &[P], pad: f64, pady: f64) -> B4 {
    let mut bb = BBox::new();
    for p in pts { bb.add_p(*p, 0.0); }
    [jround(bb.x1 - pad), jround(bb.y1 - pady), jround(bb.x2 + pad), jround(bb.y2 + pady)]
}
fn norm_box(b: B4) -> B4 { [b[0].min(b[2]), b[1].min(b[3]), b[0].max(b[2]), b[1].max(b[3])] }
fn compute_boxes(sk: &Skel, active: Option<&ActiveSpec>, wg: Option<&WeaponGeo>) -> (Vec<B4>, Vec<B4>) {
    let r = sk.r;
    let mut hurt = vec![
        bx(&[sk.head], r + 1.5, r + 1.5),
        bx(&[sk.neck, sk.sh_f, sk.sh_b, sk.hip, sk.hip_f, sk.hip_b, lerp(sk.hip, sk.neck, 0.5)], 6.0 * sk.w, 2.0),
        bx(&[sk.hip_f, sk.hip_b, sk.kn_f, sk.kn_b, sk.an_f, sk.an_b, sk.foot_f[3], sk.foot_b[3], sk.foot_f[2], sk.foot_b[2]], 3.5 * sk.w, 1.0),
    ];
    let mut hit = vec![];
    if let Some(a) = active {
        if let Some(b) = a.bx { hit.push([b[0] as f64, b[1] as f64, b[2] as f64, b[3] as f64]); } else {
            let part = part_alias(&a.part);
            let rad = a.radius.map(|x| x as f64).filter(|x| x.is_finite()).unwrap_or(9.0);
            if part == "weapon" && wg.is_some() {
                let wg = wg.unwrap();
                let mut pts = wg.blade.clone();
                pts.push(wg.tip);
                let pd = (rad * 0.35).max(1.0);
                hit.push(bx(&pts, pd, pd));
                hurt.push(bx(&[wg.base, sk.hand_f], 4.0, 4.0));
            } else if part == "aura" || part == "body" {
                hit.push(bx(&[sk.neck, sk.hip, sk.sh_f, sk.hand_f, sk.foot_f[3], sk.head], rad * 0.5, rad * 0.5));
            } else if let Some(p) = part_point(sk, part) {
                hit.push([jround(p.x - rad), jround(p.y - rad), jround(p.x + rad), jround(p.y + rad)]);
                // attacking limb stays vulnerable (slightly smaller than the hit box)
                let is_arm = part.starts_with("hand") || part.starts_with("elbow");
                let is_leg = part.starts_with("foot") || part.starts_with("knee") || part.starts_with("heel");
                let back = part.ends_with('B');
                if is_arm { hurt.push(bx(&[if back { sk.el_b } else { sk.el_f }, if back { sk.hand_b } else { sk.hand_f }], 3.5, 3.5)); }
                else if is_leg { hurt.push(bx(&[if back { sk.kn_b } else { sk.kn_f }, if back { sk.an_b } else { sk.an_f }], 3.5, 3.5)); }
            }
        }
    }
    (hurt.into_iter().map(norm_box).collect(), hit.into_iter().map(norm_box).collect())
}

// ------------------------------------------------------------------ layers
fn light() -> P { norm(pt(0.55, -0.85)) }

struct Rig<'a> {
    style: &'a Value, pose: &'a Value, sk: Skel, c: Colors, trail: f64, phase: f64, wind: f64,
    hair: Value, ht: String, helmet: bool, top: String, sleeves: String, pants: String,
}

impl<'a> Rig<'a> {
    fn h(&self, x: f64, y: f64) -> P {
        let (s, c) = self.sk.head_th.sin_cos();
        pt(self.sk.head.x + x * c - y * s, self.sk.head.y + x * s + y * c)
    }
    fn hp(&self, pts: &[(f64, f64)]) -> Vec<P> { let r = self.sk.r; pts.iter().map(|&(x, y)| self.h(x * r, y * r)).collect() }
    fn t(&self, v: f64, u: f64) -> P { add(add(self.sk.hip, self.sk.fwd, v), self.sk.up, u) }
    fn hair_type(&self) -> Option<&str> { gstr(&self.hair, "type") }

    fn flap(&self, hip_p: P, th_a: f64, sh_a: f64, length: f64, w0: f64, w1: f64, ph: f64, gravity: f64) -> Vec<P> {
        let d1 = norm(lerp(dir(th_a), dir(0.0), gravity));
        let mid_l = length.min(self.sk.l.thigh);
        let knee_p = add(hip_p, d1, mid_l);
        let mut d2 = norm(lerp(dir(sh_a), dir(self.trail * 0.4), 0.55));
        if d2.y < 0.45 { d2 = norm(lerp(d2, pt(d2.x * 0.3, 1.0), 0.7)); } // cloth never flows upwards
        let rest = (length - mid_l).max(0.0);
        let end = add(knee_p, d2, rest);
        let n1 = norm(perp(d1));
        let n2 = norm(perp(d2));
        let wv = (self.phase * 1.3 + ph).sin() * 1.6;
        vec![add(hip_p, n1, w0), add(knee_p, n1, (w0 + w1) / 2.0), add(end, n2, w1), add(lerp(end, knee_p, 0.02), d2, 2.0 + wv), add(end, n2, -w1 * 0.1), add(end, n2, -w1), add(knee_p, n1, -(w0 + w1) / 2.0), add(hip_p, n1, -w0)]
    }

    fn arm_shapes(&self, bb: &mut BBox, front: bool) -> Vec<Shape> {
        let sk = &self.sk;
        let (sh, el, wr, hd, fa) = if front { (sk.sh_f, sk.el_f, sk.wr_f, sk.hand_f, sk.fore_af) } else { (sk.sh_b, sk.el_b, sk.wr_b, sk.hand_b, sk.fore_ab) };
        let dk = |c: Rgb| if front { c } else { back_of(c) };
        let (w, s) = (sk.w, sk.s);
        let skin = self.c.c("skin");
        let top_c = self.c.c("top");
        let sleeves = self.sleeves.as_str();
        let mut out = vec![];
        let fm = |a: f64, b: f64| if sk.fem { a } else { b };
        let (up_r0, up_r1, fo_r0, fo_r1) = (3.8 * w * fm(0.82, 1.0), 3.0 * w * fm(0.82, 1.0), 3.2 * w * fm(0.82, 1.0), 2.5 * w * fm(0.85, 1.0));
        reg(bb, vec![sh, el, wr, hd], 6.0 * w);
        // upper arm + deltoid
        out.push(shp(Geo::Capsule(sh, el, up_r0, up_r1), dk(if matches!(sleeves, "none" | "wide" | "short") { skin } else { top_c }), 2.2));
        if sleeves == "none" && !sk.fem && w > 0.9 {
            out.push(shp(Geo::Ellipse(lerp(sh, el, 0.28), up_r0 * 1.18, up_r0 * 0.95, (el.y - sh.y).atan2(el.x - sh.x)), dk(skin), 2.4));
        }
        out.push(shp(Geo::Capsule(el, wr, fo_r0, fo_r1), dk(if sleeves == "long" { top_c } else { skin }), 2.0));
        if sleeves == "short" || sleeves == "gi" {
            let gi = sleeves == "gi";
            let end = if gi { lerp(el, wr, 0.35) } else { lerp(sh, el, 0.7) };
            let rr = up_r0 * if gi { 1.3 } else { 1.25 };
            if gi {
                out.push(shp(Geo::Capsule(sh, el, rr, rr * 0.95), dk(top_c), 2.6).line(true));
                let n = norm(perp(sub(wr, el)));
                let pts = reg(bb, vec![add(el, n, rr * 0.95), add(end, n, rr * 1.05), add(end, n, -rr * 1.05), add(el, n, -rr * 0.95)], 0.0);
                out.push(shp(poly(pts, false), dk(top_c), 2.2).line(true));
            } else {
                out.push(shp(Geo::Capsule(sh, end, rr, rr * 0.9), dk(top_c), 2.4).line(true));
            }
        }
        if sleeves == "wide" {
            let g = pt(0.0, 1.0);
            let d = norm(sub(wr, el));
            let n = norm(perp(sub(el, sh)));
            let s_end = add(el, d, sk.l.fore * 0.55);
            let n2 = norm(perp(d));
            let hang = add(g, dir(self.trail), 0.3);
            let pts = reg(bb, vec![add(sh, n, 4.5 * w), add(el, n, 5.0 * w), add(s_end, n2, 6.5 * w), add(add(s_end, n2, -6.5 * w), hang, 7.0 * s), add(add(el, n, -5.5 * w), hang, 4.0 * s), add(sh, n, -4.5 * w)], 0.0);
            out.push(shp(poly(pts, true), dk(top_c), 3.0).line(true));
        }
        // gloves / wraps
        let glove = gstr(self.style, "gloves").unwrap_or("none");
        let wrap_c = self.c.get("wrap").unwrap_or([0xec, 0xe6, 0xd8]);
        if matches!(glove, "gauntlet" | "glove" | "long") {
            let st = if glove == "long" { lerp(el, wr, 0.15) } else { lerp(el, wr, if glove == "gauntlet" { 0.4 } else { 0.7 }) };
            out.push(shp(Geo::Capsule(st, wr, fo_r0 * 1.18, fo_r1 * 1.22), dk(self.c.c("glove")), 2.0).line(true));
        } else if glove == "wrap" {
            out.push(shp(Geo::Capsule(lerp(el, wr, 0.7), wr, fo_r1 * 1.15, fo_r1 * 1.12), dk(wrap_c), 1.5).line(true));
        }
        // hand
        let hc = if glove == "none" || glove == "wrap" { skin } else { self.c.c("glove") };
        let open = gstr(self.pose, if front { "handF" } else { "handB" }) == Some("open");
        let hr = sk.l.hand;
        let d = dir(fa + gnum(self.pose, if front { "wristF" } else { "wristB" }, 0.0));
        if open {
            let n = perp(d);
            let base = add(wr, d, 0.5);
            let pts = reg(bb, vec![
                add(base, n, -hr * 0.85), add(add(base, d, hr * 1.4), n, -hr * 0.8), add(add(base, d, hr * 2.3), n, -hr * 0.4), add(base, d, hr * 2.5), add(add(base, d, hr * 2.2), n, hr * 0.5),
                add(add(base, d, hr * 1.2), n, hr * 0.95), add(add(base, d, hr * 1.1), n, hr * 1.5), add(add(base, d, hr * 0.4), n, hr * 1.3), add(base, n, hr * 0.9),
            ], 0.0);
            out.push(shp(poly(pts, true), dk(hc), 1.6));
        } else {
            out.push(shp(Geo::Ellipse(hd, hr * 1.12, hr * 0.98, d.y.atan2(d.x)), dk(hc), 1.6));
        }
        out
    }

    fn leg_shapes(&self, bb: &mut BBox, front: bool) -> Vec<Shape> {
        let sk = &self.sk;
        let (hp, kn, an, foot) = if front { (sk.hip_f, sk.kn_f, sk.an_f, &sk.foot_f) } else { (sk.hip_b, sk.kn_b, sk.an_b, &sk.foot_b) };
        let dk = |c: Rgb| if front { c } else { back_of(c) };
        let w = sk.w;
        let skin = self.c.c("skin");
        let pants_c = self.c.c("pants");
        let pt_ = self.pants.as_str();
        let mut out = vec![];
        reg(bb, vec![hp, kn, an], 7.0 * w);
        reg(bb, foot.clone(), 1.0);
        let baggy = match pt_ { "gi" => 1.28, "hakama" => 1.5, "baggy" => 1.35, _ => 1.0 };
        let leg_skin = matches!(pt_, "trunks" | "skirt" | "shorts");
        let (th_r0, th_r1, sh_r0, sh_r1) = (6.0 * w * if sk.fem { 0.92 } else { 1.0 }, 4.3 * w, 4.4 * w, 2.9 * w);
        let leg_col = if leg_skin { self.c.get("legs").unwrap_or(skin) } else { pants_c };
        let bg = if leg_skin { 1.0 } else { baggy };
        out.push(shp(Geo::Capsule(hp, kn, th_r0 * bg, th_r1 * bg), dk(leg_col), 2.8));
        // calf bulge
        let calf = add(lerp(kn, an, 0.32), norm(perp(sub(an, kn))), 0.9 * w);
        out.push(shp(Geo::Capsule(kn, an, sh_r0 * if leg_skin { 1.0 } else { baggy * 0.98 }, sh_r1 * if leg_skin { 1.0 } else { baggy * if pt_ == "gi" { 1.15 } else { 1.0 } }), dk(leg_col), 2.4));
        if leg_skin { out.push(shp(Geo::Ellipse(calf, sh_r0 * 1.02, sh_r0 * 1.25, (an.y - kn.y).atan2(an.x - kn.x) + PI / 2.0), dk(leg_col), 2.2)); }
        if pt_ == "trunks" || pt_ == "shorts" {
            out.push(shp(Geo::Capsule(hp, lerp(hp, kn, if pt_ == "shorts" { 0.5 } else { 0.2 }), th_r0 * 1.04, th_r0), dk(pants_c), 2.6).line(true));
        }
        // boots / feet
        let boot = gstr(self.style, "boots").unwrap_or("boot");
        let fcol = if boot == "barefoot" || boot == "wrap" { skin } else { self.c.c("boot") };
        if matches!(boot, "boot" | "greave" | "tall") {
            let st = lerp(kn, an, match boot { "tall" => 0.05, "greave" => 0.25, _ => 0.45 });
            out.push(shp(Geo::Capsule(st, an, sh_r0 * 1.12 * if boot == "tall" { 1.05 } else { 1.0 }, sh_r1 * 1.35), dk(self.c.c("boot")), 2.2).line(true));
            if boot == "greave" {
                let nn = norm(perp(sub(an, kn)));
                let pts = reg(bb, vec![add(st, nn, sh_r0 * 1.25), lerp(st, kn, 0.35), add(st, nn, -sh_r0 * 1.25)], 0.0);
                out.push(shp(poly(pts, false), dk(self.c.c("metal")), 1.0).line(true));
            }
        } else if boot == "wrap" || boot == "tabi" {
            out.push(shp(Geo::Capsule(lerp(kn, an, 0.72), an, sh_r1 * 1.3, sh_r1 * 1.25), dk(self.c.get("wrap").unwrap_or([0xec, 0xe6, 0xd8])), 1.5).line(true));
        }
        if pt_ == "gi" || pt_ == "baggy" {
            // pants cuff flare over the ankle
            let d = norm(sub(an, kn));
            let n = perp(d);
            let c0 = lerp(kn, an, 0.72);
            let c1 = add(an, d, -0.5);
            let fl = sh_r1 * baggy * 1.35;
            let pts = reg(bb, vec![add(c0, n, sh_r0 * baggy * 0.95), add(c1, n, fl), add(add(c1, n, fl * 0.2), d, 1.2), add(c1, n, -fl), add(c0, n, -sh_r0 * baggy * 0.95)], 0.0);
            out.push(shp(poly(pts, true), dk(pants_c), 2.2).line(true));
        }
        out.push(shp(poly(foot.clone(), true), dk(fcol), 1.6).line(boot != "barefoot"));
        out
    }

    fn robe_flap(&self, bb: &mut BBox, front: bool) -> Option<Shape> {
        let skirt = gv(self.style, "skirt").filter(|v| truthy(Some(v)))?;
        if gstr(skirt, "type") == Some("coat") { return None; }
        let sk = &self.sk;
        let sl = gnum(skirt, "length", 30.0) * sk.s;
        let col = self.c.cref(gv(skirt, "color"), "top");
        let hp = if front { sk.hip_f } else { sk.hip_b };
        let wst = add(hp, sk.up, 4.0);
        let (th_a, sh_a) = if front { (sk.th_af, sk.sh_af) } else { (sk.th_ab, sk.sh_ab) };
        let fw = gnum(skirt, "width", 1.0);
        // raised legs (kicks) make the cloth drape more instead of following the leg
        let rel = th_a + sk.rot;
        let grav = 0.25 + 0.6 * ((rel.abs() - 25.0) / 80.0).clamp(0.0, 1.0);
        let pts = reg(bb, self.flap(wst, th_a, sh_a, sl, 6.0 * sk.w * fw, 8.6 * sk.w * fw, if front { 0.0 } else { 2.0 }, grav), 0.0);
        Some(shp(poly(pts, true), if front { col } else { back_of(col) }, 3.0).line(true))
    }

    /// Builds every shape for the pose (pure geometry, no drawing)
    fn build_layers(&self, bb: &mut BBox, wg: Option<&WeaponGeo>) -> Vec<Layer> {
        let (style, pose, sk, c) = (self.style, self.pose, &self.sk, &self.c);
        let (s, w, r) = (sk.s, sk.w, sk.r);
        let (trail, phase) = (self.trail, self.phase);
        let mut layers = vec![];
        let skin = c.c("skin");
        let top_c = c.c("top");
        let pants_c = c.c("pants");
        let top = self.top.as_str();
        let pants_t = self.pants.as_str();
        let skirt = gv(style, "skirt").filter(|v| truthy(Some(v)));

        // ---------------- back accessories
        let mut back_l = vec![];
        if gt(style, "cape") {
            let cp = gv(style, "cape").unwrap();
            let clen = gnum(cp, "length", 55.0) * s;
            let a = add(sk.sh_b, sk.up, 2.0);
            let b2 = add(sk.sh_f, sk.up, 1.0);
            let mut pts = wave_spine(a, trail * 0.6, clen, 10.0, phase, 6, 0.0);
            let mut sp_b = wave_spine(b2, trail * 0.45, clen * 0.95, 9.0, phase + 1.3, 6, 0.0);
            sp_b.reverse();
            pts.extend(sp_b);
            let pts = reg(bb, pts, 0.0);
            back_l.push(shp(poly(pts, true), back_of(c.cref(gv(cp, "color"), "cloth")), 4.0));
        }
        let hair = &self.hair;
        let hcol = c.c("hair");
        let htype = self.hair_type();
        if htype == Some("long") || htype == Some("flowing") {
            let hl = gnum(hair, "length", 46.0) * s;
            let root1 = self.h(-0.85 * r, -0.4 * r);
            let root2 = self.h(0.1 * r, -0.95 * r);
            let sp1 = wave_spine(root1, 8.0 + trail * 0.5, hl, 8.0, phase, 7, 0.0);
            let mut sp2 = wave_spine(root2, -10.0 + trail * 0.55, hl * 1.05, 9.0, phase + 0.8, 7, 0.0);
            let mut tips = vec![];
            // jagged tip set
            let (e1, e2) = (*sp1.last().unwrap(), *sp2.last().unwrap());
            for i in 0..4 {
                let t = (i as f64 + 0.5) / 4.0;
                let base = lerp(e1, e2, t);
                tips.push(add(base, dir(trail * 0.5), 6.0 + hash(i as f64 + 3.0) * 6.0));
                tips.push(lerp(e1, e2, (i as f64 + 1.0) / 4.0));
            }
            let mut pts = sp1;
            pts.extend(tips);
            sp2.reverse();
            pts.extend(sp2);
            let pts = reg(bb, pts, 0.0);
            back_l.push(shp(poly(pts, true), mix(hcol, shade_of(hcol), 0.35), 4.0).rim(0.8));
        }
        if htype == Some("ponytail") {
            let hl = gnum(hair, "length", 40.0) * s;
            let root = self.h(-0.75 * r, -0.65 * r);
            let sp = wave_spine(root, trail * 0.9 - 5.0, hl, 16.0, phase, 8, -10.0);
            let pts = reg(bb, ribbon_pts(&sp, 8.0 * s, 1.5), 0.0);
            back_l.push(shp(poly(pts, true), hcol, 2.2));
        }
        if gt(style, "headband") {
            let hb = gv(style, "headband").unwrap();
            let knot = self.h(-0.95 * r, -0.48 * r);
            let l1 = gnum(hb, "tail", 26.0) * s;
            let col = c.cref(gv(hb, "color"), "top2");
            for i in 0..2 {
                let fi = i as f64;
                let sp = wave_spine(knot, trail * (0.95 + fi * 0.12) + fi * 8.0, l1 * (1.0 - fi * 0.18), 18.0, phase + fi * 1.7, 7, 0.0);
                let pts = reg(bb, ribbon_pts(&sp, 3.6 * s, 2.4 * s), 0.0);
                back_l.push(shp(poly(pts, true), if i > 0 { back_of(col) } else { col }, 1.0));
            }
        }
        if gt(style, "scarf") {
            let sc = gv(style, "scarf").unwrap();
            let st = add(sk.neck, sk.fwd, -3.0 * w);
            let l1 = gnum(sc, "length", 40.0) * s;
            let col = c.cref(gv(sc, "color"), "cloth");
            for i in 0..2 {
                let fi = i as f64;
                let sp = wave_spine(st, trail * (1.0 + fi * 0.1) + 6.0 * fi, l1 * (1.0 - fi * 0.25), 16.0, phase + fi * 2.0, 8, 0.0);
                let pts = reg(bb, ribbon_pts(&sp, 7.0 * s, 4.0 * s), 0.0);
                back_l.push(shp(poly(pts, true), if i > 0 { back_of(col) } else { col }, 2.0));
            }
        }
        if gt(style, "tail") {
            let tl = gv(style, "tail").unwrap();
            let sp = wave_spine(add(sk.hip, sk.fwd, -6.0 * w), -50.0 + trail * 0.4, gnum(tl, "length", 30.0) * s, 25.0, phase, 8, 40.0);
            let pts = reg(bb, ribbon_pts(&sp, 5.0 * s, 1.0), 0.0);
            back_l.push(shp(poly(pts, true), c.cref(gv(tl, "color"), "hair"), 1.5));
        }
        // coat tails (behind legs)
        if let Some(sk_) = skirt {
            if gstr(sk_, "type") == Some("coat") {
                let sl = gnum(sk_, "length", 40.0) * s;
                let col = c.cref(gv(sk_, "color"), "top");
                let wst = add(sk.hip, sk.up, 3.0);
                let p1 = reg(bb, self.flap(add(wst, sk.fwd, -3.0 * w), sk.th_ab - 12.0, sk.sh_ab - 20.0, sl, 6.0 * w, 9.0 * w, 0.0, 0.5), 0.0);
                back_l.push(shp(poly(p1, true), back_of(col), 3.0).line(true));
                let p2 = reg(bb, self.flap(add(wst, sk.fwd, -1.0 * w), sk.th_af * 0.5 - 14.0, sk.sh_af * 0.5 - 22.0, sl * 0.97, 6.0 * w, 9.0 * w, 1.4, 0.5), 0.0);
                back_l.push(shp(poly(p2, true), back_of(col), 3.0).line(true));
            }
        }
        if !back_l.is_empty() { layers.push(layer(back_l)); }

        // weapon
        let wl = wg.map(|wg| {
            let mut all = wg.blade.clone();
            all.extend(wg.handle.iter().copied());
            all.extend(wg.guard.iter().copied());
            all.push(wg.tip);
            reg(bb, all, 2.0);
            let wp = gv(style, "weapon").unwrap();
            let glow = if gt(wp, "glow") || gt(pose, "glow") { Some(c.cref(gv(wp, "glowColor"), "energy")) } else { None };
            let mut shapes = vec![];
            if !wg.handle.is_empty() { shapes.push(shp(poly(wg.handle.clone(), false), c.c("hilt"), 0.8)); }
            let blade_col = c.c("weapon");
            shapes.push(shp(poly(wg.blade.clone(), false), blade_col, if wg.ty == "greatsword" { 3.5 } else { 1.2 }).rim(0.6));
            if let Some(e) = &wg.edge {
                let mut sh = shp(poly(e.clone(), false), if c.has("weapon2") { light_of(blade_col) } else { WHITE }, 0.0);
                sh.no_outline = true;
                sh.flat = true;
                shapes.push(sh);
            }
            if !wg.guard.is_empty() { shapes.push(shp(poly(wg.guard.clone(), wg.ty == "cutlass"), c.c("accent"), 1.0)); }
            Layer { shapes, after: None, aura: glow }
        });
        let weapon_back = gv(style, "weapon").is_some_and(|wp| truthy(Some(wp)) && gstr(wp, "hand").unwrap_or("F") == "B");
        let mut wl = wl;
        if weapon_back { if let Some(l) = wl.take() { layers.push(l); } }
        layers.push(layer(self.arm_shapes(bb, false)));
        layers.push(layer(self.leg_shapes(bb, false)));
        if let Some(fb) = self.robe_flap(bb, false) { layers.push(layer(vec![fb])); }

        // ---------------- body
        let mut body = vec![];
        reg(bb, sk.torso_pts.clone(), 1.0);
        let torso_col = if top == "bare" { skin } else { top_c };
        let tr_sh = pants_t == "trunks" || pants_t == "shorts";
        body.push(shp(
            Geo::Ellipse(add(sk.hip, sk.up, 2.0), (if pants_t == "trunks" { 6.3 } else { 7.6 }) * w * (if pants_t == "gi" { 1.1 } else { 1.0 }), (if tr_sh { 4.2 } else { 6.2 }) * w, sk.fwd.y.atan2(sk.fwd.x)),
            if tr_sh { pants_c } else if pants_t == "skirt" { c.c("top") } else { pants_c }, 2.5,
        ));
        body.push(shp(poly(sk.torso_pts.clone(), true), torso_col, 4.2).rim(0.9));
        let neck_r = 2.9 * w.powf(0.8);
        body.push(shp(Geo::Capsule(sk.neck, self.h(-0.05 * r, 0.45 * r), neck_r * 1.05, neck_r), skin, 1.8));
        layers.push(Layer { shapes: body, after: Some(After::Body), aura: None });

        // belt / sash
        let belt_layer = if gt(style, "belt") {
            let bv = gv(style, "belt").unwrap();
            let bc = if bv == &Value::Bool(true) { c.c("belt") } else { c.cref(Some(bv), "") };
            let tl = sk.l.torso / 29.0;
            let (a0, a1, a2, a3) = (self.t(-8.2 * w, 0.5 * tl), self.t(8.4 * w, 0.5 * tl), self.t(8.0 * w, 5.0 * tl), self.t(-7.8 * w, 5.0 * tl));
            let mut shapes = vec![shp(poly(reg(bb, vec![a0, a1, a2, a3], 0.0), false), bc, 1.6).line(true)];
            let knot = self.t(5.5 * w, 2.5 * tl);
            let tails = gnum(style, "beltTails", 1.0);
            if tails != 0.0 {
                for i in 0..2 {
                    let fi = i as f64;
                    let sp = wave_spine(knot, 10.0 + fi * 18.0 + trail * 0.15, (10.0 + fi * 2.0) * s, 8.0, phase + fi, 4, 0.0);
                    let pts = reg(bb, ribbon_pts(&sp, 2.2 * s, 1.8 * s), 0.0);
                    shapes.push(shp(poly(pts, false), if i > 0 { back_of(bc) } else { bc }, 0.8).line(true));
                }
                shapes.push(shp(Geo::Ellipse(knot, 2.2 * s, 1.8 * s, 0.0), bc, 0.8));
            }
            Some(layer(shapes))
        } else { None };

        // ---------------- front leg + front flap + belt
        layers.push(layer(self.leg_shapes(bb, true)));
        if let Some(ff) = self.robe_flap(bb, true) { layers.push(layer(vec![ff])); }
        if let Some(bl) = belt_layer { layers.push(bl); }
        if gt(style, "cape") && gt(gv(style, "cape").unwrap(), "collar") {
            let cp = gv(style, "cape").unwrap();
            let pts = reg(bb, vec![add(sk.sh_b, sk.up, 3.0), add(sk.neck, sk.fwd, 4.0), add(sk.sh_f, sk.fwd, 3.0), add(add(sk.sh_f, sk.up, -4.0), sk.fwd, 2.0)], 0.0);
            layers.push(layer(vec![shp(poly(pts, true), c.cref(gv(cp, "color"), "cloth"), 1.5)]));
        }
        if gt(style, "scarf") {
            let sc = gv(style, "scarf").unwrap();
            let cc = add(sk.neck, sk.up, -1.0);
            layers.push(layer(vec![shp(Geo::Ellipse(cc, 5.6 * s, 3.6 * s, sk.fwd.y.atan2(sk.fwd.x) - 0.15), c.cref(gv(sc, "color"), "cloth"), 1.6)]));
        }

        // ---------------- head
        let helmet = self.helmet;
        let mut hpts: Vec<(f64, f64)> = (0..13).map(|i| { let a = (140.0 + i as f64 * (230.0 / 12.0)) * D2R; (a.cos() * 0.93, a.sin()) }).collect();
        hpts.extend([(0.93, 0.48), (0.62, 0.98), (0.2, 1.0), (-0.3, 0.74)]);
        let head_pts = reg(bb, self.hp(&hpts), 1.0);
        let mut head_shapes = vec![shp(poly(head_pts, true), if helmet { c.c("metal") } else { skin }, 2.6).rim(0.7)];
        let ear_p = self.h(-0.12 * r, 0.22 * r);
        if !helmet { head_shapes.push(shp(Geo::Ellipse(ear_p, 0.2 * r, 0.27 * r, sk.head_th), skin, 0.8)); }
        layers.push(Layer { shapes: head_shapes, after: Some(After::Head), aura: None });

        // mask (lower face)
        if gt(style, "mask") {
            let m = self.hp(&[(-0.1, 0.32), (0.55, 0.36), (1.02, 0.36), (0.98, 0.56), (0.64, 1.04), (0.18, 1.04), (-0.32, 0.8), (-0.3, 0.42)]);
            layers.push(layer(vec![shp(poly(reg(bb, m, 0.0), true), c.c("mask"), 1.6).line(true)]));
        }

        // ---------------- hair
        let mut hair_shapes = vec![];
        let ht = self.ht.as_str();
        if ht != "none" && ht != "bald" && !helmet {
            let n = gnum(hair, "spikes", match ht { "spiky" => 8.0, "wild" | "mane" => 9.0, "short" => 7.0, _ => 6.0 });
            let lmin = gnum(hair, "min", match ht { "spiky" => 0.7, "wild" | "mane" => 0.7, "short" => 0.22, _ => 0.3 });
            let lmax = gnum(hair, "max", match ht { "spiky" => 1.5, "wild" | "mane" => 1.5, "short" => 0.5, _ => 0.55 });
            let sweep = gnum(hair, "sweep", if ht == "spiky" { 28.0 } else { 40.0 });
            let a0 = match ht { "mane" | "wild" => 115.0, "long" | "flowing" => 130.0, _ => 150.0 };
            let a1 = gnum(hair, "front", 318.0);
            let hair_wind = gnum(pose, "hairWind", self.wind) * 14.0;
            let seed = gnum(hair, "seed", 0.0);
            // hairline back -> front (inner), then spikes front -> back (outer)
            let mut pts: Vec<(f64, f64)> = vec![(-0.28, 0.2), (-0.05, -0.08), (0.25, -0.42), (0.62, -0.42), (0.88, -0.38)];
            let mut i = n - 1.0;
            while i >= 0.0 {
                let t0 = i / n;
                let t1 = (i + 1.0) / n;
                let a_v = (a0 + (a1 - a0) * t1) * D2R;
                let a_t = a0 + (a1 - a0) * (t0 + t1) / 2.0;
                let ln = lmin + (lmax - lmin) * (0.45 + 0.55 * (PI * ((t0 + 0.15) * 1.05).min(1.0)).sin()) * (0.75 + 0.5 * hash(i + n * 7.0 + seed));
                let v_r = 1.06;
                pts.push((a_v.cos() * v_r * 0.95, a_v.sin() * v_r));
                let td = (a_t - sweep * (0.6 + 0.4 * (1.0 - t0)) - hair_wind) * D2R;
                let base = ((a_t * D2R).cos() * 0.95, (a_t * D2R).sin());
                pts.push((base.0 + td.cos() * ln, base.1 + td.sin() * ln));
                i -= 1.0;
            }
            let a_e = a0 * D2R;
            pts.push((a_e.cos() * 1.04 * 0.95, a_e.sin() * 1.04));
            if matches!(ht, "mane" | "wild" | "long" | "flowing") { pts.extend([(-0.7, 0.95), (-0.45, 0.6)]); } else { pts.extend([(-0.62, 0.72), (-0.42, 0.45)]); }
            let hp = reg(bb, self.hp(&pts), 1.0);
            hair_shapes.push(shp(Geo::Mixed(hp.into_iter().enumerate().map(|(i, p)| (p, i >= 5 && (i - 5) % 2 == 1)).collect()), hcol, 3.0).rim(0.9));
            // bangs
            let bangs = gnum(hair, "bangs", match ht { "spiky" => 3.0, "short" => 2.0, _ => 3.0 });
            let mut i = 0.0;
            while i < bangs {
                let bx_ = 0.22 + i * 0.26;
                let by = -0.62 - if i == 0.0 { 0.1 } else { 0.0 };
                let tip_l = gnum(hair, "bangLen", if matches!(ht, "long" | "flowing" | "ponytail") { 0.55 } else { 0.42 }) * (1.0 - i * 0.15);
                let tip = (bx_ + 0.18 + i * 0.05, by + tip_l + 0.15);
                let bp = self.hp(&[(bx_ - 0.2, by - 0.15), (bx_ + 0.25, by - 0.05), tip]);
                hair_shapes.push(shp(Geo::Mixed(bp.into_iter().enumerate().map(|(k, p)| (p, k == 2)).collect()), hcol, 1.6));
                i += 1.0;
            }
            if ht == "topknot" || ht == "bun" {
                let k = self.hp(&[(-0.55, -1.05)]);
                hair_shapes.push(shp(Geo::Ellipse(k[0], 0.38 * r, 0.32 * r, sk.head_th), hcol, 1.5));
            }
        }
        if helmet {
            // android helmet: crest + visor
            let crest = self.hp(&[(-0.9, -0.2), (-0.6, -0.95), (0.1, -1.18), (0.65, -0.9), (0.95, -0.35), (0.98, -0.05), (0.5, -0.12), (-0.3, 0.1), (-0.55, 0.5), (-0.85, 0.4)]);
            hair_shapes.push(shp(poly(reg(bb, crest, 0.0), true), c.c("hair"), 2.4).rim(0.8).line(true));
            let fin = self.hp(&[(-0.5, -0.95), (-1.45, -1.3), (-1.6, -1.1), (-0.85, -0.55)]);
            hair_shapes.push(shp(poly(reg(bb, fin, 0.0), false), c.c("accent"), 0.8));
        }
        if !hair_shapes.is_empty() { layers.push(Layer { shapes: hair_shapes, after: Some(After::Hair), aura: None }); }

        // headband / horns / hat
        let mut top_shapes = vec![];
        if gt(style, "headband") {
            let hb = gv(style, "headband").unwrap();
            let col = c.cref(gv(hb, "color"), "top2");
            let band = self.hp(&[(-0.98, -0.6), (-0.3, -0.8), (0.5, -0.78), (0.97, -0.6), (1.0, -0.38), (0.5, -0.53), (-0.3, -0.55), (-0.98, -0.36)]);
            top_shapes.push(shp(poly(reg(bb, band, 0.0), true), col, 1.0).line(true));
            top_shapes.push(shp(Geo::Ellipse(self.h(-0.98 * r, -0.48 * r), 2.0 * s, 1.6 * s, 0.0), col, 0.6));
        }
        if gt(style, "horns") {
            let hc = c.c("horn");
            let big = gnum(gv(style, "horns").unwrap(), "size", 1.0);
            let mut horn = |bx_: f64, by: f64, ang: f64, l: f64, wd: f64, curl: f64| {
                let base = self.h(bx_ * r, by * r);
                const N: usize = 6;
                let spine: Vec<P> = (0..=N).map(|i| { let t = i as f64 / N as f64; add(base, dir(ang - (sk.head_th / D2R) + curl * t * t), l * t) }).collect();
                reg(bb, ribbon_pts(&spine, wd, 0.3), 0.0)
            };
            let h1 = horn(-0.35, -0.8, 160.0, 14.0 * big * s, 4.2 * s, -45.0);
            let h2 = horn(0.35, -0.85, 155.0, 17.0 * big * s, 5.0 * s, 55.0);
            top_shapes.push(shp(poly(h1, true), back_of(hc), 1.2));
            top_shapes.push(shp(poly(h2, true), hc, 1.6));
        }
        if gt(style, "hat") {
            let hc = c.c("hat");
            let brim = self.hp(&[(-1.6, -0.45), (-0.9, -0.75), (0.0, -0.82), (0.9, -0.72), (1.65, -0.5), (1.5, -0.3), (0.6, -0.5), (-0.6, -0.5), (-1.4, -0.3)]);
            let crown = self.hp(&[(-0.85, -0.6), (-0.75, -1.35), (0.0, -1.55), (0.75, -1.35), (0.85, -0.6)]);
            top_shapes.push(shp(poly(reg(bb, crown, 0.0), true), hc, 2.0).line(true));
            top_shapes.push(shp(poly(reg(bb, brim, 0.0), true), hc, 1.2).line(true));
            let band = self.hp(&[(-0.83, -0.62), (0.83, -0.62), (0.8, -0.82), (-0.8, -0.82)]);
            let mut b = shp(poly(band, false), c.c("accent"), 0.5);
            b.no_outline = true;
            top_shapes.push(b);
        }
        if !top_shapes.is_empty() { layers.push(layer(top_shapes)); }

        // ---------------- front arm (+weapon in front hand before it)
        if let Some(l) = wl.take() { layers.push(l); }
        layers.push(layer(self.arm_shapes(bb, true)));
        if gt(style, "shoulderPads") {
            let d = dir(sk.up_af);
            layers.push(layer(vec![shp(Geo::Ellipse(add(sk.sh_f, d, 1.5), 6.2 * w, 4.4 * w, d.y.atan2(d.x) + PI / 2.0), c.c("metal"), 1.8).rim(0.6).line(true)]));
        }
        // charge / energy in hands
        if gt(pose, "charge") {
            let ch_ = gstr(pose, "charge");
            let which = if ch_ == Some("both") { vec![sk.hand_f, sk.hand_b] } else if ch_ == Some("B") { vec![sk.hand_b] } else { vec![sk.hand_f] };
            let rr = gnum(pose, "chargeSize", 7.0) * s;
            for p in &which { bb.add_p(*p, rr * 1.8); }
            layers.push(Layer { shapes: vec![], after: Some(After::Charge(which, rr)), aura: None });
        }
        layers
    }

    // ---------------------------------------------------------------- drawing
    fn draw_after(&self, ctx: &mut Ctx, a: &After) {
        match a {
            After::Body => self.draw_body(ctx),
            After::Head => self.draw_head(ctx),
            After::Hair => self.draw_hair(ctx),
            After::Charge(ps, rr) => { for p in ps { glow_orb(ctx, *p, *rr, self.c.c("energy"), self.phase); } }
        }
    }

    fn draw_body(&self, ctx: &mut Ctx) {
        let (sk, c, style) = (&self.sk, &self.c, self.style);
        let (w, s) = (sk.w, sk.s);
        let tl = sk.l.torso / 29.0;
        let t = |v: f64, u: f64| self.t(v, u);
        let skin = c.c("skin");
        let top_c = c.c("top");
        let ol = c.c("outline");
        let line = |ctx: &mut Ctx, pts: &[P], col: Fill, wd: f64| { ctx.begin(); ctx.mv(pts[0]); for p in &pts[1..] { ctx.ln(*p); } ctx.stroke_w(&col, wd); };
        let fill_pts = |ctx: &mut Ctx, pts: Vec<P>, col: Rgb, smooth: bool| { ctx.begin(); poly(pts, smooth).emit(ctx, 0.0, 0.0); ctx.fill(&sol(col, 1.0)); };
        let dot = |ctx: &mut Ctx, p: P, r: f64, col: Rgb| { ctx.begin(); ctx.arc(p.x, p.y, r, 0.0, PI * 2.0); ctx.fill(&sol(col, 1.0)); };
        ctx.save();
        ctx.begin();
        poly(sk.torso_pts.clone(), true).emit(ctx, 0.0, 0.0);
        ctx.clip();
        let top = self.top.as_str();
        if top == "gi" || top == "robe" {
            let v_bot = t(3.5 * w, 12.0 * tl);
            let skin_v = vec![t(-1.2, 32.0 * tl), t(6.5 * w, 31.0 * tl), v_bot];
            fill_pts(ctx, skin_v.clone(), skin, false);
            if gt(style, "undershirt") { fill_pts(ctx, skin_v, c.cref(gv(style, "undershirt"), ""), false); }
            // pec line
            if !gt(style, "undershirt") && !sk.fem { line(ctx, &[t(2.5 * w, 21.0 * tl), t(5.0 * w, 19.5 * tl), t(7.5 * w, 21.0 * tl)], sol(shade_of(skin), 0.9), 0.5); }
            let trim = c.c("trim");
            let a = [t(-2.5, 32.5 * tl), t(-0.5 * w, 20.0 * tl), v_bot, t(5.5 * w, 2.0 * tl)];
            let b = [t(8.0 * w, 30.0 * tl), v_bot];
            line(ctx, &a, sol(trim, 1.0), 2.4 * s);
            line(ctx, &b, sol(trim, 1.0), 2.4 * s);
            line(ctx, &a, sol(ol, 1.0), 0.45);
            line(ctx, &b, sol(ol, 1.0), 0.45);
            // fold lines
            line(ctx, &[t(-6.0 * w, 18.0 * tl), t(-3.0 * w, 10.0 * tl)], sol(shade_of(top_c), 0.8), 0.5);
        } else if top == "bare" || top == "tank" || top == "vest" {
            let sh = sol(shade_of(skin), 0.95);
            if top != "bare" {
                // tank top body
                fill_pts(ctx, vec![t(-9.0 * w, 30.0 * tl), t(-2.0, 31.0 * tl), t(1.5 * w, 26.0 * tl), t(5.5 * w, 27.0 * tl), t(9.5 * w, 24.0 * tl), t(10.0 * w, -3.0), t(-9.0 * w, -3.0)], top_c, false);
                if top == "vest" { fill_pts(ctx, vec![t(1.0 * w, 30.0 * tl), t(9.6 * w, 26.0 * tl), t(9.6 * w, 3.0), t(4.5 * w, 3.0)], skin, false); }
            } else {
                line(ctx, &[t(1.5 * w, 23.0 * tl), t(5.0 * w, 18.5 * tl), t(8.6 * w, 20.0 * tl)], sh.clone(), 0.7);
                line(ctx, &[t(4.5 * w, 15.0 * tl), t(4.5 * w, 3.0 * tl)], sh.clone(), 0.5);
                for u in [13.0, 9.0, 5.0] { line(ctx, &[t(2.2 * w, u * tl), t(6.6 * w, (u + 0.6) * tl)], sh.clone(), 0.45); }
                line(ctx, &[t(-4.0 * w, 24.0 * tl), t(-6.0 * w, 14.0 * tl)], sh, 0.5);
            }
        } else if top == "jacket" || top == "coat" {
            fill_pts(ctx, vec![t(-0.5, 32.0 * tl), t(7.5 * w, 30.0 * tl), t(9.4 * w, 22.0 * tl), t(7.5 * w, 6.0 * tl), t(4.2 * w, 6.0 * tl), t(1.5 * w, 22.0 * tl)], c.c("top2"), false);
            let lp = [t(-0.5, 32.0 * tl), t(1.5 * w, 22.0 * tl), t(4.2 * w, 6.0 * tl), t(4.4 * w, -3.0)];
            line(ctx, &lp, sol(c.c("trim"), 1.0), 1.8 * s);
            line(ctx, &lp, sol(ol, 1.0), 0.4);
            // collar
            let col = vec![t(-5.0 * w, 33.5 * tl), t(-0.8, 35.0 * tl), t(2.2 * w, 28.0 * tl), t(-0.8, 27.0 * tl)];
            fill_pts(ctx, col.clone(), c.c("trim"), false);
            let mut cl = col;
            cl.push(t(-5.0 * w, 33.5 * tl));
            line(ctx, &cl, sol(ol, 1.0), 0.4);
            for u in [18.0, 12.0] { dot(ctx, t(6.0 * w, u * tl), 0.8, c.c("accent")); }
        } else if top == "qipao" {
            let trim = c.c("trim");
            line(ctx, &[t(1.0 * w, 32.0 * tl), t(3.5 * w, 27.0 * tl), t(9.5 * w, 23.0 * tl)], sol(trim, 1.0), 1.5 * s);
            line(ctx, &[t(-1.0, 32.0 * tl), t(-2.0, 34.0 * tl), t(3.0, 34.5 * tl), t(4.0 * w, 31.0 * tl)], sol(trim, 1.0), 1.6 * s);
            line(ctx, &[t(3.5 * w, 27.0 * tl), t(9.5 * w, 23.0 * tl)], sol(ol, 1.0), 0.35);
            for u in [26.0, 24.5] { dot(ctx, t(5.5 * w + (26.0 - u), u * tl), 0.7, c.c("accent")); }
            // embroidery
            ctx.begin();
            let e = t(-2.0 * w, 16.0 * tl);
            ctx.arc(e.x, e.y, 3.0 * w, 0.5, 4.2);
            ctx.stroke_w(&sol(c.c("accent"), 0.8), 0.6);
        } else if top == "armor" || top == "suit" {
            let glow_c = c.c("visor");
            let pp = vec![t(-6.0 * w, 27.0 * tl), t(-1.0, 30.0 * tl), t(8.0 * w, 27.0 * tl), t(9.4 * w, 18.0 * tl), t(6.0 * w, 12.0 * tl), t(-6.0 * w, 14.0 * tl)];
            fill_pts(ctx, pp.clone(), c.c("metal"), true);
            let mut lp = pp;
            lp.push(t(-6.0 * w, 27.0 * tl));
            line(ctx, &lp, sol(ol, 1.0), 0.5);
            if top == "suit" {
                ctx.save();
                ctx.set_shadow(rgba(glow_c, 1.0), 6.0);
                line(ctx, &[t(2.0 * w, 24.0 * tl), t(5.5 * w, 20.0 * tl), t(5.5 * w, 15.0 * tl)], sol(glow_c, 1.0), 0.9);
                line(ctx, &[t(4.5 * w, 10.0 * tl), t(6.0 * w, 2.0 * tl)], sol(glow_c, 1.0), 0.8);
                let cc = t(3.2 * w, 21.0 * tl);
                dot(ctx, cc, 2.2 * w, glow_c);
                ctx.restore();
                ctx.begin();
                ctx.arc(cc.x, cc.y, 2.2 * w, 0.0, PI * 2.0);
                ctx.stroke_w(&sol(ol, 1.0), 0.5);
                dot(ctx, cc, 1.1 * w, WHITE);
            }
            for u in [8.0, 4.0] { line(ctx, &[t(-6.0 * w, u * tl), t(7.0 * w, u * tl)], sol(shade_of(top_c), 0.9), 0.5); }
        }
        ctx.restore();
    }

    fn draw_head(&self, ctx: &mut Ctx) {
        let (sk, c, style) = (&self.sk, &self.c, self.style);
        let r = sk.r;
        let skin = c.c("skin");
        let ol = sol(c.c("outline"), 1.0);
        let helmet = self.helmet;
        let face = gstr(self.pose, "face").unwrap_or("neutral");
        let eyes_t = gstr(style, "eyes").unwrap_or("normal");
        ctx.save();
        ctx.translate(sk.head.x, sk.head.y);
        ctx.rotate(sk.head_th);
        ctx.scale(r, r);
        if !helmet {
            // ear detail
            ctx.begin(); ctx.arc(-0.12, 0.22, 0.12, -1.2, 1.6); ctx.stroke_w(&sol(shade_of(skin), 1.0), 0.06);
            // blush / cheek shade
            ctx.begin(); ctx.ellipse(0.55, 0.52, 0.16, 0.07, 0.0, 0.0, PI * 2.0); ctx.fill(&sol([255, 0x80, 0x80], if sk.fem { 0.35 } else { 0.12 }));
        }
        let eye = |ctx: &mut Ctx, cx: f64, cy: f64, w: f64, far: bool| {
            let ang = if eyes_t == "fierce" || face == "shout" || face == "angry" { 0.22 } else if eyes_t == "cute" { -0.05 } else { 0.08 };
            let hgt = (match eyes_t { "cute" => 0.3, "fierce" => 0.17, _ => 0.23 }) * if sk.fem { 1.15 } else { 1.0 };
            if face == "hurt" || face == "ko" || face == "closed" {
                ctx.begin();
                if face == "hurt" { ctx.move_to(cx - w * 0.5, cy - hgt * 0.4); ctx.line_to(cx + w * 0.1, cy); ctx.line_to(cx - w * 0.4, cy + hgt * 0.35); }
                else { ctx.move_to(cx - w * 0.5, cy); ctx.quad_to(cx, cy + 0.08, cx + w * 0.5, cy); }
                ctx.stroke_w(&ol, 0.09);
                return;
            }
            if eyes_t == "glow" {
                let ec = c.c("eye");
                ctx.save();
                ctx.set_shadow(rgba(ec, 1.0), 8.0);
                ctx.begin(); ctx.move_to(cx - w * 0.55, cy - hgt * 0.15 - ang * 0.3); ctx.line_to(cx + w * 0.5, cy + ang * 0.4); ctx.line_to(cx - w * 0.3, cy + hgt * 0.5); ctx.close();
                ctx.fill(&sol(ec, 1.0));
                ctx.restore();
                ctx.begin(); ctx.move_to(cx - w * 0.55, cy - hgt * 0.15 - ang * 0.3); ctx.line_to(cx + w * 0.5, cy + ang * 0.4); ctx.stroke_w(&ol, 0.07);
                return;
            }
            // sclera
            ctx.begin();
            ctx.move_to(cx - w * 0.5, cy - hgt * 0.35 - ang * 0.3); ctx.quad_to(cx, cy - hgt * 0.75, cx + w * 0.5, cy - hgt * 0.2 + ang * 0.25);
            ctx.line_to(cx + w * 0.42, cy + hgt * 0.45); ctx.quad_to(cx, cy + hgt * 0.62, cx - w * 0.45, cy + hgt * 0.35); ctx.close();
            ctx.fill(&sol(WHITE, 1.0));
            ctx.save();
            ctx.clip();
            // iris
            let ix = cx + w * if far { 0.12 } else { 0.1 };
            let ec = c.c("eye");
            let g = Fill::Lin(0.0, cy - hgt * 0.6, 0.0, cy + hgt * 0.6, vec![(0.0, rgba(shade_of(ec), 1.0)), (1.0, rgba(light_of(ec), 1.0))]);
            ctx.begin(); ctx.ellipse(ix, cy + hgt * 0.05, w * 0.28, hgt * 0.62, 0.0, 0.0, PI * 2.0); ctx.fill(&g);
            ctx.begin(); ctx.ellipse(ix + w * 0.02, cy + hgt * 0.05, w * 0.12, hgt * 0.32, 0.0, 0.0, PI * 2.0); ctx.fill(&sol([0x0c, 0x08, 0x14], 1.0));
            ctx.begin(); ctx.arc(ix - w * 0.08, cy - hgt * 0.2, hgt * 0.17, 0.0, PI * 2.0); ctx.fill(&sol(WHITE, 1.0));
            ctx.restore();
            // upper lid
            ctx.begin(); ctx.move_to(cx - w * 0.6, cy - hgt * 0.25 - ang * 0.35); ctx.quad_to(cx, cy - hgt * 0.8, cx + w * 0.55, cy - hgt * 0.2 + ang * 0.3);
            ctx.stroke_w(&ol, 0.12);
            if sk.fem { ctx.begin(); ctx.move_to(cx + w * 0.45, cy - hgt * 0.25 + ang * 0.3); ctx.line_to(cx + w * 0.68, cy - hgt * 0.42 + ang * 0.3); ctx.stroke_w(&ol, 0.07); }
        };
        let brow = |ctx: &mut Ctx, cx: f64, cy: f64, w: f64| {
            let ang = if eyes_t == "fierce" || face == "shout" || face == "angry" || face == "hurt" { 0.22 } else { 0.02 };
            ctx.begin(); ctx.move_to(cx - w * 0.55, cy - 0.06 - ang * 0.2); ctx.line_to(cx + w * 0.5, cy + ang * 0.6);
            ctx.stroke_w(&sol(mix(c.c("hair"), c.c("outline"), 0.5), 1.0), if sk.fem { 0.06 } else { 0.1 });
        };
        if !helmet || style.get("visor") == Some(&Value::Bool(false)) {
            eye(ctx, 0.36, 0.16, 0.36, false);
            eye(ctx, 0.8, 0.15, 0.2, true);
            brow(ctx, 0.36, -0.12, 0.42);
            brow(ctx, 0.8, -0.12, 0.22);
            // nose
            ctx.begin(); ctx.move_to(0.95, 0.3); ctx.line_to(1.0, 0.44); ctx.line_to(0.92, 0.46); ctx.stroke_w(&sol(shade_of(skin), 1.0), 0.06);
            // mouth
            if !gt(style, "mask") {
                ctx.begin();
                if face == "shout" || face == "hurt" || face == "ko" {
                    ctx.move_to(0.48, 0.66); ctx.quad_to(0.68, 0.6, 0.82, 0.64); ctx.line_to(0.75, 0.82); ctx.quad_to(0.6, 0.86, 0.5, 0.74); ctx.close();
                    ctx.fill(&sol([0x5a, 0x1a, 0x26], 1.0));
                    ctx.stroke_w(&ol, 0.06);
                } else if face == "smile" {
                    ctx.move_to(0.5, 0.66); ctx.quad_to(0.66, 0.78, 0.82, 0.64); ctx.stroke_w(&ol, 0.07);
                } else {
                    ctx.move_to(0.54, 0.7); ctx.line_to(0.8, 0.68);
                    ctx.stroke_w(&if sk.fem { sol(c.c("lips"), 1.0) } else { ol.clone() }, 0.07);
                }
            }
        }
        if gt(style, "scar") { ctx.begin(); ctx.move_to(0.5, -0.05); ctx.line_to(0.75, 0.42); ctx.stroke_w(&sol([0xa0, 0x30, 0x40], 0.9), 0.06); }
        if gt(style, "facePaint") {
            ctx.begin(); ctx.move_to(0.25, 0.42); ctx.line_to(0.62, 0.4); ctx.move_to(0.3, 0.52); ctx.line_to(0.6, 0.5);
            ctx.stroke_w(&sol(c.cref(gv(style, "facePaint"), ""), 1.0), 0.07);
        }
        ctx.restore();
    }

    fn draw_hair(&self, ctx: &mut Ctx) {
        let c = &self.c;
        if self.helmet && self.style.get("visor") != Some(&Value::Bool(false)) {
            let v = self.hp(&[(0.12, 0.0), (1.02, -0.02), (1.0, 0.32), (0.2, 0.3)]);
            let vc = c.c("visor");
            ctx.save();
            ctx.set_shadow(rgba(vc, 1.0), 10.0);
            ctx.begin(); poly(v.clone(), false).emit(ctx, 0.0, 0.0); ctx.fill(&sol(vc, 1.0));
            ctx.restore();
            ctx.begin(); poly(v, false).emit(ctx, 0.0, 0.0); ctx.stroke_w(&sol(c.c("outline"), 1.0), 0.6);
            let hl = self.hp(&[(0.3, 0.08), (0.9, 0.06)]);
            ctx.begin(); ctx.mv(hl[0]); ctx.ln(hl[1]); ctx.stroke_w(&sol(WHITE, 1.0), 0.5);
            // mouth grill
            let g = self.hp(&[(0.55, 0.62), (0.9, 0.6)]);
            ctx.begin(); ctx.mv(g[0]); ctx.ln(g[1]); ctx.stroke_w(&sol(shade_of(c.c("metal")), 1.0), 0.6);
        }
        // hair strands
        if !self.helmet && self.ht != "none" && self.ht != "bald" {
            let s1 = self.hp(&[(-0.2, -0.75), (0.25, -0.9), (0.55, -0.7)]);
            ctx.begin(); ctx.mv(s1[0]); ctx.quad_to(s1[1].x, s1[1].y, s1[2].x, s1[2].y);
            ctx.stroke_w(&sol(light_of(c.c("hair")), 0.75), 0.9);
        }
    }
}

fn glow_orb(ctx: &mut Ctx, p: P, r: f64, col: Rgb, phase: f64) {
    ctx.save();
    ctx.st.blend = BlendMode::Plus;
    let g = Fill::Rad(p.x, p.y, 0.0, r * 1.8, vec![(0.0, rgba(WHITE, 0.95)), (0.25, rgba(light_of(col), 0.9)), (0.55, rgba(col, 0.55)), (1.0, rgba(col, 0.0))]);
    ctx.begin(); ctx.arc(p.x, p.y, r * 1.8, 0.0, PI * 2.0); ctx.fill(&g);
    ctx.st.lw = 0.6;
    for i in 0..3 {
        let a = phase + i as f64 * 2.1;
        ctx.begin(); ctx.arc(p.x, p.y, r * (0.8 + i as f64 * 0.25), a, a + 1.6); ctx.stroke(&sol(light_of(col), 0.8));
    }
    ctx.restore();
}

/// Gain applied to the batched outline glow so that the union-coverage shadow approximates the
/// per-stroke accumulated canvas shadows.
const AURA_GAIN: f32 = 1.6;

fn draw_layer(ctx: &mut Ctx, ly: &Layer, rig: &Rig) {
    let outline = rgba(rig.c.c("outline"), 1.0);
    if !ly.shapes.is_empty() {
        ctx.save();
        ctx.st.lw = 1.15 * 2.0;
        let sk = ctx.stroke_style();
        let geoms: Vec<Path> = ly.shapes.iter().filter(|s| !s.no_outline).filter_map(|s| { ctx.begin(); s.geo.emit(ctx, 0.0, 0.0); if std::env::var("NOE").is_ok() { return None; } ctx.path()?.stroke(&sk, 1.0) }).collect();
        if let Some(aura) = ly.aura {
            // canvas: shadowColor = aura, shadowBlur = 14 * RES on every outline stroke (batched here)
            let refs: Vec<&Path> = geoms.iter().collect();
            ctx.shadow_paths(&refs, 1.0, rgba(aura, 1.0), (14.0 * RES) as f32, AURA_GAIN);
        }
        if std::env::var("NOA").is_err() { for g in &geoms { ctx.fill_solid(g, outline); } }
        ctx.restore();
        if std::env::var("NOB").is_err() { for s in &ly.shapes { fill_shape(ctx, s); } }
        if std::env::var("NOD").is_err() { for s in ly.shapes.iter().filter(|s| s.line) { ctx.begin(); s.geo.emit(ctx, 0.0, 0.0); ctx.stroke_w(&Fill::Solid(outline), 0.45); } }
    }
    if std::env::var("NOC").is_err() { if let Some(a) = &ly.after { ctx.save(); rig.draw_after(ctx, a); ctx.restore(); } }
}

/// two-tone cel shading with a rim light. Canvas version: clip(shape); fill shade; fill the
/// light-offset copy in the light colour (rim); clip(light-offset copy); fill the rim-offset copy
/// with the base colour. Here the three coverages are rasterized as small masks and the colours
/// are resolved per pixel, then composited (source-over) onto the canvas.
fn fill_shape(ctx: &mut Ctx, s: &Shape) {
    ctx.begin();
    s.geo.emit(ctx, 0.0, 0.0);
    if s.flat { ctx.fill(&sol(s.fill, 1.0)); return; }
    let Some(path) = ctx.path() else { return };
    let Some((x0, y0, w, h)) = region(path.bounds(), 1.0, ctx.pm.width(), ctx.pm.height()) else { return };
    let ca = coverage(&[&path], x0, y0, w, h, 1.0, false);
    // offset copies of the shape = the coverage shifted by the (device-space) offset, resampled bilinearly
    let d = s.shade.unwrap_or(2.0);
    let rim = s.rim.unwrap_or(if lum(s.fill) < 0.2 { 0.55 } else { 0.0 });
    let light = light();
    // zero-padded copy of the coverage so shifted lookups need no bounds checks
    let m = ctx.st.m;
    let dev = |ox: f64, oy: f64| ((m.a * ox + m.c * oy) as f32, (m.b * ox + m.d * oy) as f32);
    let (sd, sr) = (dev(light.x * d.max(0.0), light.y * d.max(0.0)), dev(-light.x * rim.max(0.0), -light.y * rim.max(0.0)));
    let pad = (sd.0.abs().max(sd.1.abs()).max(sr.0.abs()).max(sr.1.abs()).ceil() as usize) + 2;
    let pw = w + 2 * pad;
    let padded: Vec<f32> = if d > 0.0 {
        let mut v = vec![0.0f32; pw * (h + 2 * pad)];
        for y in 0..h { v[(y + pad) * pw + pad..(y + pad) * pw + pad + w].copy_from_slice(&ca[y * w..(y + 1) * w]); }
        v
    } else { vec![] };
    let shifted = |(dx, dy): (f32, f32)| -> Vec<f32> {
        let (ix, iy) = (dx.floor(), dy.floor());
        let (fx, fy) = (dx - ix, dy - iy);
        let (ix, iy) = (ix as isize, iy as isize);
        let mut out = Vec::with_capacity(w * h);
        let off = (pad as isize - ix) as usize;
        for y in 0..h {
            let sy = (y as isize - iy + pad as isize) as usize;
            let r0 = &padded[sy * pw + off - 1..sy * pw + off + w];
            let r1 = &padded[(sy - 1) * pw + off - 1..(sy - 1) * pw + off + w];
            out.extend((0..w).map(|x| {
                let top = r0[x + 1] + (r0[x] - r0[x + 1]) * fx;
                let bot = r1[x + 1] + (r1[x] - r1[x + 1]) * fx;
                top + (bot - top) * fy
            }));
        }
        out
    };
    let (cb, cc) = if d > 0.0 { (Some(shifted(sd)), if rim > 0.0 { Some(shifted(sr)) } else { None }) } else { (None, None) };
    let f3 = |c: Rgb| [c[0] as f32, c[1] as f32, c[2] as f32];
    let (shade, lightc, base) = (f3(shade_of(s.fill)), f3(light_of(s.fill)), f3(s.fill));
    let has_rim = d > 0.0 && rim > 0.0;
    let clip = ctx.st.clip.clone();
    let stride = ctx.pm.width() as usize;
    let data = ctx.pm.data_mut();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut a = ca[i];
            if a <= 0.0 { continue; }
            let di = (y0 + y) * stride + x0 + x;
            if let Some(cm) = &clip { a *= cm.data()[di] as f32 * (1.0 / 255.0); if a <= 0.0 { continue; } }
            let col = if let Some(cb) = &cb {
                let bb = cb[i];
                let mut c = shade;
                if has_rim { for k in 0..3 { c[k] += (lightc[k] - c[k]) * bb; } }
                let t = bb * cc.as_ref().map_or(1.0, |cc| cc[i]);
                for k in 0..3 { c[k] += (base[k] - c[k]) * t; }
                c
            } else { base };
            blend_px(&mut data[di * 4..di * 4 + 4], [col[0] * a, col[1] * a, col[2] * a, a * 255.0], BlendMode::SourceOver);
        }
    }
}

/// straight-alpha RGBA image of a premultiplied pixmap (fast paths for empty / opaque pixels)
fn to_image(p: &Pixmap) -> Arc<ImageData> {
    let src = p.data();
    let mut out = vec![0u8; src.len()];
    for (d, s) in out.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
        match s[3] {
            0 => {}
            255 => d.copy_from_slice(s),
            a => { let a = a as u32; for i in 0..3 { d[i] = ((s[i] as u32 * 255 + a / 2) / a).min(255) as u8; } d[3] = a as u8; }
        }
    }
    ImageData::new(p.width(), p.height(), out)
}

fn to_box4(b: &B4) -> Box4 { [b[0] as f32, b[1] as f32, b[2] as f32, b[3] as f32] }

pub fn render_pose(style: &Value, pose: &Value, active: Option<&ActiveSpec>, with_images: bool, palette: Option<&Value>) -> RigRender {
    let c = Colors::new(gv(style, "colors"), palette);
    let sk = solve(style, pose);
    let wg = weapon_geo(style, pose, &sk);
    let (hurt, hit) = compute_boxes(&sk, active, wg.as_ref());
    let mut bb = BBox::new();
    let wind = gnum(pose, "wind", 0.35);
    let trail = if sk.rot.abs() > 50.0 && sk.rot.abs() < 130.0 { -92.0 } else { -(25.0 + wind * 60.0) - sk.rot * 0.3 };
    let hair = gv(style, "hair").cloned().unwrap_or_else(|| serde_json::json!({ "type": "short" }));
    let top = gstr(style, "top").unwrap_or("gi").to_string();
    let sleeves = gstr(style, "sleeves").map(str::to_string).unwrap_or_else(|| (if top == "tank" || top == "bare" { "none" } else if top == "robe" { "wide" } else { "long" }).to_string());
    let rig = Rig {
        style, pose, c, trail, phase: gnum(pose, "phase", 0.0), wind,
        ht: gstr(&hair, "type").unwrap_or("short").to_string(), helmet: gstr(&hair, "type") == Some("helmet"), hair,
        top, sleeves, pants: gstr(style, "pants").unwrap_or("gi").to_string(), sk,
    };
    let mut layers = rig.build_layers(&mut bb, wg.as_ref());
    for b in hurt.iter().chain(hit.iter()) { bb.add_p(pt(b[0], b[1]), 0.0); bb.add_p(pt(b[2], b[3]), 0.0); }
    let aura = if gt(pose, "aura") { Some(rig.c.c("aura")) } else { None };
    let pad = PAD + if aura.is_some() { 5.0 } else { 0.0 };
    let (x1, y1, x2, y2) = ((bb.x1 - pad).floor(), (bb.y1 - pad).floor(), (bb.x2 + pad).ceil(), (bb.y2.max(0.0) + pad).ceil());
    let (width, height) = ((x2 - x1) * RES, (y2 - y1) * RES);
    let (axis_x, axis_y) = (-x1 * RES, -y1 * RES);
    let mut image = None;
    if with_images {
        if let Some(mut ctx) = Ctx::new(width as u32, height as u32) {
            ctx.translate(axis_x, axis_y);
            ctx.scale(RES, RES);
            let sk = &rig.sk;
            if let Some(aura) = aura {
                // aura flame behind the body
                ctx.save();
                ctx.st.blend = BlendMode::Plus;
                let cc = lerp(sk.hip, sk.neck, 0.4);
                let g = Fill::Rad(cc.x, cc.y, 4.0, 62.0 * sk.s, vec![(0.0, rgba(aura, 0.5)), (0.6, rgba(aura, 0.18)), (1.0, rgba(aura, 0.0))]);
                ctx.begin();
                let ph = gnum(pose, "phase", 0.0);
                for i in 0..=24 {
                    let a = (i as f64 / 24.0) * PI * 2.0;
                    let rr = (34.0 + ((a * 5.0 + ph * 3.0).sin() * 0.5 + 0.5) * 16.0 + if a.sin() < 0.0 { 14.0 } else { 0.0 }) * sk.s;
                    let p = pt(cc.x + a.sin() * rr * 0.75, cc.y - a.cos() * rr * 1.05);
                    if i == 0 { ctx.mv(p) } else { ctx.ln(p) }
                }
                ctx.fill(&g);
                ctx.restore();
            }
            for ly in layers.iter_mut() {
                if aura.is_some() && ly.aura.is_none() && !ly.shapes.is_empty() { ly.aura = aura; }
                draw_layer(&mut ctx, ly, &rig);
            }
            if gt(pose, "flash") {
                let fc = match gv(pose, "flash") { Some(Value::Bool(true)) => WHITE, Some(v) => parse_rgb(v.as_str().unwrap_or("")), None => WHITE };
                ctx.st.blend = BlendMode::SourceAtop;
                ctx.fill_rect(x1, y1, x2 - x1, y2 - y1, &sol(fc, 0.55));
            }
            if std::env::var("NOF").is_err() { image = Some(to_image(&ctx.pm)); }
        }
    }
    RigRender { image, width: width as f32, height: height as f32, axis_x: axis_x as f32, axis_y: axis_y as f32, scale: (1.0 / RES) as f32, hurt: hurt.iter().map(to_box4).collect(), hit: hit.iter().map(to_box4).collect() }
}

// ------------------------------------------------------------------ effects
/// effect: { type, size?, length?, color?, color2?, phase?, angle? }
/// types: orb, slash, beam, fire, darkfire, ice, lightning, kunai, shuriken, water, wave, pillar, shockwave, burst, petal
/// The axis is the effect centre (beam: its origin on the left; pillar/shockwave: ground centre).
pub fn render_effect(effect: &Value, with_images: bool, palette: Option<&Value>) -> RigRender {
    let e = effect;
    let c = Colors::new(None, palette);
    let ty = gstr(e, "type").unwrap_or("orb");
    let size = gnum(e, "size", 14.0);
    let ph = gnum(e, "phase", 0.0);
    let col = match gstr(e, "color") { Some(s) => parse_rgb(s), None => c.get("energy").unwrap_or([0x7f, 0xd4, 0xff]) };
    let col2 = match gstr(e, "color2") { Some(s) => parse_rgb(s), None => light_of(col) };
    let s = size;
    let (body, draw): (B4, B4) = match ty {
        "beam" => { let l = gnum(e, "length", 200.0); ([0.0, -s, l, s], [-s * 1.5, -s * 2.2, l + s * 1.8, s * 2.2]) }
        "pillar" => { let h = gnum(e, "length", 90.0); ([-s, -h, s, 0.0], [-s * 2.0, -h - s, s * 2.0, s * 0.6]) }
        "shockwave" => ([-s, -s * 1.4, s * 1.1, 0.0], [-s * 2.4, -s * 2.2, s * 1.8, s * 0.5]),
        "slash" | "water" | "wave" => ([-s * 0.35, -s, s * 0.55, s], [-s * 1.6, -s * 1.4, s * 1.2, s * 1.4]),
        "kunai" | "shuriken" => ([-s * 0.6, -s * 0.45, s * 0.8, s * 0.45], [-s * 3.4, -s * 1.4, s * 1.6, s * 1.4]),
        "ice" => ([-s * 0.8, -s * 0.6, s * 1.1, s * 0.6], [-s * 2.6, -s * 1.6, s * 2.0, s * 1.6]),
        "burst" => ([-s * 0.6, -s * 0.6, s * 0.6, s * 0.6], [-s * 1.8, -s * 1.8, s * 1.8, s * 1.8]),
        _ => ([-s * 0.8, -s * 0.8, s * 0.9, s * 0.8], [-s * 3.2, -s * 1.9, s * 1.9, s * 1.9]),
    };
    let body = body.map(jround);
    let (x1, y1, x2, y2) = ((draw[0] - 4.0).floor(), (draw[1] - 4.0).floor(), (draw[2] + 4.0).ceil(), (draw[3] + 4.0).ceil());
    let (width, height, axis_x, axis_y) = ((x2 - x1) * RES, (y2 - y1) * RES, -x1 * RES, -y1 * RES);
    let nohit = gt(e, "nohit");
    let boxes = if nohit { vec![] } else { vec![to_box4(&body)] };
    let mut image = None;
    if with_images {
        if let Some(mut ctx) = Ctx::new(width as u32, height as u32) {
            ctx.translate(axis_x, axis_y);
            ctx.scale(RES, RES);
            draw_effect(&mut ctx, ty, size, ph, col, col2, e);
            image = Some(to_image(&ctx.pm));
        }
    }
    RigRender { image, width: width as f32, height: height as f32, axis_x: axis_x as f32, axis_y: axis_y as f32, scale: (1.0 / RES) as f32, hit: boxes.clone(), hurt: boxes }
}

fn radial(ctx: &mut Ctx, x: f64, y: f64, r: f64, st: &[(f32, Color)]) {
    ctx.begin();
    ctx.arc(x, y, r, 0.0, PI * 2.0);
    ctx.fill(&Fill::Rad(x, y, 0.0, r, st.to_vec()));
}
fn glow_stroke(ctx: &mut Ctx, col: Color, w: f64, blur: f64, f: impl FnOnce(&mut Ctx)) {
    ctx.save(); ctx.set_shadow(col, blur * RES); ctx.st.lw = w; ctx.begin(); f(ctx); ctx.stroke(&Fill::Solid(col)); ctx.restore();
}
fn glow_fill(ctx: &mut Ctx, col: Color, blur: f64, f: impl FnOnce(&mut Ctx)) {
    ctx.save(); ctx.set_shadow(col, blur * RES); ctx.begin(); f(ctx); ctx.fill(&Fill::Solid(col)); ctx.restore();
}

#[allow(clippy::too_many_arguments)]
fn draw_effect(ctx: &mut Ctx, ty: &str, s: f64, ph: f64, col: Rgb, col2: Rgb, e: &Value) {
    let w = WHITE;
    ctx.st.blend = BlendMode::Plus;
    match ty {
        "slash" | "water" | "wave" => {
            let water = ty != "slash";
            let h = s;
            let crescent = |ctx: &mut Ctx, sc: f64| { ctx.move_to(0.0, -h * sc); ctx.quad_to(h * 0.95 * sc, 0.0, 0.0, h * sc); ctx.quad_to(h * 0.35 * sc, 0.0, 0.0, -h * sc); ctx.close(); };
            // after images
            for i in (1..=3).rev() {
                let fi = i as f64;
                ctx.save(); ctx.translate(-fi * s * 0.28, 0.0); ctx.st.alpha = (0.18 * (4.0 - fi)) as f32;
                glow_fill(ctx, rgba(col, 1.0), 4.0, |c| crescent(c, 1.0 - fi * 0.07));
                ctx.restore();
            }
            glow_fill(ctx, rgba(col, 0.95), 10.0, |c| crescent(c, 1.05));
            ctx.save(); ctx.translate(h * 0.06, 0.0); glow_fill(ctx, rgba(col2, 1.0), 3.0, |c| crescent(c, 0.85)); ctx.restore();
            ctx.save(); ctx.translate(h * 0.12, 0.0); ctx.begin(); crescent(ctx, 0.62); ctx.fill(&sol(w, 1.0)); ctx.restore();
            if water {
                for i in 0..9 {
                    let fi = i as f64;
                    let y = (hash(fi + 1.0) * 2.0 - 1.0) * h * 0.9;
                    let x = -s * (0.2 + hash(fi + 5.0 + ph) * 1.1);
                    radial(ctx, x, y, 1.2 + hash(fi * 3.0) * 2.2, &[(0.0, rgba(w, 0.95)), (1.0, rgba(col, 0.0))]);
                }
                glow_stroke(ctx, rgba(w, 0.8), 0.8, 3.0, |c| { c.move_to(-s * 0.6, -h * 0.6); c.quad_to(-s * 0.2 + ph.sin() * 3.0, 0.0, -s * 0.6, h * 0.6); });
            } else {
                for i in 0..6 {
                    let fi = i as f64;
                    let y = (fi / 5.0 - 0.5) * h * 1.6;
                    glow_stroke(ctx, rgba(col2, 0.6), 0.7, 2.0, |c| { c.move_to(-s * (0.3 + hash(fi + ph) * 0.4), y); c.line_to(-s * (1.0 + hash(fi * 7.0 + ph) * 0.6), y * 0.9); });
                }
            }
        }
        "beam" => {
            let l = gnum(e, "length", 200.0);
            let wob = |x: f64, k: f64| (x * 0.12 + ph * 3.0 + k).sin() * s * 0.12;
            let band = |ctx: &mut Ctx, th: f64, k: f64| {
                ctx.move_to(0.0, -th);
                let mut x = 0.0;
                while x <= l { ctx.line_to(x, -th - wob(x, k)); x += 8.0; }
                ctx.arc(l, 0.0, th, -PI / 2.0, PI / 2.0);
                let mut x = l;
                while x >= 0.0 { ctx.line_to(x, th + wob(x, k + 1.0)); x -= 8.0; }
                ctx.close();
            };
            glow_fill(ctx, rgba(col, 0.55), 14.0, |c| band(c, s * 1.25, 0.0));
            glow_fill(ctx, rgba(col, 0.9), 6.0, |c| band(c, s * 0.85, 2.0));
            ctx.begin(); band(ctx, s * 0.55, 4.0); ctx.fill(&sol(col2, 1.0));
            ctx.begin(); band(ctx, s * 0.28, 6.0); ctx.fill(&sol(w, 1.0));
            // origin flare and head
            radial(ctx, 0.0, 0.0, s * 2.1, &[(0.0, rgba(w, 1.0)), (0.35, rgba(col2, 0.8)), (1.0, rgba(col, 0.0))]);
            radial(ctx, l, 0.0, s * 1.8, &[(0.0, rgba(w, 1.0)), (0.4, rgba(col2, 0.7)), (1.0, rgba(col, 0.0))]);
            // energy streaks
            for i in 0..10 {
                let fi = i as f64;
                let x = (hash(fi + 2.0) * l + ph * 40.0) % l;
                let y = (hash(fi + 9.0) * 2.0 - 1.0) * s * 0.9;
                glow_stroke(ctx, rgba(w, 0.7), 0.7, 2.0, |c| { c.move_to(x, y); c.line_to(x + 14.0 + hash(fi) * 20.0, y); });
            }
        }
        "ice" => {
            let shard = |ctx: &mut Ctx, x: f64, y: f64, l: f64, wd: f64, a: f64| {
                ctx.save(); ctx.translate(x, y); ctx.rotate(a);
                glow_fill(ctx, rgba(col, 0.75), 6.0, |c| { c.move_to(l, 0.0); c.line_to(0.0, -wd); c.line_to(-l * 0.5, 0.0); c.line_to(0.0, wd); c.close(); });
                ctx.begin(); ctx.move_to(l, 0.0); ctx.line_to(0.0, -wd); ctx.line_to(-l * 0.1, 0.0); ctx.close(); ctx.fill(&sol(w, 0.85));
                ctx.begin(); ctx.move_to(l, 0.0); ctx.line_to(0.0, -wd); ctx.line_to(-l * 0.5, 0.0); ctx.line_to(0.0, wd); ctx.close(); ctx.stroke_w(&sol(w, 0.9), 0.5);
                ctx.restore();
            };
            for i in 0..5 { let t = i as f64 / 5.0; radial(ctx, -s * (0.8 + t * 1.6), (i as f64 * 2.0 + ph).sin() * s * 0.3, s * (0.5 - t * 0.3), &[(0.0, rgba(col, 0.5)), (1.0, rgba(col, 0.0))]); }
            radial(ctx, 0.0, 0.0, s * 1.5, &[(0.0, rgba(col, 0.6)), (1.0, rgba(col, 0.0))]);
            shard(ctx, -s * 0.3, -s * 0.45, s * 0.9, s * 0.25, -0.25 + ph.sin() * 0.05);
            shard(ctx, -s * 0.3, s * 0.45, s * 0.9, s * 0.25, 0.25);
            shard(ctx, 0.0, 0.0, s * 1.15, s * 0.4, 0.0);
            for i in 0..6 { let fi = i as f64; radial(ctx, -s * (0.6 + hash(fi + ph * 3.0) * 1.6), (hash(fi * 5.0) * 2.0 - 1.0) * s * 0.8, 1.4, &[(0.0, rgba(w, 1.0)), (1.0, rgba(col, 0.0))]); }
        }
        "lightning" => {
            radial(ctx, 0.0, 0.0, s * 1.8, &[(0.0, rgba(col, 0.7)), (0.5, rgba(col, 0.25)), (1.0, rgba(col, 0.0))]);
            radial(ctx, 0.0, 0.0, s * 0.8, &[(0.0, rgba(w, 1.0)), (0.6, rgba(col2, 0.8)), (1.0, rgba(col, 0.0))]);
            for k in 0..6 {
                let fk = k as f64;
                let a0 = (fk / 6.0) * PI * 2.0 + ph * 1.7;
                glow_stroke(ctx, rgba(col2, 1.0), 0.9, 5.0, |c| {
                    let (mut x, mut y) = (0.0, 0.0);
                    c.move_to(0.0, 0.0);
                    for i in 1..=5 {
                        let a = a0 + (hash(fk * 10.0 + i as f64 + (ph * 7.0).floor()) - 0.5) * 1.3;
                        x += a.cos() * s * 0.33; y += a.sin() * s * 0.33;
                        c.line_to(x, y);
                    }
                });
            }
            for i in 0..4 {
                let fi = i as f64;
                glow_stroke(ctx, rgba(w, 0.9), 0.6, 2.0, |c| {
                    let (mut x, mut y) = (-s * 0.3, (fi - 1.5) * s * 0.3);
                    c.move_to(x, y);
                    for j in 0..5 { x -= s * 0.45; y += (hash(fi * 9.0 + j as f64 + ph * 5.0) - 0.5) * s * 0.5; c.line_to(x, y); }
                });
            }
        }
        "kunai" | "shuriken" => {
            // motion streaks
            for i in 0..3 {
                let fi = i as f64;
                glow_stroke(ctx, rgba(col, 0.6 - fi * 0.15), 1.2 - fi * 0.3, 3.0, |c| { c.move_to(-s * 0.4, (fi - 1.0) * s * 0.25); c.line_to(-s * (2.6 - fi * 0.4), (fi - 1.0) * s * 0.3); });
            }
            ctx.st.blend = BlendMode::SourceOver;
            let dark = [0x14, 0x10, 0x18];
            if ty == "kunai" {
                ctx.save(); ctx.set_shadow(rgba(col, 1.0), 5.0 * RES);
                ctx.st.lw = 0.6;
                ctx.begin(); ctx.move_to(s * 0.9, 0.0); ctx.line_to(0.0, -s * 0.28); ctx.line_to(-s * 0.1, 0.0); ctx.line_to(0.0, s * 0.28); ctx.close();
                ctx.fill(&sol([0xdf, 0xe6, 0xf0], 1.0)); ctx.stroke(&sol(dark, 1.0));
                ctx.restore();
                ctx.begin(); ctx.move_to(s * 0.9, 0.0); ctx.line_to(0.0, s * 0.28); ctx.line_to(-s * 0.1, 0.0); ctx.close(); ctx.fill(&sol([0x7a, 0x84, 0x96], 1.0));
                ctx.fill_rect(-s * 0.55, -s * 0.08, s * 0.45, s * 0.16, &sol([0x2a, 0x22, 0x30], 1.0));
                ctx.begin(); ctx.arc(-s * 0.7, 0.0, s * 0.14, 0.0, PI * 2.0); ctx.stroke_w(&sol([0x2a, 0x22, 0x30], 1.0), 0.8);
            } else {
                ctx.save(); ctx.rotate(ph * 4.0); ctx.set_shadow(rgba(col, 1.0), 5.0 * RES);
                ctx.st.lw = 0.6;
                ctx.begin();
                for i in 0..8 { let a = (i as f64 / 8.0) * PI * 2.0; let rr = if i % 2 == 1 { s * 0.18 } else { s * 0.6 }; ctx.line_to(a.cos() * rr, a.sin() * rr); }
                ctx.close(); ctx.fill(&sol([0xc9, 0xd2, 0xde], 1.0)); ctx.stroke(&sol(dark, 1.0)); ctx.restore();
                ctx.begin(); ctx.arc(0.0, 0.0, s * 0.08, 0.0, PI * 2.0); ctx.fill(&sol([0x2a, 0x22, 0x30], 1.0));
            }
        }
        "pillar" => {
            let h = gnum(e, "length", 90.0);
            glow_fill(ctx, rgba(col, 0.55), 12.0, |c| {
                c.move_to(-s * 1.3, 0.0);
                let mut y = 0.0;
                while y >= -h { c.line_to(-s * (1.1 - (-y / h) * 0.4) + (y * 0.2 + ph * 4.0).sin() * 2.0, y); y -= 6.0; }
                c.line_to(0.0, -h - s * 0.8);
                let mut y = -h;
                while y <= 0.0 { c.line_to(s * (1.1 - (-y / h) * 0.4) + (y * 0.25 + ph * 4.0 + 1.0).sin() * 2.0, y); y += 6.0; }
                c.close();
            });
            ctx.begin(); ctx.move_to(-s * 0.6, 0.0); ctx.quad_to(-s * 0.3, -h * 0.6, 0.0, -h); ctx.quad_to(s * 0.3, -h * 0.6, s * 0.6, 0.0); ctx.fill(&sol(col2, 0.95));
            ctx.begin(); ctx.move_to(-s * 0.25, 0.0); ctx.quad_to(0.0, -h * 0.5, 0.0, -h * 0.8); ctx.quad_to(0.0, -h * 0.5, s * 0.25, 0.0); ctx.fill(&sol(w, 0.9));
            radial(ctx, 0.0, 0.0, s * 2.0, &[(0.0, rgba(w, 0.9)), (0.4, rgba(col, 0.6)), (1.0, rgba(col, 0.0))]);
        }
        "shockwave" => {
            for i in 0..3 {
                let fi = i as f64;
                let x = -fi * s * 0.6;
                glow_fill(ctx, rgba(col, 0.6 - fi * 0.15), 8.0, |c| { c.move_to(x - s, 0.0); c.quad_to(x - s * 0.2, -s * (1.6 - fi * 0.3) - (ph * 3.0).sin() * 3.0, x + s, 0.0); c.close(); });
            }
            ctx.st.blend = BlendMode::SourceOver;
            for i in 0..5 {
                let fi = i as f64;
                let x = (hash(fi + 4.0) - 0.6) * s * 2.0;
                let y = -hash(fi + 8.0 + ph) * s * 1.4;
                let rr = 1.5 + hash(fi) * 2.5;
                ctx.begin(); ctx.move_to(x - rr, y); ctx.line_to(x, y - rr); ctx.line_to(x + rr, y + rr * 0.3); ctx.line_to(x - rr * 0.2, y + rr); ctx.close();
                ctx.fill(&sol([0x6a, 0x58, 0x48], 1.0)); ctx.stroke_w(&sol([0x1a, 0x12, 0x10], 1.0), 0.5);
            }
        }
        "burst" => {
            let t = ph.clamp(0.0, 1.0);
            let rr = s * (0.6 + t * 1.1);
            radial(ctx, 0.0, 0.0, rr, &[(0.0, rgba(w, 1.0 - t * 0.7)), (0.4, rgba(col2, 0.8 * (1.0 - t))), (1.0, rgba(col, 0.0))]);
            glow_stroke(ctx, rgba(col2, 1.0 - t), 1.6 * (1.0 - t) + 0.3, 6.0, |c| c.arc(0.0, 0.0, rr * 0.85, 0.0, PI * 2.0));
            for i in 0..8 {
                let a = (i as f64 / 8.0) * PI * 2.0 + 0.3;
                glow_stroke(ctx, rgba(w, 1.0 - t), 0.9, 3.0, |c| { c.move_to(a.cos() * rr * 0.5, a.sin() * rr * 0.5); c.line_to(a.cos() * rr * 1.3, a.sin() * rr * 1.3); });
            }
        }
        "orb" | "fire" | "darkfire" => {
            let fire = ty != "orb";
            // trail
            for i in 0..6 {
                let t = i as f64 / 6.0;
                let x = -s * (0.6 + t * 2.2);
                let y = (ph * 2.0 + i as f64 * 1.3).sin() * s * 0.18 * t;
                radial(ctx, x, y, s * (0.85 - t * 0.55), &[(0.0, rgba(col, 0.55 * (1.0 - t))), (1.0, rgba(col, 0.0))]);
            }
            if fire {
                for i in 0..9 {
                    let a = (i as f64 / 9.0) * PI * 2.0 + ph;
                    let fl = s * (1.0 + 0.5 * hash(i as f64 + ph * 10.0));
                    ctx.save();
                    ctx.translate(a.cos() * s * 0.3, a.sin() * s * 0.3);
                    glow_fill(ctx, rgba(col, 0.75), 6.0, |c| {
                        c.move_to(0.0, -s * 0.45);
                        c.quad_to(-fl * 1.3, -s * 0.5 + a.sin() * 4.0, -fl * 1.9, (a * 2.0).sin() * s * 0.4);
                        c.quad_to(-fl * 1.1, s * 0.5, 0.0, s * 0.45);
                        c.close();
                    });
                    ctx.restore();
                }
            }
            radial(ctx, 0.0, 0.0, s * 1.9, &[(0.0, rgba(col, 0.8)), (0.45, rgba(col, 0.35)), (1.0, rgba(col, 0.0))]);
            radial(ctx, 0.0, 0.0, s * 0.95, &[(0.0, rgba(w, 1.0)), (0.5, rgba(col2, 0.95)), (1.0, rgba(col, 0.2))]);
            // swirl
            for i in 0..3 {
                let a = ph * 2.0 + i as f64 * 2.1;
                let rr = s * (0.75 + i as f64 * 0.22);
                glow_stroke(ctx, rgba(col2, 0.9), 1.1, 4.0, |c| c.arc(0.0, 0.0, rr, a, a + 2.2));
            }
        }
        _ => {
            // lotus / petal burst: trailing petals, a ring of petals and a hot core
            for i in 0..7 {
                let fi = i as f64;
                let x = -s * (0.8 + hash(fi + 1.0) * 1.8);
                let y = (hash(fi + 7.0) * 2.0 - 1.0) * s * 0.8;
                let a = ph * 3.0 + fi;
                ctx.save(); ctx.translate(x, y); ctx.rotate(a);
                glow_fill(ctx, rgba(col, 0.7 - fi * 0.07), 3.0, |c| { c.move_to(-s * 0.2, 0.0); c.quad_to(0.0, -s * 0.16, s * 0.2, 0.0); c.quad_to(0.0, s * 0.16, -s * 0.2, 0.0); });
                ctx.restore();
            }
            radial(ctx, 0.0, 0.0, s * 1.5, &[(0.0, rgba(col, 0.55)), (1.0, rgba(col, 0.0))]);
            for i in 0..6 {
                let a = (i as f64 / 6.0) * PI * 2.0 + ph * 1.5;
                ctx.save(); ctx.rotate(a);
                glow_fill(ctx, rgba(col, 0.9), 5.0, |c| { c.move_to(s * 0.15, 0.0); c.quad_to(s * 0.55, -s * 0.38, s * 1.05, 0.0); c.quad_to(s * 0.55, s * 0.38, s * 0.15, 0.0); });
                ctx.begin(); ctx.move_to(s * 0.2, 0.0); ctx.quad_to(s * 0.5, -s * 0.16, s * 0.85, 0.0); ctx.quad_to(s * 0.5, s * 0.16, s * 0.2, 0.0); ctx.fill(&sol(col2, 0.9));
                ctx.restore();
            }
            radial(ctx, 0.0, 0.0, s * 0.55, &[(0.0, rgba(w, 1.0)), (0.6, rgba(col2, 0.9)), (1.0, rgba(col, 0.0))]);
        }
    }
}
