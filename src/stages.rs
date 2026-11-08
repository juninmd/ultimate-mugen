//! Original procedural stages + MUGEN stage adapter.
//!
//! Every built-in stage is a stack of parallax layers rasterized once with tiny-skia plus procedural noise passes (`texgen`):
//! sky/cloud noise, brick, wood grain, stone, sand, water... Layers flagged `dynamic` are empty placeholders that only carry a
//! parallax transform: the animated parts (flags, foliage, crowd, lava, neon flicker...) are drawn per frame by `stagefx`.
use crate::audio::MusicLoop;
use crate::texgen::{blobs, grain, hash, haze, n1, shade_v};
use std::sync::Arc;
use ume_engine::chardef::ImageData;
use ume_engine::matchs::StageInfo;
use ume_engine::raster::*;
use ume_formats::stage::MugenStage;

/// One parallax layer. `front` layers are drawn after the fighters; `dynamic` layers have no bitmap (animated by `stagefx`).
pub struct Layer {
    pub image: Arc<ImageData>,
    pub w: f32,
    pub h: f32,
    pub y: f32,
    pub delta: f32,
    pub dy: f32,
    pub front: bool,
    pub dynamic: bool,
    /** empty rows (units) cropped off the top of the bitmap: draw at `y + trim` */
    pub trim: f32,
}
#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub s: f32,
    pub a: f32,
    pub c: [u8; 3],
}

pub struct MugenBg {
    pub stage: Arc<MugenStage>,
    pub images: std::collections::HashMap<(i32, i32), Arc<ImageData>>,
    pub scl: f32,
    pub scl_y: f32,
}

/// Which built-in look a stage has (drives the animated / reactive layer in `stagefx`); `Custom` = MUGEN stages (untouched).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StageKind {
    Dojo,
    Neon,
    Bamboo,
    Plains,
    Spirit,
    Volcano,
    #[default]
    Custom,
}

pub enum StageArt {
    Procedural {
        back: Vec<Layer>,
        particles: Vec<Particle>,
        tint: Option<[f32; 4]>,
    },
    Illustrated {
        image: Arc<ImageData>,
        ambience: &'static ambience::Profile,
    },
    Mugen(Box<MugenBg>),
}
pub struct Stage {
    pub info: StageInfo,
    pub art: StageArt,
    pub music: u32,
    pub kind: StageKind,
    pub bgm: Option<Arc<(Vec<u8>, MusicLoop)>>,
}

pub struct Rng(u32);
impl Rng {
    pub fn f(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0 as f32 / 4294967296.0
    }
}

type Painter<'a> = &'a dyn Fn(&mut Pixmap, Transform, f32, f32, &mut Rng);
/// `cap` limits the pixels-per-unit of soft / distant layers (memory at 4K); `front` layers draw over the fighters.
#[allow(clippy::too_many_arguments)]
fn lay(
    cap: f32,
    front: bool,
    w: f32,
    h: f32,
    y: f32,
    delta: f32,
    seed: u32,
    dy: f32,
    paint: Painter,
) -> Layer {
    // pixels per game unit follows the art-quality setting (4K → 2x), capped so no layer exceeds 8192 px (GPU texture limit)
    let r = ume_engine::native::rig::art_res()
        .min(cap)
        .min(8192.0 / w.max(h).max(1.0));
    let mut pm = Pixmap::new(((w * r) as u32).max(1), ((h * r) as u32).max(1)).unwrap();
    let mut rng = Rng(seed);
    paint(&mut pm, Transform::from_scale(r, r), w, h, &mut rng);
    // crop the transparent rows at the top: skipping them saves fill-rate every frame (far silhouette layers are half empty)
    let (pw, ph) = (pm.width() as usize, pm.height() as usize);
    let first = pm
        .data()
        .chunks(pw * 4)
        .position(|row| row.chunks(4).any(|px| px[3] > 3))
        .unwrap_or(ph.saturating_sub(1));
    let first = first.saturating_sub(2);
    if first > 4 && !front {
        let cropped = pm.data()[first * pw * 4..].to_vec();
        if let Some(sz) = tiny_skia::IntSize::from_wh(pw as u32, (ph - first) as u32) {
            if let Some(c) = Pixmap::from_vec(cropped, sz) {
                return Layer {
                    image: pixmap_to_image(&c),
                    w,
                    h,
                    y,
                    delta,
                    dy,
                    front,
                    dynamic: false,
                    trim: first as f32 / r,
                };
            }
        }
    }
    Layer {
        image: pixmap_to_image(&pm),
        w,
        h,
        y,
        delta,
        dy,
        front,
        dynamic: false,
        trim: 0.0,
    }
}
fn layer(w: f32, h: f32, y: f32, delta: f32, seed: u32, dy: f32, paint: Painter) -> Layer {
    lay(99.0, false, w, h, y, delta, seed, dy, paint)
}
/// Transparent placeholder: carries only the parallax transform of animated content drawn by `stagefx`.
fn dynamic(w: f32, h: f32, y: f32, delta: f32, dy: f32, front: bool) -> Layer {
    Layer {
        image: ImageData::new(1, 1, vec![0, 0, 0, 0]),
        w,
        h,
        y,
        delta,
        dy,
        front,
        dynamic: true,
        trim: 0.0,
    }
}

fn lin(y0: f32, y1: f32, stops: &[(f32, &str)]) -> Paint<'static> {
    let st = stops
        .iter()
        .map(|(o, c)| GradientStop::new(*o, rgba(c, 1.0)))
        .collect();
    let mut p = Paint::default();
    p.shader = LinearGradient::new(
        Point::from_xy(0.0, y0),
        Point::from_xy(0.0, y1),
        st,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    p.anti_alias = true;
    p
}
fn lin_a(y0: f32, y1: f32, stops: &[(f32, &str, f32)]) -> Paint<'static> {
    let st = stops
        .iter()
        .map(|(o, c, a)| GradientStop::new(*o, rgba(c, *a)))
        .collect();
    let mut p = Paint::default();
    p.shader = LinearGradient::new(
        Point::from_xy(0.0, y0),
        Point::from_xy(0.0, y1),
        st,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    p.anti_alias = true;
    p
}
fn lin_h(x0: f32, x1: f32, stops: &[(f32, &str, f32)]) -> Paint<'static> {
    let st = stops
        .iter()
        .map(|(o, c, a)| GradientStop::new(*o, rgba(c, *a)))
        .collect();
    let mut p = Paint::default();
    p.shader = LinearGradient::new(
        Point::from_xy(x0, 0.0),
        Point::from_xy(x1, 0.0),
        st,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    p.anti_alias = true;
    p
}
fn rect(pm: &mut Pixmap, t: Transform, x: f32, y: f32, w: f32, h: f32, p: &Paint) {
    if let Some(r) = tiny_skia::Rect::from_xywh(x, y, w.max(0.01), h.max(0.01)) {
        pm.fill_rect(r, p, t, None);
    }
}
fn fill(pm: &mut Pixmap, t: Transform, path: Option<tiny_skia::Path>, p: &Paint) {
    if let Some(pa) = path {
        pm.fill_path(&pa, p, FillRule::Winding, t, None);
    }
}
fn stroke(pm: &mut Pixmap, t: Transform, path: Option<tiny_skia::Path>, p: &Paint, w: f32) {
    if let Some(pa) = path {
        pm.stroke_path(
            &pa,
            p,
            &Stroke {
                width: w,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            },
            t,
            None,
        );
    }
}
fn ellipse(x: f32, y: f32, rx: f32, ry: f32) -> Option<tiny_skia::Path> {
    tiny_skia::Rect::from_xywh(x - rx, y - ry, rx * 2.0, ry * 2.0).and_then(PathBuilder::from_oval)
}
fn line(x0: f32, y0: f32, x1: f32, y1: f32) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    b.move_to(x0, y0);
    b.line_to(x1, y1);
    b.finish()
}
fn poly(pts: &[(f32, f32)]) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    for (i, p) in pts.iter().enumerate() {
        if i == 0 {
            b.move_to(p.0, p.1)
        } else {
            b.line_to(p.0, p.1)
        }
    }
    b.close();
    b.finish()
}
fn radial(cx: f32, cy: f32, r: f32, stops: &[(f32, &str, f32)]) -> Paint<'static> {
    let st = stops
        .iter()
        .map(|(o, c, a)| GradientStop::new(*o, rgba(c, *a)))
        .collect();
    let mut p = Paint::default();
    p.shader = RadialGradient::new(
        Point::from_xy(cx, cy),
        Point::from_xy(cx, cy),
        r,
        st,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    p.anti_alias = true;
    p
}
fn glowc(pm: &mut Pixmap, t: Transform, x: f32, y: f32, r: f32, col: &str, a: f32) {
    fill(
        pm,
        t,
        circle(x, y, r),
        &radial(
            x,
            y,
            r,
            &[(0.0, col, a), (0.4, col, a * 0.45), (1.0, col, 0.0)],
        ),
    );
}
/// Noise-driven ridge line filled with a vertical gradient (hills, mesas, distant mountains).
#[allow(clippy::too_many_arguments)]
fn ridge(
    pm: &mut Pixmap,
    t: Transform,
    w: f32,
    base: f32,
    amp: f32,
    top: &str,
    bot: &str,
    seed: f32,
    freq: f32,
) {
    let mut b = PathBuilder::new();
    b.move_to(0.0, base + 260.0);
    let mut x = 0.0;
    while x <= w + 3.0 {
        let n = n1(x * freq, seed);
        b.line_to(x, base - amp * (n * n * (3.0 - 2.0 * n)).powf(1.3));
        x += 2.5;
    }
    b.line_to(w + 3.0, base + 260.0);
    b.close();
    fill(
        pm,
        t,
        b.finish(),
        &lin(base - amp, base + 50.0, &[(0.0, top), (1.0, bot)]),
    );
}
fn parts(seed: u32, n: usize, f: impl Fn(&mut Rng) -> Particle) -> Vec<Particle> {
    let mut r = Rng(seed);
    (0..n).map(|_| f(&mut r)).collect()
}
fn info(name: &str) -> StageInfo {
    StageInfo {
        start_x: 0.0,
        start_y: 0.0,
        zoom_out: 1.0,
        name: name.into(),
        bound_left: -230.0,
        bound_right: 230.0,
        bound_high: -80.0,
        bound_low: 0.0,
        p1x: -70.0,
        p2x: 70.0,
        zoffset: 205.0,
        vertical_follow: 0.25,
        tension: 60.0,
        ..StageInfo::default()
    }
}

/// Deterministic skyline: (x, width, height) of each tower, shared by the baked art and the animated blink lights.
pub fn skyline(
    seed: u32,
    total: f32,
    wmin: f32,
    wmax: f32,
    hmin: f32,
    hmax: f32,
    gap: f32,
) -> Vec<(f32, f32, f32)> {
    let (mut x, mut i, mut v) = (0.0, 0u32, vec![]);
    while x < total {
        let w = wmin + hash(i, seed) * (wmax - wmin);
        let h = hmin + hash(i, seed + 1) * (hmax - hmin);
        v.push((x, w, h));
        x += w + hash(i, seed + 2) * gap;
        i += 1;
    }
    v
}
/// Neon signs on the mid city layer: (x, y, w, h, colour index). Baked glow + animated flicker use the same list.
pub fn neon_signs() -> Vec<(f32, f32, f32, f32, usize)> {
    skyline(41, 1000.0, 28.0, 56.0, 90.0, 190.0, 54.0)
        .iter()
        .enumerate()
        .filter(|(i, _)| hash(*i as u32, 7) < 0.85)
        .map(|(i, (x, w, h))| {
            let sw = 9.0 + hash(i as u32, 8) * 7.0;
            (
                x + w / 2.0 - sw / 2.0,
                205.0 - h + 10.0 + hash(i as u32, 9) * 40.0,
                sw,
                24.0 + hash(i as u32, 10) * 34.0,
                (hash(i as u32, 11) * 5.0) as usize % 5,
            )
        })
        .collect()
}
pub const NEON: [&str; 5] = ["#ff2fd0", "#22e8ff", "#ffe23a", "#7d5bff", "#3dff8a"];
/// Floor glow cracks (obsidian): polylines in the floor layer's coordinates, shared by the baked seams and the animated pulse.
pub fn lava_cracks(w: f32) -> Vec<Vec<(f32, f32)>> {
    (0..22u32)
        .map(|i| {
            let (mut x, mut y) = (hash(i, 61) * w, 36.0 + hash(i, 62) * 76.0);
            let mut v = vec![(x, y)];
            for k in 0..5u32 {
                x += (hash(i, 70 + k) - 0.3) * 24.0;
                y = (y + (hash(i, 80 + k) - 0.5) * 14.0).clamp(33.0, 118.0);
                v.push((x, y));
            }
            v
        })
        .collect()
}
/// Stone-lantern x positions on the spirit stage's lantern layer.
pub fn toro_xs() -> Vec<f32> {
    let (mut x, mut v, mut i) = (40.0, vec![], 0u32);
    while x < 1000.0 {
        v.push(x);
        x += 80.0 + hash(i, 3) * 36.0;
        i += 1;
    }
    v
}
/// Rail lantern x positions on the dojo floor layer.
pub fn dojo_lanterns() -> Vec<f32> {
    (0..16).map(|k| 10.0 + 70.0 * k as f32 + 2.5).collect()
}

// ───────────────────────────────────────────── DOJO ─────────────────────────────────────────────

fn pagoda(
    pm: &mut Pixmap,
    t: Transform,
    x: f32,
    by: f32,
    bw: f32,
    tiers: i32,
    col: &str,
    win: f32,
) {
    let dark = solid(rgba(col, 1.0));
    rect(pm, t, x - bw * 0.35, by - 30.0, bw * 0.7, 30.0, &dark);
    for k in 0..tiers {
        let ty = by - 30.0 - k as f32 * 22.0;
        let tw = bw * (1.0 - k as f32 * 0.2);
        let mut b = PathBuilder::new();
        b.move_to(x - tw / 2.0 - 11.0, ty + 7.0);
        b.quad_to(x - tw / 4.0, ty - 2.0, x, ty - 13.0);
        b.quad_to(x + tw / 4.0, ty - 2.0, x + tw / 2.0 + 11.0, ty + 7.0);
        b.quad_to(x + tw / 2.0, ty + 3.0, x + tw / 2.0 - 4.0, ty + 3.0);
        b.line_to(x - tw / 2.0 + 4.0, ty + 3.0);
        b.quad_to(x - tw / 2.0, ty + 3.0, x - tw / 2.0 - 11.0, ty + 7.0);
        b.close();
        fill(pm, t, b.finish(), &dark);
        rect(pm, t, x - tw * 0.3, ty + 2.0, tw * 0.6, 18.0, &dark);
        if win > 0.0 {
            for q in 0..3 {
                rect(
                    pm,
                    t,
                    x - tw * 0.22 + q as f32 * tw * 0.18,
                    ty + 7.0,
                    3.0,
                    6.0,
                    &solid(rgba("#ffbe5a", win)),
                );
            }
        }
    }
    let top = by - 30.0 - tiers as f32 * 22.0;
    stroke(pm, t, line(x, top - 6.0, x, top - 26.0), &dark, 1.4);
    for q in 0..3 {
        fill(
            pm,
            t,
            ellipse(x, top - 10.0 - q as f32 * 5.0, 3.0, 1.0),
            &dark,
        );
    }
}
fn pine(pm: &mut Pixmap, t: Transform, x: f32, by: f32, h: f32, col: &str) {
    let p = solid(rgba(col, 1.0));
    rect(pm, t, x - 1.0, by - h * 0.25, 2.0, h * 0.25, &p);
    for k in 0..5 {
        let f = k as f32 / 5.0;
        let y = by - h * 0.18 - f * h * 0.8;
        let w = h * 0.28 * (1.0 - f * 0.8);
        fill(
            pm,
            t,
            poly(&[
                (x - w, y + h * 0.16),
                (x, y - h * 0.1),
                (x + w, y + h * 0.16),
            ]),
            &p,
        );
    }
}
fn sakura(pm: &mut Pixmap, t: Transform, r: &mut Rng, x: f32, by: f32, h: f32) {
    let bark = solid(rgba("#241218", 1.0));
    fill(
        pm,
        t,
        poly(&[
            (x - 7.0, by),
            (x - 4.0, by - h * 0.4),
            (x - 2.0, by - h * 0.62),
            (x + 3.0, by - h * 0.62),
            (x + 5.0, by - h * 0.4),
            (x + 9.0, by),
        ]),
        &bark,
    );
    let mut tips = vec![];
    for k in 0..7 {
        let a = -1.57 + (k as f32 - 3.0) * 0.42 + (r.f() - 0.5) * 0.3;
        let l = h * (0.35 + r.f() * 0.25);
        let (sx, sy) = (x + 0.5, by - h * (0.45 + r.f() * 0.2));
        let (ex, ey) = (sx + a.cos() * l, sy + a.sin() * l);
        let mut b = PathBuilder::new();
        b.move_to(sx, sy);
        b.quad_to(
            sx + a.cos() * l * 0.4 + (r.f() - 0.5) * 10.0,
            sy + a.sin() * l * 0.6,
            ex,
            ey,
        );
        stroke(pm, t, b.finish(), &bark, 3.4 - k as f32 * 0.2);
        tips.push((ex, ey));
    }
    stroke(
        pm,
        t,
        line(x - 3.0, by, x - 2.0, by - h * 0.55),
        &solid(rgba("#8a4a50", 0.4)),
        0.8,
    );
    let pal = ["#ffd4e0", "#ffbcd0", "#f8a4c0", "#e87aa0", "#b84e80"];
    for (ex, ey) in tips {
        fill(
            pm,
            t,
            ellipse(ex, ey + 3.0, 27.0, 20.0),
            &solid(rgba("#a8406a", 0.9)),
        );
        for _ in 0..70 {
            let (a, d) = (r.f() * std::f32::consts::TAU, r.f().sqrt() * 25.0);
            let (px, py) = (ex + a.cos() * d * 1.1, ey + a.sin() * d * 0.8);
            let lit =
                (1.0 - ((py - (ey - 18.0)) / 36.0).clamp(0.0, 1.0)) * 4.0 + (r.f() - 0.5) * 1.6;
            let k = (4.0 - lit).clamp(0.0, 4.0) as usize;
            fill(
                pm,
                t,
                circle(px, py, 2.2 + r.f() * 3.6),
                &solid(rgba(pal[k], 1.0)),
            );
        }
        for _ in 0..10 {
            fill(
                pm,
                t,
                circle(
                    ex + (r.f() - 0.5) * 36.0,
                    ey - 12.0 + r.f() * 6.0,
                    1.4 + r.f(),
                ),
                &solid(rgba("#fff0f4", 0.9)),
            );
        }
    }
}
fn deck(pm: &mut Pixmap, t: Transform, w: f32, h: f32, rng: &mut Rng) {
    let top = 30.0;
    rect(pm, t, 0.0, 24.0, w, h, &solid(rgba("#2a1410", 1.0)));
    let n = 8;
    let tones = ["#b8754f", "#a96a44", "#c58259", "#9c5f3c", "#b06d48"];
    for i in 0..n {
        let (y0, y1) = (
            top + (i as f32 / n as f32).powf(1.6) * (h - top),
            top + ((i + 1) as f32 / n as f32).powf(1.6) * (h - top),
        );
        let scale = 0.6 + (i as f32 / n as f32) * 1.5;
        let bw = 95.0 * scale;
        let mut x = -rng.f() * bw;
        while x < w {
            let c = tones[(rng.f() * 5.0) as usize % 5];
            rect(
                pm,
                t,
                x + 0.5,
                y0 + 0.2,
                bw - 1.0,
                y1 - y0 - 0.4,
                &lin(y0, y1, &[(0.0, c), (1.0, "#8a5434")]),
            );
            rect(
                pm,
                t,
                x + 0.5,
                y0 + 0.2,
                bw - 1.0,
                0.9,
                &solid(rgba("#ffe0b0", 0.28)),
            );
            rect(pm, t, x, y0, 0.9, y1 - y0, &solid(rgba("#1a0a08", 0.8)));
            for nx in [x + 5.0, x + bw - 6.0] {
                fill(
                    pm,
                    t,
                    circle(nx, (y0 + y1) / 2.0, 0.7 * scale.max(0.7)),
                    &solid(rgba("#2a1008", 0.8)),
                );
            }
            if rng.f() < 0.25 {
                stroke(
                    pm,
                    t,
                    ellipse(
                        x + bw * (0.2 + rng.f() * 0.6),
                        (y0 + y1) / 2.0,
                        3.0 * scale,
                        1.1 * scale,
                    ),
                    &solid(rgba("#4a2410", 0.6)),
                    0.6,
                );
            }
            x += bw;
        }
        rect(pm, t, 0.0, y1 - 0.5, w, 1.0, &solid(rgba("#1a0a08", 0.75)));
    }
    let r = t.sx;
    grain(pm, r, (0.0, top, w, h - top), 0.07, 5.0, 0.6, 3.0);
    blobs(
        pm,
        r,
        (0.0, top, w, h - top),
        0.12,
        3.5,
        5.0,
        0.55,
        0.25,
        "#2a1008",
        0.5,
        0.0,
        true,
    );
    blobs(
        pm,
        r,
        (0.0, top, w, h - top),
        0.2,
        3.0,
        8.0,
        0.7,
        0.2,
        "#ffd9a8",
        0.28,
        0.0,
        true,
    );
    shade_v(pm, r, (0.0, top, w, h - top), 0.72, 1.15);
    // railing: beams, posts with caps, kick-board, hanging paper lanterns
    let wood = lin(0.0, 30.0, &[(0.0, "#5c3226"), (1.0, "#2c1612")]);
    rect(pm, t, 0.0, 22.0, w, 8.0, &solid(rgba("#2a1410", 1.0)));
    rect(pm, t, 0.0, 14.0, w, 3.4, &wood);
    rect(pm, t, 0.0, 14.0, w, 0.8, &solid(rgba("#ffb088", 0.35)));
    rect(pm, t, 0.0, 22.0, w, 2.0, &wood);
    for x in dojo_lanterns() {
        let px = x - 2.5;
        rect(
            pm,
            t,
            px,
            3.0,
            5.0,
            29.0,
            &lin(3.0, 32.0, &[(0.0, "#4a2a20"), (1.0, "#22100c")]),
        );
        rect(pm, t, px - 1.2, 1.6, 7.4, 2.6, &solid(rgba("#6a3a2c", 1.0)));
        rect(
            pm,
            t,
            px + 3.6,
            3.0,
            1.0,
            29.0,
            &solid(rgba("#ffb088", 0.2)),
        );
        glowc(pm, t, x, 12.0, 17.0, "#ff8a3a", 0.35);
        stroke(
            pm,
            t,
            line(x, 4.0, x, 6.0),
            &solid(rgba("#1a0c0a", 1.0)),
            0.6,
        );
        fill(
            pm,
            t,
            ellipse(x, 12.0, 5.2, 7.0),
            &radial(
                x,
                12.0,
                7.0,
                &[
                    (0.0, "#fff0b0", 1.0),
                    (0.45, "#ff7a30", 1.0),
                    (1.0, "#b8281c", 1.0),
                ],
            ),
        );
        for k in 0..4 {
            stroke(
                pm,
                t,
                line(
                    x - 5.0 + k as f32 * 3.3,
                    12.0 - 6.0 * (1.0 - ((k as f32 - 1.5) / 2.0).powi(2)).sqrt(),
                    x - 5.0 + k as f32 * 3.3,
                    12.0 + 6.0 * (1.0 - ((k as f32 - 1.5) / 2.0).powi(2)).sqrt(),
                ),
                &solid(rgba("#7a1810", 0.5)),
                0.35,
            );
        }
        rect(pm, t, x - 3.0, 4.6, 6.0, 1.4, &solid(rgba("#1c0c0a", 1.0)));
        rect(pm, t, x - 3.0, 18.4, 6.0, 1.4, &solid(rgba("#1c0c0a", 1.0)));
    }
}
fn dojo() -> Stage {
    let back = vec![
        // 0 sunset sky with banded cloud noise
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.05,
            3,
            0.05,
            &|pm, t, w, h, _| {
                rect(
                    pm,
                    t,
                    0.0,
                    0.0,
                    w,
                    h,
                    &lin(
                        0.0,
                        h,
                        &[
                            (0.0, "#1b1450"),
                            (0.3, "#5a2a72"),
                            (0.5, "#d4507a"),
                            (0.7, "#ff8f4a"),
                            (1.0, "#ffd38a"),
                        ],
                    ),
                );
                let r = t.sx;
                blobs(
                    pm,
                    r,
                    (0.0, 10.0, w, 150.0),
                    0.7,
                    2.6,
                    1.0,
                    0.5,
                    0.28,
                    "#ff9a6a",
                    0.6,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    r,
                    (0.0, 20.0, w, 110.0),
                    0.9,
                    3.2,
                    2.0,
                    0.56,
                    0.2,
                    "#3d1858",
                    0.6,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    r,
                    (0.0, 50.0, w, 110.0),
                    0.8,
                    3.4,
                    3.0,
                    0.6,
                    0.16,
                    "#ffe6b0",
                    0.55,
                    0.0,
                    false,
                );
                glowc(pm, t, w / 2.0, 150.0, 110.0, "#ffd890", 0.85);
                fill(
                    pm,
                    t,
                    circle(w / 2.0, 150.0, 25.0),
                    &radial(
                        w / 2.0,
                        150.0,
                        25.0,
                        &[
                            (0.0, "#ffffff", 1.0),
                            (0.8, "#fff2c0", 1.0),
                            (1.0, "#ffd890", 1.0),
                        ],
                    ),
                );
                blobs(
                    pm,
                    r,
                    (0.0, 100.0, w, 80.0),
                    0.5,
                    4.0,
                    4.0,
                    0.52,
                    0.2,
                    "#3d1858",
                    0.5,
                    0.0,
                    false,
                );
            },
        ),
        // 1 far mountains in haze
        lay(
            2.0,
            false,
            700.0,
            240.0,
            0.0,
            0.18,
            7,
            0.18,
            &|pm, t, w, _, _| {
                ridge(pm, t, w, 150.0, 70.0, "#8a4488", "#c8647a", 1.0, 0.55);
                ridge(pm, t, w, 178.0, 52.0, "#6a3070", "#a84a72", 2.0, 0.8);
                ridge(pm, t, w, 198.0, 34.0, "#4a2458", "#82406a", 3.0, 1.1);
                haze(pm, t.sx, (0.0, 130.0, w, 110.0), "#ffa070", 0.0, 0.5);
                blobs(
                    pm,
                    t.sx,
                    (0.0, 120.0, w, 100.0),
                    0.5,
                    3.5,
                    6.0,
                    0.55,
                    0.3,
                    "#ffb888",
                    0.3,
                    0.0,
                    true,
                );
            },
        ),
        // 2 village of pagodas and pines
        lay(
            3.5,
            false,
            900.0,
            240.0,
            0.0,
            0.4,
            11,
            0.4,
            &|pm, t, w, _, r| {
                for i in 0..14 {
                    pine(
                        pm,
                        t,
                        20.0 + i as f32 * 66.0 + r.f() * 30.0,
                        214.0,
                        40.0 + r.f() * 26.0,
                        "#35203e",
                    );
                }
                for i in 0..8 {
                    let x = 50.0 + i as f32 * 112.0 + r.f() * 30.0;
                    pagoda(
                        pm,
                        t,
                        x,
                        212.0 - r.f() * 8.0,
                        54.0 + r.f() * 30.0,
                        2 + (r.f() * 2.0) as i32,
                        "#2b1736",
                        0.7,
                    );
                }
                for i in 0..14 {
                    let x = r.f() * w;
                    rect(
                        pm,
                        t,
                        x,
                        196.0,
                        22.0 + r.f() * 14.0,
                        20.0,
                        &solid(rgba("#2b1736", 1.0)),
                    );
                    fill(
                        pm,
                        t,
                        poly(&[(x - 3.0, 196.0), (x + 14.0, 188.0), (x + 32.0, 196.0)]),
                        &solid(rgba("#24122e", 1.0)),
                    );
                    let _ = i;
                }
                haze(pm, t.sx, (0.0, 150.0, w, 70.0), "#d4507a", 0.0, 0.55);
                grain(pm, t.sx, (0.0, 130.0, w, 90.0), 3.0, 3.0, 0.2, 5.0);
            },
        ),
        // 3 temple wall with sakura trees (students are drawn on top by stagefx)
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.72,
            13,
            0.72,
            &|pm, t, w, _, r| {
                sakura(pm, t, r, 250.0, 208.0, 118.0);
                sakura(pm, t, r, 730.0, 208.0, 104.0);
                sakura(pm, t, r, 940.0, 208.0, 90.0);
                rect(
                    pm,
                    t,
                    0.0,
                    168.0,
                    w,
                    44.0,
                    &lin(168.0, 212.0, &[(0.0, "#4e2c3c"), (1.0, "#2a1624")]),
                );
                grain(pm, t.sx, (0.0, 168.0, w, 44.0), 3.0, 3.0, 0.55, 1.0);
                blobs(
                    pm,
                    t.sx,
                    (0.0, 168.0, w, 44.0),
                    0.35,
                    0.8,
                    9.0,
                    0.6,
                    0.2,
                    "#1c0e18",
                    0.5,
                    0.0,
                    true,
                );
                let mut x = 0.0;
                while x < w {
                    fill(
                        pm,
                        t,
                        ellipse(x + 3.5, 165.0, 3.8, 5.0),
                        &solid(rgba("#1e1018", 1.0)),
                    );
                    rect(pm, t, x, 164.0, 7.0, 4.0, &solid(rgba("#1e1018", 1.0)));
                    x += 7.0;
                }
                rect(pm, t, 0.0, 168.0, w, 1.2, &solid(rgba("#ff9a6a", 0.4)));
                let mut x = 40.0;
                while x < w {
                    rect(
                        pm,
                        t,
                        x,
                        168.0,
                        6.0,
                        44.0,
                        &lin(168.0, 212.0, &[(0.0, "#4a2224"), (1.0, "#2a1214")]),
                    );
                    rect(
                        pm,
                        t,
                        x + 4.5,
                        168.0,
                        1.0,
                        44.0,
                        &solid(rgba("#ff9a6a", 0.2)),
                    );
                    x += 120.0;
                }
            },
        ),
        // 4 wooden engawa deck with railing and paper lanterns
        layer(1100.0, 120.0, 175.0, 1.0, 13, 1.0, &|pm, t, w, h, r| {
            deck(pm, t, w, h, r)
        }),
        // 5 front: temple eave across the top (dark beam + tile row), the lanterns hang from it
        lay(
            3.0,
            true,
            580.0,
            36.0,
            -2.0,
            0.1,
            17,
            0.0,
            &|pm, t, w, h, _| {
                rect(
                    pm,
                    t,
                    0.0,
                    0.0,
                    w,
                    h,
                    &lin(
                        0.0,
                        h,
                        &[(0.0, "#120810"), (0.6, "#1e0e16"), (1.0, "#2a1420")],
                    ),
                );
                let mut x = 0.0;
                while x < w {
                    fill(
                        pm,
                        t,
                        ellipse(x + 5.0, 12.0, 5.2, 8.0),
                        &lin(4.0, 20.0, &[(0.0, "#2c1620"), (1.0, "#0e060a")]),
                    );
                    x += 10.0;
                }
                rect(
                    pm,
                    t,
                    0.0,
                    19.0,
                    w,
                    6.0,
                    &lin(19.0, 25.0, &[(0.0, "#3a1c1c"), (1.0, "#1a0c0c")]),
                );
                rect(pm, t, 0.0, 19.0, w, 0.8, &solid(rgba("#ff9a6a", 0.45)));
                grain(pm, t.sx, (0.0, 19.0, w, 6.0), 0.1, 4.0, 0.6, 2.0);
                rect(
                    pm,
                    t,
                    0.0,
                    25.0,
                    w,
                    11.0,
                    &lin_a(25.0, 36.0, &[(0.0, "#120810", 0.6), (1.0, "#120810", 0.0)]),
                );
            },
        ),
        // 6 dynamic placeholder for foreground banners
        dynamic(560.0, 240.0, 0.0, 1.22, 1.0, true),
    ];
    let particles = parts(5, 24, |r| Particle {
        x: r.f() * 470.0,
        y: r.f() * 240.0,
        vx: 0.3 + r.f() * 0.5,
        vy: 0.25 + r.f() * 0.4,
        s: 0.8 + r.f() * 1.1,
        a: 0.6 + r.f() * 0.4,
        c: if r.f() < 0.5 {
            [255, 192, 216]
        } else {
            [255, 158, 192]
        },
    });
    Stage {
        info: info("Telhado do Dojo — Pôr do Sol"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: Some([1.0, 0.55, 0.2, 0.04]),
        },
        music: 0,
        kind: StageKind::Dojo,
        bgm: None,
    }
}

// ───────────────────────────────────────────── NEON ─────────────────────────────────────────────

fn tower(
    pm: &mut Pixmap,
    t: Transform,
    x: f32,
    w: f32,
    h: f32,
    base: f32,
    body: &str,
    seed: u32,
    win_p: f32,
) {
    rect(
        pm,
        t,
        x,
        base - h,
        w,
        h + 40.0,
        &lin(base - h, base + 30.0, &[(0.0, body), (1.0, "#05030c")]),
    );
    let mut y = base - h + 4.0;
    let mut row = 0u32;
    while y < base {
        let mut xx = x + 2.0;
        let mut col = 0u32;
        while xx < x + w - 2.0 {
            if hash(row * 31 + col, seed) < win_p {
                let c = if hash(row, col + seed) < 0.55 {
                    rgba("#8fd4ff", 0.55)
                } else if hash(col, row + seed) < 0.5 {
                    rgba("#ff9ae0", 0.5)
                } else {
                    rgba("#ffe4a0", 0.45)
                };
                rect(pm, t, xx, y, 1.8, 2.2, &solid(c));
            }
            xx += 3.8;
            col += 1;
        }
        y += 4.6;
        row += 1;
    }
}
fn neon() -> Stage {
    let signs = neon_signs();
    let back = vec![
        // 0 smoggy night sky with a ringed planet
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.05,
            2,
            0.05,
            &|pm, t, w, h, r| {
                rect(
                    pm,
                    t,
                    0.0,
                    0.0,
                    w,
                    h,
                    &lin(
                        0.0,
                        h,
                        &[
                            (0.0, "#04020e"),
                            (0.5, "#1a0a38"),
                            (0.8, "#3d1260"),
                            (1.0, "#6a1c70"),
                        ],
                    ),
                );
                let rr = t.sx;
                for _ in 0..90 {
                    let a = r.f() * 0.7;
                    rect(
                        pm,
                        t,
                        r.f() * w,
                        r.f() * 120.0,
                        0.7,
                        0.7,
                        &solid(rgba("#ffffff", a)),
                    );
                }
                glowc(pm, t, w * 0.72, 62.0, 56.0, "#ff3cc8", 0.4);
                fill(
                    pm,
                    t,
                    circle(w * 0.72, 62.0, 22.0),
                    &radial(
                        w * 0.72 - 6.0,
                        56.0,
                        28.0,
                        &[
                            (0.0, "#ffb0e8", 1.0),
                            (0.6, "#a02a98", 1.0),
                            (1.0, "#3a0e50", 1.0),
                        ],
                    ),
                );
                stroke(
                    pm,
                    t,
                    ellipse(w * 0.72, 62.0, 40.0, 7.0),
                    &solid(rgba("#ffd0f4", 0.5)),
                    1.4,
                );
                blobs(
                    pm,
                    rr,
                    (0.0, 20.0, w, 200.0),
                    0.6,
                    2.6,
                    2.0,
                    0.5,
                    0.3,
                    "#ff3cc8",
                    0.22,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    rr,
                    (0.0, 60.0, w, 160.0),
                    0.7,
                    2.8,
                    5.0,
                    0.55,
                    0.25,
                    "#22d8ff",
                    0.16,
                    0.0,
                    false,
                );
            },
        ),
        // 1 far skyline
        lay(
            2.5,
            false,
            760.0,
            240.0,
            0.0,
            0.2,
            4,
            0.2,
            &|pm, t, w, _, _| {
                for (x, bw, bh) in skyline(21, w, 16.0, 34.0, 60.0, 170.0, 8.0) {
                    tower(pm, t, x, bw, bh, 200.0, "#1c1040", 3, 0.38);
                }
                haze(pm, t.sx, (0.0, 60.0, w, 160.0), "#7a1c88", 0.0, 0.55);
            },
        ),
        // 2 mid city with neon sign boards
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.5,
            9,
            0.5,
            &|pm, t, w, _, _| {
                for (x, bw, bh) in skyline(41, w, 28.0, 56.0, 90.0, 190.0, 54.0) {
                    tower(pm, t, x, bw, bh, 205.0, "#150c2c", 5, 0.2);
                    rect(pm, t, x, 205.0 - bh, bw, 1.4, &solid(rgba("#6a50b0", 0.5)));
                    grain(pm, t.sx, (x, 205.0 - bh, bw, bh), 4.0, 4.0, 0.3, 2.0);
                    for k in 0..2 {
                        rect(
                            pm,
                            t,
                            x + 4.0 + k as f32 * (bw - 12.0),
                            205.0 - bh - 5.0,
                            1.2,
                            5.0,
                            &solid(rgba("#33255a", 1.0)),
                        );
                    }
                }
                for (x, y, sw, sh, c) in &signs {
                    let col = NEON[*c];
                    glowc(
                        pm,
                        t,
                        x + sw / 2.0,
                        y + sh / 2.0,
                        sh.max(*sw) * 1.1,
                        col,
                        0.42,
                    );
                    let rp =
                        tiny_skia::Rect::from_xywh(*x, *y, *sw, *sh).map(PathBuilder::from_rect);
                    rect(pm, t, *x, *y, *sw, *sh, &solid(rgba("#0a0618", 0.9)));
                    stroke(pm, t, rp.clone(), &solid(rgba(col, 0.3)), 4.0);
                    stroke(pm, t, rp, &solid(rgba(col, 1.0)), 1.2);
                    for q in 0..3 {
                        stroke(
                            pm,
                            t,
                            line(
                                x + 2.0,
                                y + 4.0 + q as f32 * sh / 3.3,
                                x + sw - 2.0,
                                y + 6.0 + q as f32 * sh / 3.3,
                            ),
                            &solid(rgba(col, 1.0)),
                            1.1,
                        );
                    }
                }
                haze(pm, t.sx, (0.0, 130.0, w, 90.0), "#5a1c80", 0.0, 0.38);
            },
        ),
        // 3 street-level shop fronts (awnings, lit windows, vending machines); the balcony crowd is drawn by stagefx
        lay(
            3.5,
            false,
            1100.0,
            240.0,
            0.0,
            0.8,
            15,
            0.8,
            &|pm, t, w, _, r| {
                let mut x = 0.0;
                let mut i = 0u32;
                while x < w {
                    let bw = 64.0 + r.f() * 50.0;
                    let top = 130.0 + r.f() * 36.0;
                    rect(
                        pm,
                        t,
                        x,
                        top,
                        bw,
                        210.0 - top,
                        &lin(top, 210.0, &[(0.0, "#1a1038"), (1.0, "#0a0618")]),
                    );
                    grain(pm, t.sx, (x, top, bw, 210.0 - top), 3.0, 3.0, 0.35, 3.0);
                    let c = NEON[(hash(i, 3) * 5.0) as usize % 5];
                    rect(
                        pm,
                        t,
                        x + 6.0,
                        176.0,
                        bw - 12.0,
                        30.0,
                        &lin(176.0, 206.0, &[(0.0, "#fff2c8"), (1.0, "#ffb870")]),
                    );
                    rect(
                        pm,
                        t,
                        x + 6.0,
                        176.0,
                        bw - 12.0,
                        30.0,
                        &lin_a(
                            176.0,
                            206.0,
                            &[
                                (0.0, "#000000", 0.5),
                                (0.35, "#000000", 0.0),
                                (1.0, "#000000", 0.35),
                            ],
                        ),
                    );
                    for q in 0..(bw / 14.0) as i32 {
                        rect(
                            pm,
                            t,
                            x + 10.0 + q as f32 * 14.0,
                            184.0,
                            1.0,
                            22.0,
                            &solid(rgba("#1a0c20", 0.7)),
                        );
                    }
                    rect(
                        pm,
                        t,
                        x + 2.0,
                        164.0,
                        bw - 4.0,
                        12.0,
                        &lin(164.0, 176.0, &[(0.0, c), (1.0, "#150a28")]),
                    );
                    for q in 0..((bw - 8.0) / 6.0) as i32 {
                        rect(
                            pm,
                            t,
                            x + 4.0 + q as f32 * 6.0,
                            164.0,
                            3.0,
                            12.0,
                            &solid(rgba("#000000", 0.18)),
                        );
                    }
                    glowc(pm, t, x + bw / 2.0, 190.0, bw * 0.8, "#ffa060", 0.22);
                    stroke(
                        pm,
                        t,
                        line(x + 4.0, top + 6.0, x + bw - 4.0, top + 6.0),
                        &solid(rgba(c, 0.9)),
                        1.0,
                    );
                    x += bw + r.f() * 12.0;
                    i += 1;
                }
                haze(pm, t.sx, (0.0, 120.0, w, 100.0), "#3a1060", 0.0, 0.3);
            },
        ),
        // 4 wet asphalt street with puddles, lane paint, manholes and a barrier
        layer(1100.0, 120.0, 175.0, 1.0, 12, 1.0, &|pm, t, w, h, r| {
            let rr = t.sx;
            rect(
                pm,
                t,
                0.0,
                28.0,
                w,
                h,
                &lin(
                    28.0,
                    h,
                    &[(0.0, "#2a1c48"), (0.35, "#160c2c"), (1.0, "#080414")],
                ),
            );
            grain(pm, rr, (0.0, 28.0, w, h - 28.0), 6.0, 6.0, 0.7, 1.0);
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.25,
                0.8,
                4.0,
                0.58,
                0.25,
                "#000000",
                0.4,
                0.0,
                true,
            );
            for _ in 0..380 {
                let a = 0.1 + r.f() * 0.25;
                rect(
                    pm,
                    t,
                    r.f() * w,
                    30.0 + r.f() * 88.0,
                    0.7,
                    0.7,
                    &solid(rgba(if r.f() < 0.5 { "#8a7ac8" } else { "#000000" }, a)),
                );
            }
            for _ in 0..16 {
                let x = r.f() * w;
                let y = 40.0 + r.f() * 70.0;
                let mut b = PathBuilder::new();
                b.move_to(x, y);
                for _ in 0..3 {
                    b.line_to(x + (r.f() - 0.2) * 30.0, y + (r.f() - 0.5) * 8.0);
                }
                stroke(pm, t, b.finish(), &solid(rgba("#000000", 0.5)), 0.5);
            }
            // curb + lane markings
            rect(
                pm,
                t,
                0.0,
                26.0,
                w,
                4.0,
                &lin(26.0, 30.0, &[(0.0, "#4a3a78"), (1.0, "#1c1236")]),
            );
            let mut x = 8.0;
            while x < w {
                rect(pm, t, x, 70.0, 34.0, 2.4, &solid(rgba("#e8e0ff", 0.55)));
                x += 78.0;
            }
            rect(pm, t, 0.0, 104.0, w, 2.0, &solid(rgba("#ffe23a", 0.35)));
            for i in 0..3 {
                let x = 200.0 + i as f32 * 380.0;
                fill(
                    pm,
                    t,
                    ellipse(x, 62.0, 13.0, 3.6),
                    &solid(rgba("#0c0618", 1.0)),
                );
                stroke(
                    pm,
                    t,
                    ellipse(x, 62.0, 13.0, 3.6),
                    &solid(rgba("#6a5aa8", 0.6)),
                    0.6,
                );
                for k in 0..4 {
                    rect(
                        pm,
                        t,
                        x - 9.0 + k as f32 * 4.6,
                        61.0,
                        0.7,
                        2.2,
                        &solid(rgba("#4a3a78", 0.8)),
                    );
                }
            }
            // puddles reflecting the neon sky
            for i in 0..11 {
                let (x, y) = (r.f() * w, 44.0 + r.f() * 66.0);
                let (pw, ph) = (22.0 + r.f() * 40.0, 3.0 + r.f() * 4.5);
                fill(
                    pm,
                    t,
                    ellipse(x, y, pw, ph),
                    &radial(
                        x,
                        y,
                        pw,
                        &[
                            (0.0, if i % 2 == 0 { "#ff6ad8" } else { "#52e8ff" }, 0.8),
                            (0.5, "#6a40c0", 0.65),
                            (1.0, "#1a0c38", 0.5),
                        ],
                    ),
                );
                stroke(
                    pm,
                    t,
                    ellipse(x, y, pw, ph),
                    &solid(rgba("#c0b0ff", 0.4)),
                    0.5,
                );
                rect(
                    pm,
                    t,
                    x - pw * 0.5,
                    y - 0.6,
                    pw,
                    0.8,
                    &solid(rgba("#ffffff", 0.35)),
                );
            }
            // barrier
            rect(pm, t, 0.0, 6.0, w, 2.0, &solid(rgba("#4c3a80", 1.0)));
            rect(pm, t, 0.0, 6.0, w, 0.7, &solid(rgba("#a898ff", 0.6)));
            let mut x = 0.0;
            while x < w {
                rect(
                    pm,
                    t,
                    x,
                    6.0,
                    1.6,
                    22.0,
                    &lin(6.0, 28.0, &[(0.0, "#4a3a80"), (1.0, "#1a1234")]),
                );
                x += 15.0;
            }
            let mut x = 0.0;
            while x < w {
                rect(pm, t, x, 20.0, 12.0, 1.0, &solid(rgba("#33255a", 0.9)));
                x += 15.0;
            }
        }),
        // 5 dynamic: hanging cables + street lamps in front
        dynamic(560.0, 240.0, 0.0, 1.2, 1.0, true),
    ];
    let particles = parts(9, 24, |r| Particle {
        x: r.f() * 470.0,
        y: r.f() * 240.0,
        vx: -0.6,
        vy: 5.0 + r.f() * 2.0,
        s: 0.35,
        a: 0.35,
        c: [154, 216, 255],
    });
    Stage {
        info: info("Neo-Metrópole 2099"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: None,
        },
        music: 1,
        kind: StageKind::Neon,
        bgm: None,
    }
}

// ───────────────────────────────────────────── BAMBOO ─────────────────────────────────────────────

fn bamboo() -> Stage {
    let back = vec![
        // 0 moonlit sky: moon, stars, thin clouds
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.04,
            2,
            0.04,
            &|pm, t, w, h, r| {
                rect(
                    pm,
                    t,
                    0.0,
                    0.0,
                    w,
                    h,
                    &lin(
                        0.0,
                        h,
                        &[(0.0, "#030d1a"), (0.55, "#0f3646"), (1.0, "#2c6a6a")],
                    ),
                );
                for _ in 0..110 {
                    let a = 0.2 + r.f() * 0.7;
                    fill(
                        pm,
                        t,
                        circle(r.f() * w, r.f() * 130.0, 0.35 + r.f() * 0.5),
                        &solid(rgba("#dffcff", a)),
                    );
                }
                glowc(pm, t, w * 0.35, 55.0, 90.0, "#bff4ee", 0.55);
                fill(
                    pm,
                    t,
                    circle(w * 0.35, 55.0, 17.0),
                    &radial(
                        w * 0.35 - 4.0,
                        51.0,
                        20.0,
                        &[
                            (0.0, "#ffffff", 1.0),
                            (0.7, "#e6fff8", 1.0),
                            (1.0, "#b8e8e0", 1.0),
                        ],
                    ),
                );
                for (dx, dy, rr) in [
                    (-5.0, -4.0, 3.2),
                    (4.0, 3.0, 4.2),
                    (-2.0, 7.0, 2.0),
                    (6.0, -6.0, 2.2),
                ] {
                    fill(
                        pm,
                        t,
                        circle(w * 0.35 + dx, 55.0 + dy, rr),
                        &solid(rgba("#9fd0c8", 0.4)),
                    );
                }
                blobs(
                    pm,
                    t.sx,
                    (0.0, 20.0, w, 140.0),
                    0.6,
                    2.8,
                    4.0,
                    0.56,
                    0.22,
                    "#0a2a3a",
                    0.5,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    t.sx,
                    (0.0, 20.0, w, 140.0),
                    0.6,
                    2.8,
                    4.0,
                    0.64,
                    0.2,
                    "#9fe6e0",
                    0.22,
                    0.0,
                    false,
                );
            },
        ),
        // 1 far bamboo silhouettes in mist
        lay(
            2.0,
            false,
            900.0,
            260.0,
            -10.0,
            0.18,
            21,
            0.18,
            &|pm, t, w, _, r| {
                let mut x = 0.0;
                while x < w {
                    let bw = 2.5 + r.f() * 2.0;
                    let c = solid(rgba("#194a4a", 1.0));
                    rect(pm, t, x, 0.0, bw, 260.0, &c);
                    rect(
                        pm,
                        t,
                        x + bw * 0.6,
                        0.0,
                        bw * 0.3,
                        260.0,
                        &solid(rgba("#a0e6dc", 0.22)),
                    );
                    let mut y = 24.0 + r.f() * 24.0;
                    while y < 240.0 {
                        rect(
                            pm,
                            t,
                            x - 0.6,
                            y,
                            bw + 1.2,
                            1.2,
                            &solid(rgba("#0e3434", 1.0)),
                        );
                        y += 22.0 + r.f() * 10.0;
                    }
                    x += 8.0 + r.f() * 14.0;
                }
                haze(pm, t.sx, (0.0, 0.0, w, 260.0), "#3a8a88", 0.35, 0.6);
            },
        ),
        // 2 dynamic swaying mid grove, 3 dynamic near grove
        dynamic(900.0, 260.0, -10.0, 0.42, 0.42, false),
        dynamic(1100.0, 260.0, -10.0, 0.72, 0.72, false),
        // 4 shallow pond with stepping stones, lily pads, reeds
        layer(1100.0, 120.0, 175.0, 1.0, 5, 1.0, &|pm, t, w, h, r| {
            let rr = t.sx;
            rect(
                pm,
                t,
                0.0,
                28.0,
                w,
                h,
                &lin(
                    28.0,
                    h,
                    &[(0.0, "#2f6a66"), (0.4, "#14403f"), (1.0, "#082220")],
                ),
            );
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.18,
                0.9,
                3.0,
                0.5,
                0.3,
                "#0a2a2c",
                0.5,
                0.0,
                true,
            );
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.3,
                1.4,
                7.0,
                0.62,
                0.2,
                "#8fe8d8",
                0.2,
                0.0,
                true,
            );
            for _ in 0..90 {
                let y = 32.0 + r.f() * 84.0;
                let x = r.f() * w;
                let l = 6.0 + r.f() * 26.0;
                rect(
                    pm,
                    t,
                    x,
                    y,
                    l,
                    0.5,
                    &solid(rgba("#c8fff4", 0.1 + r.f() * 0.18)),
                );
            }
            // pebbles under shallow water
            for _ in 0..70 {
                fill(
                    pm,
                    t,
                    ellipse(
                        r.f() * w,
                        40.0 + r.f() * 74.0,
                        1.5 + r.f() * 2.5,
                        0.8 + r.f(),
                    ),
                    &solid(rgba(if r.f() < 0.5 { "#6a8a80" } else { "#3a5a56" }, 0.35)),
                );
            }
            // stepping stones
            for i in 0..7 {
                let x = 60.0 + i as f32 * 150.0 + r.f() * 40.0;
                let y = 60.0 + r.f() * 40.0;
                let (sw, sh) = (18.0 + r.f() * 10.0, 3.6 + r.f() * 1.5);
                fill(
                    pm,
                    t,
                    ellipse(x, y + 1.6, sw, sh),
                    &solid(rgba("#07201f", 0.6)),
                );
                fill(
                    pm,
                    t,
                    ellipse(x, y, sw, sh),
                    &lin(y - sh, y + sh, &[(0.0, "#8aa8a0"), (1.0, "#3a5a56")]),
                );
                stroke(
                    pm,
                    t,
                    ellipse(x, y, sw, sh),
                    &solid(rgba("#d8fff0", 0.35)),
                    0.5,
                );
            }
            grain(pm, rr, (0.0, 28.0, w, h - 28.0), 6.0, 6.0, 0.35, 2.0);
            // lily pads and flowers
            for _ in 0..22 {
                let (x, y, lr) = (r.f() * w, 38.0 + r.f() * 78.0, 3.0 + r.f() * 3.4);
                let mut b = PathBuilder::new();
                b.move_to(x, y);
                b.push_oval(
                    tiny_skia::Rect::from_xywh(x - lr * 1.8, y - lr * 0.5, lr * 3.6, lr).unwrap(),
                );
                fill(
                    pm,
                    t,
                    ellipse(x, y, lr * 1.8, lr * 0.5),
                    &lin(
                        y - lr * 0.5,
                        y + lr * 0.5,
                        &[(0.0, "#5aa860"), (1.0, "#2a6a46")],
                    ),
                );
                stroke(
                    pm,
                    t,
                    line(x, y, x + lr * 1.6, y - lr * 0.1),
                    &solid(rgba("#0e3a2a", 0.8)),
                    0.35,
                );
                if r.f() < 0.3 {
                    fill(
                        pm,
                        t,
                        ellipse(x - lr, y - lr * 0.4, 1.4, 1.0),
                        &solid(rgba("#ffd0e8", 0.95)),
                    );
                }
            }
            // far bank with ferns and rocks
            rect(
                pm,
                t,
                0.0,
                0.0,
                w,
                30.0,
                &lin(0.0, 30.0, &[(0.0, "#071c1c"), (1.0, "#0e3030")]),
            );
            let mut x = 0.0;
            while x < w {
                let hh = 8.0 + r.f() * 18.0;
                let mut b = PathBuilder::new();
                b.move_to(x, 30.0);
                b.quad_to(x + 2.0, 30.0 - hh * 0.8, x + 5.0 + r.f() * 6.0, 30.0 - hh);
                b.quad_to(x + 5.0, 30.0 - hh * 0.5, x + 7.0, 30.0);
                b.close();
                fill(
                    pm,
                    t,
                    b.finish(),
                    &solid(rgba(if r.f() < 0.5 { "#0e3a30" } else { "#164a3a" }, 1.0)),
                );
                x += 3.0 + r.f() * 3.0;
            }
            for _ in 0..16 {
                let x = r.f() * w;
                fill(
                    pm,
                    t,
                    ellipse(x, 28.0, 7.0 + r.f() * 9.0, 4.0 + r.f() * 3.0),
                    &lin(22.0, 32.0, &[(0.0, "#4a6e66"), (1.0, "#14302c")]),
                );
            }
            rect(pm, t, 0.0, 29.0, w, 1.4, &solid(rgba("#b0fff0", 0.25)));
        }),
        // 5 dynamic foreground stalks
        dynamic(560.0, 260.0, -10.0, 1.28, 1.0, true),
    ];
    let particles = parts(3, 20, |r| Particle {
        x: r.f() * 470.0,
        y: 80.0 + r.f() * 140.0,
        vx: (r.f() - 0.5) * 0.3,
        vy: (r.f() - 0.5) * 0.2,
        s: 0.9,
        a: 0.8,
        c: [217, 255, 122],
    });
    Stage {
        info: info("Bambuzal ao Luar"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: Some([0.16, 0.47, 0.55, 0.06]),
        },
        music: 2,
        kind: StageKind::Bamboo,
        bgm: None,
    }
}

// ───────────────────────────────────────────── PLAINS ─────────────────────────────────────────────

fn cactus(pm: &mut Pixmap, t: Transform, x: f32, by: f32, h: f32) {
    let g = lin(by - h, by, &[(0.0, "#5a7a3a"), (1.0, "#2e4a24")]);
    rect(pm, t, x - 3.0, by - h, 6.0, h, &g);
    fill(pm, t, ellipse(x, by - h, 3.0, 3.0), &g);
    for (dx, up) in [(-9.0, h * 0.55), (9.0, h * 0.4)] {
        rect(pm, t, x.min(x + dx), by - up, dx.abs(), 3.4, &g);
        rect(pm, t, x + dx - 1.7, by - up - 12.0, 3.4, 12.0 + 3.0, &g);
        fill(pm, t, ellipse(x + dx, by - up - 12.0, 1.7, 1.7), &g);
    }
    rect(pm, t, x + 1.2, by - h, 1.0, h, &solid(rgba("#c8e8a0", 0.3)));
}
fn rockform(
    pm: &mut Pixmap,
    t: Transform,
    r: &mut Rng,
    x: f32,
    by: f32,
    w: f32,
    h: f32,
    c1: &str,
    c2: &str,
) {
    let n = 7;
    let mut pts = vec![(x, by)];
    for i in 0..=n {
        let f = i as f32 / n as f32;
        let hh = h * (0.45 + 0.55 * (1.0 - (f * 2.0 - 1.0).powi(2))) * (0.75 + 0.25 * r.f());
        pts.push((x + f * w, by - hh));
    }
    pts.push((x + w, by));
    fill(pm, t, poly(&pts), &lin(by - h, by, &[(0.0, c1), (1.0, c2)]));
    // lit edge on the sun side + strata
    let mut b = PathBuilder::new();
    for (i, p) in pts.iter().enumerate().skip(1).take(n + 1) {
        if i == 1 {
            b.move_to(p.0, p.1)
        } else {
            b.line_to(p.0, p.1)
        }
    }
    stroke(pm, t, b.finish(), &solid(rgba("#ffe6c0", 0.55)), 0.8);
    for k in 1..4 {
        let y = by - h * k as f32 * 0.22;
        stroke(
            pm,
            t,
            line(x + w * 0.1, y, x + w * 0.9, y + (r.f() - 0.5) * 3.0),
            &solid(rgba("#3a1a0e", 0.25)),
            0.7,
        );
    }
}
fn plains() -> Stage {
    let back = vec![
        // 0 desert sky: gradient, sun, cloud noise
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.04,
            8,
            0.04,
            &|pm, t, w, h, _| {
                rect(
                    pm,
                    t,
                    0.0,
                    0.0,
                    w,
                    h,
                    &lin(
                        0.0,
                        h,
                        &[
                            (0.0, "#1f5fc8"),
                            (0.45, "#6ab0ee"),
                            (0.8, "#c8e4f4"),
                            (1.0, "#f4ecd8"),
                        ],
                    ),
                );
                let r = t.sx;
                glowc(pm, t, w * 0.78, 40.0, 100.0, "#fff4c0", 0.8);
                fill(
                    pm,
                    t,
                    circle(w * 0.78, 40.0, 11.0),
                    &solid(rgba("#ffffff", 1.0)),
                );
                blobs(
                    pm,
                    r,
                    (0.0, 10.0, w, 170.0),
                    0.7,
                    2.4,
                    1.0,
                    0.55,
                    0.22,
                    "#ffffff",
                    0.85,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    r,
                    (0.0, 10.0, w, 170.0),
                    0.7,
                    2.4,
                    1.0,
                    0.7,
                    0.2,
                    "#9ab0d0",
                    0.35,
                    0.0,
                    false,
                );
            },
        ),
        // 1 far mesas
        lay(
            2.0,
            false,
            760.0,
            240.0,
            0.0,
            0.14,
            31,
            0.14,
            &|pm, t, w, _, r| {
                let mut x = 0.0;
                while x < w {
                    let mw = 60.0 + r.f() * 120.0;
                    let mh = 24.0 + r.f() * 44.0;
                    let by = 178.0;
                    fill(
                        pm,
                        t,
                        poly(&[
                            (x, by + 30.0),
                            (x + 6.0, by - mh * 0.7),
                            (x + 10.0, by - mh),
                            (x + mw - 10.0, by - mh),
                            (x + mw - 4.0, by - mh * 0.6),
                            (x + mw, by + 30.0),
                        ]),
                        &lin(by - mh, by + 10.0, &[(0.0, "#c9a898"), (1.0, "#a8847c")]),
                    );
                    for k in 0..4 {
                        rect(
                            pm,
                            t,
                            x + 6.0,
                            by - mh + 6.0 + k as f32 * (mh / 4.0),
                            mw - 12.0,
                            1.0,
                            &solid(rgba("#8a6a64", 0.4)),
                        );
                    }
                    x += mw * 0.7 + r.f() * 60.0;
                }
                ridge(pm, t, w, 196.0, 26.0, "#bf9a86", "#d8c0a8", 4.0, 0.7);
                haze(pm, t.sx, (0.0, 120.0, w, 120.0), "#e8e0d0", 0.2, 0.7);
            },
        ),
        // 2 mid canyon walls: lit / shadow faces, strata, erosion grooves
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.4,
            33,
            0.4,
            &|pm, t, w, _, r| {
                let mut x = -20.0;
                while x < w {
                    let mh = 60.0 + r.f() * 86.0;
                    let mw = 56.0 + r.f() * 56.0;
                    let top = 198.0 - mh;
                    let split = x + mw * (0.45 + r.f() * 0.15);
                    let sil = [
                        (x, 212.0),
                        (x + 4.0, 196.0 - mh * 0.85),
                        (x + 13.0, top + 2.0),
                        (split, top),
                        (x + mw - 7.0, 196.0 - mh * 0.7),
                        (x + mw, 212.0),
                    ];
                    fill(
                        pm,
                        t,
                        poly(&[sil[0], sil[1], sil[2], sil[3], (split - 2.0, 212.0)]),
                        &lin(
                            top,
                            212.0,
                            &[(0.0, "#e8a874"), (0.5, "#c47c50"), (1.0, "#8a5036")],
                        ),
                    );
                    fill(
                        pm,
                        t,
                        poly(&[(split - 2.0, 212.0), sil[3], sil[4], sil[5]]),
                        &lin(
                            top,
                            212.0,
                            &[(0.0, "#a8684a"), (0.5, "#8a4c34"), (1.0, "#5a3022")],
                        ),
                    );
                    let mut y = top + 8.0;
                    let mut k = 0;
                    while y < 210.0 {
                        let col = if k % 2 == 0 {
                            rgba("#4a2214", 0.4)
                        } else {
                            rgba("#f4cc9c", 0.25)
                        };
                        stroke(
                            pm,
                            t,
                            line(x + mw * 0.12, y, x + mw * 0.88, y + (r.f() - 0.5) * 2.0),
                            &solid(col),
                            0.9,
                        );
                        y += 4.0 + r.f() * 7.0;
                        k += 1;
                    }
                    for _ in 0..3 {
                        let gx = x + mw * (0.2 + r.f() * 0.6);
                        let gy = top + 8.0 + r.f() * mh * 0.4;
                        stroke(
                            pm,
                            t,
                            line(gx, gy, gx + (r.f() - 0.5) * 4.0, gy + 10.0 + r.f() * 16.0),
                            &solid(rgba("#3a1a0e", 0.22)),
                            0.7,
                        );
                    }
                    stroke(
                        pm,
                        t,
                        poly(&[sil[1], sil[2], sil[3]]),
                        &solid(rgba("#ffe6c0", 0.6)),
                        0.9,
                    );
                    x += mw * 0.8 + r.f() * 60.0;
                }
                grain(pm, t.sx, (0.0, 80.0, w, 140.0), 6.0, 6.0, 0.35, 2.0);
                haze(pm, t.sx, (0.0, 100.0, w, 112.0), "#f0e0c8", 0.0, 0.22);
            },
        ),
        // 3 near rocks, cacti, dry shrubs (windmill is animated by stagefx)
        lay(
            3.5,
            false,
            1100.0,
            240.0,
            0.0,
            0.7,
            35,
            0.7,
            &|pm, t, w, _, r| {
                for i in 0..5 {
                    let x = 40.0 + i as f32 * 230.0 + r.f() * 60.0;
                    if i != 2 {
                        let (rw, rh) = (46.0 + r.f() * 40.0, 30.0 + r.f() * 36.0);
                        rockform(pm, t, r, x, 211.0, rw, rh, "#c8845a", "#6a3a26");
                    }
                }
                for i in 0..6 {
                    cactus(
                        pm,
                        t,
                        120.0 + i as f32 * 170.0 + r.f() * 60.0,
                        210.0,
                        22.0 + r.f() * 20.0,
                    );
                }
                for _ in 0..40 {
                    let x = r.f() * w;
                    let hh = 4.0 + r.f() * 8.0;
                    for k in 0..5 {
                        stroke(
                            pm,
                            t,
                            line(x, 210.0, x + (k as f32 - 2.0) * 2.2, 210.0 - hh),
                            &solid(rgba(if r.f() < 0.5 { "#8a7a3a" } else { "#a89448" }, 0.9)),
                            0.6,
                        );
                    }
                }
                grain(pm, t.sx, (0.0, 150.0, w, 70.0), 3.0, 3.0, 0.4, 5.0);
                haze(pm, t.sx, (0.0, 160.0, w, 60.0), "#f4e4c8", 0.0, 0.25);
            },
        ),
        // 4 cracked dry earth
        layer(1100.0, 120.0, 175.0, 1.0, 35, 1.0, &|pm, t, w, h, r| {
            let rr = t.sx;
            rect(
                pm,
                t,
                0.0,
                28.0,
                w,
                h,
                &lin(
                    28.0,
                    h,
                    &[(0.0, "#e0bc8c"), (0.5, "#c89c6c"), (1.0, "#9a7048")],
                ),
            );
            grain(pm, rr, (0.0, 28.0, w, h - 28.0), 7.0, 7.0, 0.55, 1.0);
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.3,
                1.0,
                2.0,
                0.55,
                0.3,
                "#8a5a38",
                0.45,
                0.0,
                true,
            );
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.4,
                1.4,
                6.0,
                0.66,
                0.2,
                "#fff0cc",
                0.35,
                0.0,
                true,
            );
            // sand ripples
            for _ in 0..60 {
                let x = r.f() * w;
                let y = 36.0 + r.f() * 80.0;
                let l = 20.0 + r.f() * 60.0;
                stroke(
                    pm,
                    t,
                    line(x, y, x + l, y + (r.f() - 0.5) * 3.0),
                    &solid(rgba("#7a4c2c", 0.14)),
                    0.7,
                );
            }
            // dried-mud cracks (polygon network)
            for _ in 0..34 {
                let (mut x, mut y) = (r.f() * w, 36.0 + r.f() * 80.0);
                let mut b = PathBuilder::new();
                b.move_to(x, y);
                for _ in 0..5 {
                    x += (r.f() - 0.5) * 22.0;
                    y += (r.f() - 0.5) * 8.0;
                    b.line_to(x, y);
                }
                let p = b.finish();
                stroke(pm, t, p.clone(), &solid(rgba("#3a2010", 0.55)), 0.8);
            }
            for _ in 0..180 {
                fill(
                    pm,
                    t,
                    ellipse(
                        r.f() * w,
                        34.0 + r.f() * 84.0,
                        0.8 + r.f() * 2.8,
                        0.5 + r.f() * 1.0,
                    ),
                    &solid(rgba(
                        if r.f() < 0.5 { "#5a3c28" } else { "#f4dcb0" },
                        0.4 + r.f() * 0.3,
                    )),
                );
            }
            shade_v(pm, rr, (0.0, 28.0, w, h - 28.0), 0.78, 1.05);
            // far edge: scattered stones and tufts
            rect(
                pm,
                t,
                0.0,
                0.0,
                w,
                30.0,
                &lin_a(0.0, 30.0, &[(0.0, "#e0bc8c", 0.0), (1.0, "#e0bc8c", 1.0)]),
            );
            for _ in 0..40 {
                let x = r.f() * w;
                let hh = 3.0 + r.f() * 8.0;
                for k in 0..4 {
                    stroke(
                        pm,
                        t,
                        line(x, 29.0, x + (k as f32 - 1.5) * 1.6, 29.0 - hh),
                        &solid(rgba("#a88c4a", 0.9)),
                        0.5,
                    );
                }
            }
            for _ in 0..22 {
                fill(
                    pm,
                    t,
                    ellipse(r.f() * w, 28.0, 2.0 + r.f() * 5.0, 1.4 + r.f() * 2.0),
                    &lin(24.0, 30.0, &[(0.0, "#b88a68"), (1.0, "#6a4630")]),
                );
            }
            rect(pm, t, 0.0, 29.0, w, 1.0, &solid(rgba("#fff0cc", 0.3)));
        }),
        // 5 dynamic foreground grass
        dynamic(560.0, 240.0, 0.0, 1.25, 1.0, true),
    ];
    let particles = parts(4, 14, |r| Particle {
        x: r.f() * 470.0,
        y: 150.0 + r.f() * 80.0,
        vx: 0.8 + r.f(),
        vy: -0.05,
        s: 0.6,
        a: 0.4,
        c: [232, 200, 160],
    });
    Stage {
        info: info("Planalto Rochoso"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: Some([1.0, 0.85, 0.5, 0.025]),
        },
        music: 3,
        kind: StageKind::Plains,
        bgm: None,
    }
}

// ───────────────────────────────────────────── SPIRIT ─────────────────────────────────────────────

fn gate(pm: &mut Pixmap, t: Transform, cx: f32) {
    let stone = lin(60.0, 214.0, &[(0.0, "#2a4a50"), (1.0, "#0c1c22")]);
    // pillars with round bases, stacked beams (kasagi + nuki), shimenawa rope
    for sx in [-1.0f32, 1.0] {
        let px = cx + sx * 98.0;
        rect(pm, t, px - 11.0, 62.0, 22.0, 150.0, &stone);
        rect(
            pm,
            t,
            px - 14.0,
            190.0,
            28.0,
            24.0,
            &lin(190.0, 214.0, &[(0.0, "#305458"), (1.0, "#0e2228")]),
        );
        rect(
            pm,
            t,
            px - 11.0 + if sx < 0.0 { 15.0 } else { 0.0 },
            62.0,
            4.0,
            130.0,
            &solid(rgba("#9ad8d0", 0.16)),
        );
    }
    let mut b = PathBuilder::new();
    b.move_to(cx - 154.0, 54.0);
    b.quad_to(cx, 24.0, cx + 154.0, 54.0);
    b.line_to(cx + 142.0, 68.0);
    b.quad_to(cx, 44.0, cx - 142.0, 68.0);
    b.close();
    fill(
        pm,
        t,
        b.finish(),
        &lin(30.0, 68.0, &[(0.0, "#3a6068"), (1.0, "#12282e")]),
    );
    rect(
        pm,
        t,
        cx - 124.0,
        80.0,
        248.0,
        11.0,
        &lin(80.0, 91.0, &[(0.0, "#2e5258"), (1.0, "#10242a")]),
    );
    rect(
        pm,
        t,
        cx - 124.0,
        80.0,
        248.0,
        1.0,
        &solid(rgba("#a0fff0", 0.35)),
    );
    let mut b = PathBuilder::new();
    b.move_to(cx - 98.0, 97.0);
    b.quad_to(cx, 116.0, cx + 98.0, 97.0);
    stroke(pm, t, b.finish(), &solid(rgba("#c8b88a", 1.0)), 4.0);
    for k in 0..7 {
        let f = k as f32 / 6.0;
        let x = cx - 98.0 + f * 196.0;
        let y = 97.0 + 19.0 * 2.0 * f * (1.0 - f) * 0.9;
        fill(
            pm,
            t,
            poly(&[
                (x - 2.5, y + 2.0),
                (x + 2.5, y + 2.0),
                (x + 1.5, y + 11.0),
                (x - 1.5, y + 8.0),
            ]),
            &solid(rgba("#f4fff8", 0.85)),
        );
    }
    // inner portal
    rect(
        pm,
        t,
        cx - 87.0,
        91.0,
        174.0,
        123.0,
        &radial(
            cx,
            150.0,
            96.0,
            &[
                (0.0, "#d8fffa", 0.6),
                (0.5, "#5ac8c8", 0.3),
                (1.0, "#2a8898", 0.0),
            ],
        ),
    );
    grain(
        pm,
        t.sx,
        (cx - 160.0, 24.0, 320.0, 190.0),
        4.0,
        4.0,
        0.35,
        3.0,
    );
    blobs(
        pm,
        t.sx,
        (cx - 160.0, 24.0, 320.0, 190.0),
        0.5,
        1.8,
        3.0,
        0.6,
        0.2,
        "#10303a",
        0.45,
        0.0,
        true,
    );
    // moss
    blobs(
        pm,
        t.sx,
        (cx - 160.0, 150.0, 320.0, 64.0),
        0.6,
        1.6,
        9.0,
        0.5,
        0.25,
        "#2a7a5a",
        0.4,
        0.0,
        true,
    );
}
fn toro(pm: &mut Pixmap, t: Transform, x: f32, by: f32) {
    let c = lin(by - 50.0, by, &[(0.0, "#4a7078"), (1.0, "#1a343a")]);
    rect(pm, t, x - 5.0, by - 12.0, 10.0, 12.0, &c);
    rect(pm, t, x - 2.5, by - 26.0, 5.0, 16.0, &c);
    rect(pm, t, x - 8.0, by - 30.0, 16.0, 4.0, &c);
    rect(
        pm,
        t,
        x - 6.0,
        by - 42.0,
        12.0,
        12.0,
        &solid(rgba("#142a30", 1.0)),
    );
    rect(
        pm,
        t,
        x - 4.0,
        by - 40.0,
        8.0,
        8.0,
        &radial(
            x,
            by - 36.0,
            6.0,
            &[
                (0.0, "#ffffff", 1.0),
                (0.5, "#b8fff0", 1.0),
                (1.0, "#4ad8c8", 1.0),
            ],
        ),
    );
    fill(
        pm,
        t,
        poly(&[(x - 11.0, by - 42.0), (x, by - 52.0), (x + 11.0, by - 42.0)]),
        &c,
    );
    fill(pm, t, circle(x, by - 54.0, 1.8), &c);
}
fn spirit() -> Stage {
    let back = vec![
        // 0 night sky (aurora is animated)
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.04,
            40,
            0.04,
            &|pm, t, w, h, r| {
                rect(
                    pm,
                    t,
                    0.0,
                    0.0,
                    w,
                    h,
                    &lin(
                        0.0,
                        h,
                        &[(0.0, "#050c18"), (0.45, "#12363e"), (1.0, "#7ac0b8")],
                    ),
                );
                for _ in 0..120 {
                    let a = r.f() * 0.8;
                    fill(
                        pm,
                        t,
                        circle(r.f() * w, r.f() * 150.0, r.f() * 1.1),
                        &solid(rgba("#c8fff4", a)),
                    );
                }
                glowc(pm, t, w * 0.62, 70.0, 80.0, "#a8fff0", 0.45);
                fill(
                    pm,
                    t,
                    circle(w * 0.62, 70.0, 26.0),
                    &radial(
                        w * 0.62 - 6.0,
                        64.0,
                        30.0,
                        &[
                            (0.0, "#ffffff", 1.0),
                            (0.7, "#d8fff8", 1.0),
                            (1.0, "#9ad8d0", 1.0),
                        ],
                    ),
                );
                blobs(
                    pm,
                    t.sx,
                    (0.0, 30.0, w, 160.0),
                    0.6,
                    2.6,
                    3.0,
                    0.58,
                    0.25,
                    "#0a2630",
                    0.5,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    t.sx,
                    (0.0, 30.0, w, 160.0),
                    0.6,
                    2.6,
                    3.0,
                    0.67,
                    0.2,
                    "#a8fff0",
                    0.2,
                    0.0,
                    false,
                );
            },
        ),
        // 1 far mountains + mist
        lay(
            2.0,
            false,
            760.0,
            240.0,
            0.0,
            0.14,
            45,
            0.14,
            &|pm, t, w, _, _| {
                ridge(pm, t, w, 150.0, 70.0, "#1c4a52", "#3c8a88", 5.0, 0.5);
                ridge(pm, t, w, 180.0, 46.0, "#143a42", "#2a6a6c", 6.0, 0.8);
                haze(pm, t.sx, (0.0, 120.0, w, 120.0), "#9ad8d0", 0.0, 0.6);
                blobs(
                    pm,
                    t.sx,
                    (0.0, 130.0, w, 90.0),
                    0.4,
                    3.0,
                    2.0,
                    0.5,
                    0.3,
                    "#c8fff4",
                    0.3,
                    0.0,
                    true,
                );
            },
        ),
        // 2 the great gate
        lay(
            3.5,
            false,
            800.0,
            240.0,
            0.0,
            0.25,
            41,
            0.25,
            &|pm, t, w, _, _| {
                // spirit forest silhouette behind the gate
                for i in 0..22 {
                    let x = i as f32 * 38.0 + hash(i, 1) * 16.0;
                    pine(pm, t, x, 214.0, 70.0 + hash(i, 2) * 50.0, "#0c2228");
                }
                gate(pm, t, w / 2.0);
                haze(pm, t.sx, (0.0, 170.0, w, 60.0), "#7ad8d0", 0.0, 0.5);
            },
        ),
        // 3 stone lantern row, twisted spirit trees
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.5,
            43,
            0.5,
            &|pm, t, w, _, _| {
                for (i, x) in toro_xs().into_iter().enumerate() {
                    toro(pm, t, x, 212.0);
                    if i % 2 == 1 {
                        let tx = x + 30.0;
                        let bark = solid(rgba("#0c1e22", 1.0));
                        let mut b = PathBuilder::new();
                        b.move_to(tx, 214.0);
                        b.quad_to(tx - 6.0, 170.0, tx + 4.0, 140.0);
                        b.quad_to(tx + 10.0, 120.0, tx - 2.0, 100.0);
                        stroke(pm, t, b.finish(), &bark, 5.0);
                        for k in 0..4 {
                            let a = -1.2 + k as f32 * 0.8;
                            stroke(
                                pm,
                                t,
                                line(
                                    tx + 2.0,
                                    130.0,
                                    tx + 2.0 + a.cos() * 28.0,
                                    130.0 + a.sin() * 24.0,
                                ),
                                &bark,
                                2.0,
                            );
                        }
                    }
                }
                grain(pm, t.sx, (0.0, 90.0, w, 130.0), 4.0, 4.0, 0.3, 3.0);
                haze(pm, t.sx, (0.0, 160.0, w, 60.0), "#6ad0c8", 0.0, 0.3);
            },
        ),
        // 4 polished stone floor with carved runes
        layer(1100.0, 120.0, 175.0, 1.0, 44, 1.0, &|pm, t, w, h, r| {
            let rr = t.sx;
            rect(
                pm,
                t,
                0.0,
                28.0,
                w,
                h,
                &lin(
                    28.0,
                    h,
                    &[(0.0, "#a8c0bc"), (0.2, "#6a8684"), (1.0, "#2a3c3c")],
                ),
            );
            // stone slabs
            let n = 7;
            for i in 0..n {
                let (y0, y1) = (
                    30.0 + (i as f32 / n as f32).powf(1.5) * 90.0,
                    30.0 + ((i + 1) as f32 / n as f32).powf(1.5) * 90.0,
                );
                let sw = 70.0 * (0.7 + i as f32 * 0.25);
                let mut x = -(hash(i, 5) * sw);
                while x < w {
                    let tone = hash((x / 7.0) as u32, i + 9);
                    rect(
                        pm,
                        t,
                        x + 0.4,
                        y0 + 0.2,
                        sw - 0.8,
                        y1 - y0 - 0.4,
                        &solid(rgba(
                            if tone < 0.5 { "#aac4c0" } else { "#8aa8a6" },
                            0.18 * (1.0 + i as f32 * 0.1),
                        )),
                    );
                    rect(pm, t, x, y0, 0.9, y1 - y0, &solid(rgba("#10262a", 0.7)));
                    x += sw;
                }
                rect(pm, t, 0.0, y1 - 0.4, w, 0.9, &solid(rgba("#10262a", 0.7)));
                rect(pm, t, 0.0, y0 + 0.5, w, 0.6, &solid(rgba("#e8fffa", 0.28)));
            }
            grain(pm, rr, (0.0, 28.0, w, h - 28.0), 6.0, 6.0, 0.5, 2.0);
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.25,
                1.1,
                4.0,
                0.55,
                0.3,
                "#16302e",
                0.5,
                0.0,
                true,
            );
            blobs(
                pm,
                rr,
                (0.0, 28.0, w, h - 28.0),
                0.45,
                1.6,
                9.0,
                0.6,
                0.25,
                "#2a8a6a",
                0.35,
                0.0,
                true,
            );
            for _ in 0..24 {
                let x = r.f() * w;
                let y = 34.0 + r.f() * 80.0;
                let mut b = PathBuilder::new();
                b.move_to(x, y);
                b.line_to(x + (r.f() - 0.5) * 20.0, y + (r.f() - 0.5) * 6.0);
                stroke(pm, t, b.finish(), &solid(rgba("#0a1c20", 0.6)), 0.5);
            }
            // rune glyph rings every 270 units
            for k in 0..4 {
                let cx = 120.0 + k as f32 * 290.0;
                let cy = 74.0;
                for (rx, ry) in [(52.0, 11.0), (36.0, 7.5)] {
                    stroke(
                        pm,
                        t,
                        ellipse(cx, cy, rx, ry),
                        &solid(rgba("#5affea", 0.28)),
                        0.8,
                    );
                }
                for q in 0..12 {
                    let a = q as f32 / 12.0 * std::f32::consts::TAU;
                    let (x, y) = (cx + a.cos() * 44.0, cy + a.sin() * 9.0);
                    stroke(
                        pm,
                        t,
                        line(x - 1.5, y - 1.0, x + 1.5, y + 1.0 + (q % 3) as f32),
                        &solid(rgba("#7affee", 0.5)),
                        0.7,
                    );
                }
            }
            shade_v(pm, rr, (0.0, 28.0, w, h - 28.0), 0.86, 1.0);
            // top: low stone wall + steps
            rect(
                pm,
                t,
                0.0,
                12.0,
                w,
                18.0,
                &lin(12.0, 30.0, &[(0.0, "#5a7a78"), (1.0, "#1c3030")]),
            );
            let mut x = 0.0;
            while x < w {
                rect(pm, t, x, 12.0, 0.8, 18.0, &solid(rgba("#0a1c20", 0.7)));
                x += 30.0 + hash((x / 30.0) as u32, 6) * 24.0;
            }
            rect(pm, t, 0.0, 12.0, w, 1.6, &solid(rgba("#d8fff8", 0.4)));
            grain(pm, rr, (0.0, 12.0, w, 18.0), 5.0, 5.0, 0.5, 4.0);
        }),
        // 5 dynamic foreground: hanging charms
        dynamic(560.0, 240.0, 0.0, 1.2, 1.0, true),
    ];
    let particles = parts(6, 22, |r| Particle {
        x: r.f() * 470.0,
        y: r.f() * 240.0,
        vx: (r.f() - 0.5) * 0.2,
        vy: -0.3 - r.f() * 0.4,
        s: 0.7 + r.f(),
        a: 0.5,
        c: [184, 255, 240],
    });
    Stage {
        info: info("Portão dos Espíritos"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: None,
        },
        music: 4,
        kind: StageKind::Spirit,
        bgm: None,
    }
}

// ───────────────────────────────────────────── VOLCANO ─────────────────────────────────────────────

fn basalt(pm: &mut Pixmap, t: Transform, r: &mut Rng, x: f32, by: f32, w: f32, h: f32, glow: bool) {
    let n = 4;
    let cw = w / n as f32;
    for i in 0..n {
        let hh = h * (0.55 + r.f() * 0.45);
        let cx = x + i as f32 * cw;
        fill(
            pm,
            t,
            poly(&[
                (cx, by),
                (cx + 1.0, by - hh),
                (cx + cw * 0.5, by - hh - 3.0),
                (cx + cw, by - hh + 1.0),
                (cx + cw + 1.0, by),
            ]),
            &lin_h(
                cx,
                cx + cw,
                &[
                    (0.0, "#3a1a14", 1.0),
                    (0.5, "#241008", 1.0),
                    (1.0, "#0e0504", 1.0),
                ],
            ),
        );
        rect(
            pm,
            t,
            cx,
            by - hh,
            1.0,
            hh,
            &solid(rgba("#ff7a30", if glow { 0.7 } else { 0.35 })),
        );
        stroke(
            pm,
            t,
            line(cx + cw * 0.5, by - hh - 3.0, cx + cw, by - hh + 1.0),
            &solid(rgba("#ff9a50", 0.5)),
            0.8,
        );
        for k in 1..4 {
            rect(
                pm,
                t,
                cx + 1.0,
                by - hh * k as f32 / 4.0,
                cw - 1.5,
                0.6,
                &solid(rgba("#0a0302", 0.6)),
            );
        }
    }
}
fn volcano() -> Stage {
    let cracks = lava_cracks(1100.0);
    let back = vec![
        // 0 ash sky glowing from below
        lay(
            1.6,
            false,
            520.0,
            240.0,
            0.0,
            0.04,
            50,
            0.04,
            &|pm, t, w, h, _| {
                rect(
                    pm,
                    t,
                    0.0,
                    0.0,
                    w,
                    h,
                    &lin(
                        0.0,
                        h,
                        &[
                            (0.0, "#080202"),
                            (0.45, "#3a0c06"),
                            (0.8, "#a82a10"),
                            (1.0, "#e8581e"),
                        ],
                    ),
                );
                let r = t.sx;
                blobs(
                    pm,
                    r,
                    (0.0, 0.0, w, h),
                    0.6,
                    2.2,
                    1.0,
                    0.5,
                    0.3,
                    "#140404",
                    0.75,
                    0.0,
                    false,
                );
                blobs(
                    pm,
                    r,
                    (0.0, 60.0, w, 180.0),
                    0.7,
                    2.6,
                    6.0,
                    0.6,
                    0.2,
                    "#ff6a1e",
                    0.35,
                    0.0,
                    false,
                );
            },
        ),
        // 1 far volcano cone with crater glow and lava streaks
        lay(
            2.0,
            false,
            760.0,
            240.0,
            0.0,
            0.14,
            51,
            0.14,
            &|pm, t, w, _, r| {
                ridge(pm, t, w, 190.0, 50.0, "#2c0c08", "#4a1810", 2.0, 0.6);
                let cx = w * 0.55;
                fill(
                    pm,
                    t,
                    poly(&[
                        (cx - 190.0, 215.0),
                        (cx - 40.0, 96.0),
                        (cx - 18.0, 90.0),
                        (cx + 18.0, 90.0),
                        (cx + 44.0, 98.0),
                        (cx + 200.0, 215.0),
                    ]),
                    &lin(90.0, 215.0, &[(0.0, "#2a0a06"), (1.0, "#4a1a10")]),
                );
                glowc(pm, t, cx, 90.0, 80.0, "#ff7a2a", 0.9);
                fill(
                    pm,
                    t,
                    ellipse(cx, 91.0, 20.0, 4.0),
                    &solid(rgba("#ffd070", 1.0)),
                );
                for k in 0..9 {
                    let (mut x, mut y) = (cx + (r.f() - 0.5) * 24.0, 92.0);
                    let mut b = PathBuilder::new();
                    b.move_to(x, y);
                    for _ in 0..5 {
                        x += (r.f() - 0.5) * 22.0 + (k as f32 - 4.0) * 3.0;
                        y += 14.0 + r.f() * 10.0;
                        b.line_to(x, y);
                    }
                    let p = b.finish();
                    stroke(pm, t, p.clone(), &solid(rgba("#ff5000", 0.35)), 3.0);
                    stroke(pm, t, p, &solid(rgba("#ffa040", 0.95)), 0.9);
                }
                grain(pm, t.sx, (0.0, 80.0, w, 140.0), 3.0, 3.0, 0.5, 2.0);
                haze(pm, t.sx, (0.0, 150.0, w, 90.0), "#c8381a", 0.0, 0.5);
            },
        ),
        // 2 lava river + jagged ridge
        lay(
            3.5,
            false,
            1000.0,
            240.0,
            0.0,
            0.45,
            52,
            0.45,
            &|pm, t, w, _, r| {
                ridge(pm, t, w, 196.0, 56.0, "#1c0806", "#2c0e0a", 7.0, 0.9);
                let mut x = 0.0;
                while x < w {
                    let hh = 30.0 + r.f() * 60.0;
                    fill(
                        pm,
                        t,
                        poly(&[
                            (x, 214.0),
                            (x + 12.0, 204.0 - hh),
                            (x + 16.0, 207.0 - hh),
                            (x + 32.0, 214.0),
                        ]),
                        &lin(205.0 - hh, 214.0, &[(0.0, "#2a0e0a"), (1.0, "#120504")]),
                    );
                    stroke(
                        pm,
                        t,
                        line(x + 12.0, 204.0 - hh, x + 16.0, 207.0 - hh),
                        &solid(rgba("#ff8a40", 0.6)),
                        0.9,
                    );
                    x += 40.0 + r.f() * 60.0;
                }
                // lava river band
                let mut b = PathBuilder::new();
                b.move_to(0.0, 214.0);
                let mut xx = 0.0;
                while xx <= w {
                    b.line_to(xx, 200.0 + n1(xx * 0.3, 9.0) * 8.0);
                    xx += 4.0;
                }
                b.line_to(w, 220.0);
                b.line_to(0.0, 220.0);
                b.close();
                fill(
                    pm,
                    t,
                    b.finish(),
                    &lin(
                        196.0,
                        220.0,
                        &[(0.0, "#ffe070"), (0.4, "#ff8a1e"), (1.0, "#b82a0a")],
                    ),
                );
                blobs(
                    pm,
                    t.sx,
                    (0.0, 195.0, w, 25.0),
                    0.5,
                    3.0,
                    4.0,
                    0.55,
                    0.25,
                    "#3a0a04",
                    0.5,
                    0.0,
                    true,
                );
                glowc(pm, t, w * 0.25, 204.0, 70.0, "#ff6a1e", 0.5);
                glowc(pm, t, w * 0.75, 204.0, 70.0, "#ff6a1e", 0.5);
                grain(pm, t.sx, (0.0, 100.0, w, 120.0), 3.0, 3.0, 0.4, 7.0);
            },
        ),
        // 3 near basalt pillars with glowing seams
        lay(
            3.5,
            false,
            1100.0,
            240.0,
            0.0,
            0.75,
            54,
            0.75,
            &|pm, t, w, _, r| {
                let mut x = 20.0;
                while x < w {
                    let ww = 34.0 + r.f() * 40.0;
                    let (bh, gl) = (34.0 + r.f() * 60.0, r.f() < 0.5);
                    basalt(pm, t, r, x, 212.0, ww, bh, gl);
                    x += ww + 70.0 + r.f() * 160.0;
                }
                glowc(pm, t, w * 0.5, 214.0, 140.0, "#ff6a1e", 0.25);
                grain(pm, t.sx, (0.0, 100.0, w, 120.0), 4.0, 4.0, 0.45, 8.0);
            },
        ),
        // 4 obsidian slab floor with glowing seams
        layer(
            1100.0,
            120.0,
            175.0,
            1.0,
            53,
            1.0,
            &move |pm, t, w, h, r| {
                let rr = t.sx;
                rect(
                    pm,
                    t,
                    0.0,
                    28.0,
                    w,
                    h,
                    &lin(
                        28.0,
                        h,
                        &[(0.0, "#3a1c16"), (0.35, "#1c0a08"), (1.0, "#0a0302")],
                    ),
                );
                // hexagonal basalt slabs
                let n = 6;
                for i in 0..n {
                    let (y0, y1) = (
                        30.0 + (i as f32 / n as f32).powf(1.5) * 90.0,
                        30.0 + ((i + 1) as f32 / n as f32).powf(1.5) * 90.0,
                    );
                    let sw = 60.0 * (0.7 + i as f32 * 0.3);
                    let mut x = -hash(i, 4) * sw;
                    while x < w {
                        let tone = hash((x / 5.0) as u32, i);
                        rect(
                            pm,
                            t,
                            x + 0.5,
                            y0 + 0.3,
                            sw - 1.0,
                            y1 - y0 - 0.6,
                            &lin(
                                y0,
                                y1,
                                &[
                                    (0.0, if tone < 0.5 { "#4a2a24" } else { "#3a201c" }),
                                    (1.0, "#160807"),
                                ],
                            ),
                        );
                        rect(
                            pm,
                            t,
                            x + 0.5,
                            y0 + 0.3,
                            sw - 1.0,
                            0.8,
                            &solid(rgba("#ffb080", 0.18)),
                        );
                        x += sw;
                    }
                }
                grain(pm, rr, (0.0, 28.0, w, h - 28.0), 6.0, 6.0, 0.7, 2.0);
                blobs(
                    pm,
                    rr,
                    (0.0, 28.0, w, h - 28.0),
                    0.3,
                    1.0,
                    4.0,
                    0.6,
                    0.3,
                    "#000000",
                    0.5,
                    0.0,
                    true,
                );
                for pl in &cracks {
                    let mut b = PathBuilder::new();
                    b.move_to(pl[0].0, pl[0].1);
                    for p in &pl[1..] {
                        b.line_to(p.0, p.1);
                    }
                    let path = b.finish();
                    stroke(pm, t, path.clone(), &solid(rgba("#ff4a00", 0.22)), 4.0);
                    stroke(pm, t, path.clone(), &solid(rgba("#ff8a30", 0.7)), 1.5);
                    stroke(pm, t, path, &solid(rgba("#ffe090", 0.95)), 0.5);
                }
                shade_v(pm, rr, (0.0, 28.0, w, h - 28.0), 0.9, 1.0);
                // far edge: cooled lava ridge
                rect(
                    pm,
                    t,
                    0.0,
                    6.0,
                    w,
                    24.0,
                    &lin(6.0, 30.0, &[(0.0, "#1c0a08"), (1.0, "#3a1a14")]),
                );
                let mut x = 0.0;
                while x < w {
                    let hh = 6.0 + r.f() * 14.0;
                    fill(
                        pm,
                        t,
                        poly(&[
                            (x, 30.0),
                            (x + 3.0, 30.0 - hh),
                            (x + 9.0 + r.f() * 6.0, 30.0),
                        ]),
                        &solid(rgba("#140807", 1.0)),
                    );
                    x += 7.0;
                }
                rect(pm, t, 0.0, 28.5, w, 1.4, &solid(rgba("#ff7a30", 0.8)));
            },
        ),
        // 5 front: basalt silhouettes rimmed with lava light
        lay(
            3.5,
            true,
            560.0,
            240.0,
            0.0,
            1.25,
            55,
            1.0,
            &|pm, t, w, h, r| {
                basalt(pm, t, r, -10.0, h + 4.0, 56.0, 84.0, true);
                basalt(pm, t, r, w - 50.0, h + 4.0, 56.0, 96.0, true);
            },
        ),
    ];
    let particles = parts(8, 36, |r| Particle {
        x: r.f() * 470.0,
        y: r.f() * 240.0,
        vx: (r.f() - 0.5) * 0.4,
        vy: -0.5 - r.f() * 0.8,
        s: 0.5 + r.f() * 0.8,
        a: 0.9,
        c: if r.f() < 0.5 {
            [255, 176, 48]
        } else {
            [255, 90, 16]
        },
    });
    Stage {
        info: info("Cratera Vulcânica"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: Some([1.0, 0.24, 0.0, 0.05]),
        },
        music: 5,
        kind: StageKind::Volcano,
        bgm: None,
    }
}

pub mod ambience;
mod wonders;

const ILLUSTRATION_IDS: [&str; 7] = [
    "dojo", "neon", "bamboo", "plains", "spirit", "volcano", "brazil",
];

fn decode_illustration(bytes: &[u8]) -> Result<Arc<ImageData>, String> {
    let png = ume_formats::png::decode_png(bytes).map_err(|e| e.to_string())?;
    let rgba = png.rgba.ok_or("expected truecolor stage PNG")?;
    Ok(ImageData::new(png.width, png.height, rgba))
}

pub fn illustration_preview(id: &str) -> Option<Arc<ImageData>> {
    if let Some(image) = wonders::illustration_preview(id) {
        return Some(image);
    }
    use std::sync::OnceLock;
    static IMAGES: [OnceLock<Arc<ImageData>>; 7] = [const { OnceLock::new() }; 7];
    let index = ILLUSTRATION_IDS.iter().position(|name| *name == id)?;
    let bytes: [&[u8]; 7] = [
        include_bytes!("../assets/stages/thumbs/dojo.png"),
        include_bytes!("../assets/stages/thumbs/neon.png"),
        include_bytes!("../assets/stages/thumbs/bamboo.png"),
        include_bytes!("../assets/stages/thumbs/plains.png"),
        include_bytes!("../assets/stages/thumbs/spirit.png"),
        include_bytes!("../assets/stages/thumbs/volcano.png"),
        include_bytes!("../assets/stages/thumbs/brazil.png"),
    ];
    Some(
        IMAGES[index]
            .get_or_init(|| decode_illustration(bytes[index]).expect("embedded stage thumbnail"))
            .clone(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub fn illustration(id: &str) -> Option<Arc<ImageData>> {
    if let Some(image) = wonders::illustration(id) {
        return Some(image);
    }
    use std::sync::OnceLock;
    static IMAGES: [OnceLock<Arc<ImageData>>; 7] = [const { OnceLock::new() }; 7];
    let index = ILLUSTRATION_IDS.iter().position(|name| *name == id)?;
    let bytes: [&[u8]; 7] = [
        include_bytes!("../assets/stages/dojo.png"),
        include_bytes!("../assets/stages/neon.png"),
        include_bytes!("../assets/stages/bamboo.png"),
        include_bytes!("../assets/stages/plains.png"),
        include_bytes!("../assets/stages/spirit.png"),
        include_bytes!("../assets/stages/volcano.png"),
        include_bytes!("../assets/stages/brazil.png"),
    ];
    Some(
        IMAGES[index]
            .get_or_init(|| decode_illustration(bytes[index]).expect("embedded stage PNG"))
            .clone(),
    )
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_ILLUSTRATION: std::cell::RefCell<Option<(String, Arc<ImageData>)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
pub fn illustration(id: &str) -> Option<Arc<ImageData>> {
    WEB_ILLUSTRATION.with(|cache| {
        cache
            .borrow()
            .as_ref()
            .filter(|(key, _)| key == id)
            .map(|(_, image)| image.clone())
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn prepare_illustration(id: &str) -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        fn ume_fetch_asset(path: &str) -> js_sys::Promise;
    }
    if illustration(id).is_some() {
        return Ok(());
    }
    if !ILLUSTRATION_IDS.contains(&id) && !wonders::WONDER_IDS.contains(&id) {
        return Err(format!("Unknown stage: {id}"));
    }
    let data = js_sys::futures::JsFuture::from(ume_fetch_asset(&format!("assets/stages/{id}.png")))
        .await
        .map_err(|e| format!("Stage download failed: {e:?}"))?;
    let image = decode_illustration(&js_sys::Uint8Array::new(&data).to_vec())?;
    WEB_ILLUSTRATION.with(|cache| *cache.borrow_mut() = Some((id.to_string(), image)));
    Ok(())
}

fn illustrated(id: &str, name: &str, music: u32, kind: StageKind) -> Stage {
    Stage {
        info: info(name),
        art: StageArt::Illustrated {
            image: illustration(id).expect("builtin stage image"),
            ambience: ambience::profile_for(id),
        },
        music,
        kind,
        bgm: None,
    }
}

pub struct StageEntry {
    pub id: String,
    pub name: String,
    pub thumb: [[u8; 3]; 3],
    pub build: Box<dyn Fn() -> Stage + Send + Sync>,
}

pub fn builtin() -> Vec<StageEntry> {
    let mut stages = vec![
        StageEntry {
            id: "dojo".into(),
            name: "Telhado do Dojo — Pôr do Sol".into(),
            thumb: [[42, 30, 92], [212, 80, 122], [255, 154, 74]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => dojo(),
                _ => illustrated("dojo", "Telhado do Dojo — Pôr do Sol", 0, StageKind::Dojo),
            }),
        },
        StageEntry {
            id: "neon".into(),
            name: "Neo-Metrópole 2099".into(),
            thumb: [[5, 3, 15], [29, 11, 58], [255, 47, 208]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => neon(),
                _ => illustrated("neon", "Neo-Metrópole 2099", 1, StageKind::Neon),
            }),
        },
        StageEntry {
            id: "bamboo".into(),
            name: "Bambuzal ao Luar".into(),
            thumb: [[4, 16, 28], [18, 58, 74], [44, 106, 106]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => bamboo(),
                _ => illustrated("bamboo", "Bambuzal ao Luar", 2, StageKind::Bamboo),
            }),
        },
        StageEntry {
            id: "plains".into(),
            name: "Planalto Rochoso".into(),
            thumb: [[44, 111, 214], [140, 198, 242], [216, 176, 128]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => plains(),
                _ => illustrated("plains", "Planalto Rochoso", 3, StageKind::Plains),
            }),
        },
        StageEntry {
            id: "spirit".into(),
            name: "Portão dos Espíritos".into(),
            thumb: [[10, 20, 32], [29, 74, 82], [154, 208, 200]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => spirit(),
                _ => illustrated("spirit", "Portão dos Espíritos", 4, StageKind::Spirit),
            }),
        },
        StageEntry {
            id: "volcano".into(),
            name: "Cratera Vulcânica".into(),
            thumb: [[18, 4, 4], [74, 14, 8], [200, 56, 26]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => volcano(),
                _ => illustrated("volcano", "Cratera Vulcânica", 5, StageKind::Volcano),
            }),
        },
        StageEntry {
            id: "brazil".into(),
            name: "Praça do Interior — Brasil".into(),
            thumb: [[97, 59, 132], [238, 163, 69], [166, 106, 70]],
            build: Box::new(|| {
                illustrated("brazil", "Praça do Interior — Brasil", 6, StageKind::Custom)
            }),
        },
    ];
    stages.extend(wonders::builtin());
    stages
}

pub fn from_mugen(mut st: MugenStage) -> Stage {
    let scl = 320.0 / if st.local_w > 0.0 { st.local_w } else { 320.0 };
    // eixo Y: cenarios 1280x720 (16:9) precisam de 240/720 para ocupar a altura toda
    let scl_y = 240.0 / if st.local_h > 0.0 { st.local_h } else { 240.0 };
    let info = StageInfo {
        name: st.name.clone(),
        bound_left: st.bound_left * scl,
        bound_right: st.bound_right * scl,
        bound_high: st.bound_high * scl_y,
        bound_low: st.bound_low * scl_y,
        p1x: st.p1x * scl,
        p2x: st.p2x * scl,
        zoffset: st.zoffset * scl_y,
        vertical_follow: st.vertical_follow,
        tension: st.tension * scl,
        start_x: st.start_x * scl,
        start_y: st.start_y * scl_y,
        zoom_out: st.zoom_out,
        zoom_in: st.zoom_in,
        camera: ume_engine::matchs::CameraParams {
            vertical_follow_delta: st.vertical_follow_delta * scl_y,
            y_scroll_speed: st.y_scroll_speed,
            zoom_in_speed: st.zoom_in_speed,
            zoom_out_speed: st.zoom_out_speed,
            lowest_cap: st.lowest_cap,
            dynamic_bound_high: st.dynamic_bound_high,
            bound_high_delta: st.bound_high_delta * scl_y,
        },
        auto_turn: st.auto_turn,
    };
    let images = st
        .sprites
        .iter_mut()
        .filter(|(_, im)| im.w > 0 && im.h > 0)
        .map(|(k, im)| (*k, ImageData::new(im.w, im.h, std::mem::take(&mut im.rgba))))
        .collect();
    Stage {
        info,
        art: StageArt::Mugen(Box::new(MugenBg {
            stage: Arc::new(st),
            images,
            scl,
            scl_y,
        })),
        music: 0,
        kind: StageKind::Custom,
        bgm: None,
    }
}

pub fn mugen_preview(st: &MugenStage) -> Option<Arc<ImageData>> {
    if st.layers.is_empty() || st.sprites.is_empty() {
        return None;
    }
    const W: usize = 320;
    const H: usize = 180;
    let (vw, vh) = (W as f32, H as f32);
    let scl = vw / if st.local_w > 0.0 { st.local_w } else { 320.0 };
    let scl_y = vh / if st.local_h > 0.0 { st.local_h } else { 240.0 };
    let (cx, cy) = (st.start_x, st.start_y);
    let mut rgba = vec![0u8; W * H * 4];
    for p in rgba.as_chunks_mut::<4>().0 {
        p.copy_from_slice(&[20, 20, 30, 255]);
    }
    let mut drawn = false;
    let tiles = |t: i32, p0: f32, span: f32, step: f32, lim: f32| -> Vec<f32> {
        match t {
            1 => {
                let st = p0.rem_euclid(step) - step;
                if !st.is_finite() {
                    return vec![];
                }
                let (mut v, mut p) = (vec![], st);
                while p < lim && v.len() <= W + 2 {
                    if p + span > 0.0 {
                        v.push(p);
                    }
                    p += step;
                }
                v
            }
            n if n > 1 => {
                let i0 = (((-span - p0) / step).floor() as i64).clamp(0, n as i64);
                let i1 = (((lim - p0) / step).ceil() as i64)
                    .clamp(i0, (i0 + W as i64 + 2).min(n as i64));
                (i0..i1).map(|i| p0 + i as f32 * step).collect()
            }
            _ => vec![p0],
        }
    };
    for front in [false, true] {
        let mut root_off = [0.0f32; 2];
        let mut root_vis = true;
        for l in &st.layers {
            let mut off = [0.0f32; 2];
            let mut vis = true;
            let mut anim_override = None;
            for def in &st.ctrl_defs {
                for c in &def.ctrls {
                    if c.start <= 0 && 0 <= c.end && (c.ids.is_empty() || c.ids.contains(&l.id)) {
                        match c.kind {
                            ume_formats::stage::BgCtrlKind::Visible(v) => vis = v,
                            ume_formats::stage::BgCtrlKind::PosSet(x, y) => {
                                if let Some(x) = x {
                                    off[0] = x - l.start[0];
                                }
                                if let Some(y) = y {
                                    off[1] = y - l.start[1];
                                }
                            }
                            ume_formats::stage::BgCtrlKind::PosAdd(x, y) => {
                                if let Some(x) = x {
                                    off[0] += x;
                                }
                                if let Some(y) = y {
                                    off[1] += y;
                                }
                            }
                            ume_formats::stage::BgCtrlKind::Anim(a) => anim_override = Some(a),
                            _ => {}
                        }
                    }
                }
            }
            if l.position_link {
                off[0] += root_off[0];
                off[1] += root_off[1];
                vis &= root_vis;
            } else {
                root_off = off;
                root_vis = vis;
            }
            if (l.layer == 1) != front || !vis {
                continue;
            }
            let scroll = l.window_scroll();
            let (wx0, wy0, wx1, wy1) = match l.clip_window() {
                Some(win) if win[2] <= win[0] || win[3] <= win[1] => continue,
                Some(win) => (
                    ((win[0] - cx * scroll[0]) * scl + vw * 0.5)
                        .floor()
                        .max(0.0) as isize,
                    ((win[1] - cy * scroll[1]) * scl_y).floor().max(0.0) as isize,
                    (((win[2] + 1.0) - cx * scroll[0]) * scl + vw * 0.5)
                        .ceil()
                        .min(vw) as isize,
                    (((win[3] + 1.0) - cy * scroll[1]) * scl_y).ceil().min(vh) as isize,
                ),
                None => (0, 0, W as isize, H as isize),
            };
            let anim = anim_override
                .and_then(|a| st.anims.get(&a))
                .or(l.anim.as_ref());
            let key = anim
                .and_then(|a| a.frames.first())
                .map(|f| (f.group, f.index))
                .unwrap_or(l.sprite);
            let Some(meta) = st.sprites.get(&key) else {
                continue;
            };
            let (mw, mh) = (meta.w as usize, meta.h as usize);
            if mw == 0
                || mh == 0
                || mw
                    .checked_mul(mh)
                    .and_then(|n| n.checked_mul(4))
                    .is_none_or(|n| meta.rgba.len() < n)
            {
                continue;
            }
            let layer_scale = l.scale_at([cx, cy]);
            let (lx, ly) = (scl * layer_scale[0], scl_y * layer_scale[1]);
            let sxp = (l.start[0] + off[0] - cx * l.delta[0]) * scl + vw * 0.5;
            let syp = (l.start[1] + off[1] - cy * l.delta[1]) * scl_y;
            let (w, h) = (meta.w as f32 * lx, meta.h as f32 * ly);
            if w < 0.5 || h < 0.5 {
                continue;
            }
            let (axis_dx, axis_dy) = (meta.axis_x as f32 * lx, meta.axis_y as f32 * ly);
            let (x0, y0) = (sxp - axis_dx, syp - axis_dy);
            let (stx, sty) = (
                (w + l.tile_spacing[0] * scl).max(1.0),
                (h + l.tile_spacing[1] * scl_y).max(1.0),
            );
            if ![scl, scl_y, sxp, syp, w, h, x0, y0, stx, sty]
                .iter()
                .all(|v| v.is_finite())
            {
                continue;
            }
            let xs = tiles(l.tile[0], x0, w, stx, vw);
            let ys = tiles(l.tile[1], y0, h, sty, vh);
            let add = l.trans.starts_with("add") && l.trans != "addalpha";
            let add1 = l.trans == "add1";
            let sub = l.trans == "sub";
            let addalpha = l.trans == "addalpha";
            let sa = (l.alpha[0] / 256.0).clamp(0.0, 1.0);
            let da = (l.alpha[1] / 256.0).clamp(0.0, 1.0);
            let transformed = l.angle != 0.0 || l.x_shear != 0.0;
            let (sin_a, cos_a) = (-l.angle).to_radians().sin_cos();
            for &tx in &xs {
                for &ty in &ys {
                    let (bx0, bx1, by0, by1) = if transformed {
                        let (pivot_x, pivot_y) = (tx + axis_dx, ty + axis_dy);
                        let mut bounds = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                        for (cxp, cyp) in [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)] {
                            let (rx, ry) = (cxp - axis_dx, cyp - axis_dy);
                            let sheared = rx - l.x_shear * ry;
                            let (qx, qy) = (
                                pivot_x + sheared * cos_a - ry * sin_a,
                                pivot_y + sheared * sin_a + ry * cos_a,
                            );
                            bounds = (
                                bounds.0.min(qx),
                                bounds.1.max(qx),
                                bounds.2.min(qy),
                                bounds.3.max(qy),
                            );
                        }
                        if !bounds.0.is_finite()
                            || !bounds.1.is_finite()
                            || !bounds.2.is_finite()
                            || !bounds.3.is_finite()
                        {
                            continue;
                        }
                        (bounds.0, bounds.1, bounds.2, bounds.3)
                    } else {
                        (tx, tx + w, ty, ty + h)
                    };
                    let dx0 = (bx0.floor() as isize).max(wx0).max(0);
                    let dx1 = (bx1.ceil() as isize).min(wx1).min(W as isize);
                    let dy0 = (by0.floor() as isize).max(wy0).max(0);
                    let dy1 = (by1.ceil() as isize).min(wy1).min(H as isize);
                    if dx0 >= dx1 || dy0 >= dy1 {
                        continue;
                    }
                    for py in dy0..dy1 {
                        for px in dx0..dx1 {
                            let (u, v) = if transformed {
                                let (dx, dy) = (
                                    px as f32 + 0.5 - (tx + axis_dx),
                                    py as f32 + 0.5 - (ty + axis_dy),
                                );
                                let ry = -dx * sin_a + dy * cos_a;
                                let rx = dx * cos_a + dy * sin_a + l.x_shear * ry;
                                ((rx + axis_dx) / w, (ry + axis_dy) / h)
                            } else {
                                ((px as f32 + 0.5 - tx) / w, (py as f32 + 0.5 - ty) / h)
                            };
                            if transformed && (!(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v))
                            {
                                continue;
                            }
                            let sy = ((v.max(0.0) * mh as f32) as usize).min(mh - 1);
                            let sx = ((u.max(0.0) * mw as f32) as usize).min(mw - 1);
                            let s = &meta.rgba[(sy * mw + sx) * 4..][..4];
                            if s[3] == 0 {
                                continue;
                            }
                            let d = &mut rgba[(py as usize * W + px as usize) * 4..][..4];
                            if add1 {
                                for c in 0..3 {
                                    d[c] = (d[c] / 2).saturating_add(s[c]);
                                }
                            } else if add {
                                for c in 0..3 {
                                    d[c] = d[c].saturating_add(s[c]);
                                }
                            } else if sub {
                                for c in 0..3 {
                                    d[c] = d[c].saturating_sub(s[c]);
                                }
                            } else if addalpha {
                                for c in 0..3 {
                                    d[c] = (s[c] as f32 * sa + d[c] as f32 * da).clamp(0.0, 255.0)
                                        as u8;
                                }
                            } else if s[3] == 255 {
                                d[..3].copy_from_slice(&s[..3]);
                            } else {
                                let a = s[3] as u32;
                                let ia = 255 - a;
                                for c in 0..3 {
                                    d[c] = ((s[c] as u32 * a + d[c] as u32 * ia) / 255) as u8;
                                }
                            }
                            drawn = true;
                        }
                    }
                }
            }
        }
    }
    drawn.then(|| ImageData::new(W as u32, H as u32, rgba))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ume_formats::stage::{load_stage, StageImage};
    use ume_formats::vfs::Vfs;

    fn preview_of(layer_extra: &str) -> Vec<u8> {
        let def = format!("[BGdef]\n\n[BG square]\ntype = normal\nspriteno = 0, 0\nstart = 0, 120\n{layer_extra}\n");
        let mut vfs = Vfs::new();
        vfs.insert("stages/t.def".into(), def.into_bytes());
        let mut st = load_stage(&vfs, "stages/t.def").unwrap();
        let rgba = [255u8, 0, 0, 255].repeat(20 * 20);
        st.sprites.insert(
            (0, 0),
            StageImage {
                w: 20,
                h: 20,
                axis_x: 10,
                axis_y: 10,
                rgba,
            },
        );
        mugen_preview(&st).unwrap().rgba.to_vec()
    }

    fn red_pixels(rgba: &[u8]) -> Vec<(usize, usize)> {
        rgba.as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, p)| p[0] == 255 && p[1] == 0)
            .map(|(i, _)| (i % 320, i / 320))
            .collect()
    }

    fn extent(points: &[(usize, usize)]) -> (usize, usize) {
        let xs = points.iter().map(|p| p.0);
        let ys = points.iter().map(|p| p.1);
        (
            xs.clone().max().unwrap() - xs.min().unwrap() + 1,
            ys.clone().max().unwrap() - ys.min().unwrap() + 1,
        )
    }

    #[test]
    fn preview_applies_layer_scale() {
        let base = red_pixels(&preview_of(""));
        let scaled = red_pixels(&preview_of("scalestart = 2, 2"));
        assert_eq!(extent(&base).0, 20);
        assert_eq!(extent(&scaled).0, 40);
        assert!(extent(&scaled).1.abs_diff(extent(&base).1 * 2) <= 2);
        assert!(scaled.len() > base.len() * 3);
    }

    #[test]
    fn preview_applies_angle_and_shear() {
        let base = red_pixels(&preview_of(""));
        let quarter_turn = red_pixels(&preview_of("angle = 90"));
        let rotated = red_pixels(&preview_of("angle = 45"));
        let sheared = red_pixels(&preview_of("xshear = 1"));
        assert!(extent(&quarter_turn).0.abs_diff(extent(&base).1) <= 2);
        assert!(extent(&quarter_turn).1.abs_diff(extent(&base).0) <= 2);
        assert!(extent(&rotated).0 > extent(&base).0 + 2);
        assert!((rotated.len() as f32 - base.len() as f32).abs() < base.len() as f32 * 0.1);
        assert!(extent(&sheared).0 > extent(&base).0 + 4);
        assert!(extent(&sheared).1.abs_diff(extent(&base).1) <= 1);
    }

    #[test]
    fn preview_clips_to_window_intersection() {
        let clipped = red_pixels(&preview_of(
            "window = -10, 100, 10, 140\nmaskwindow = 0, 100, 100, 140",
        ));
        assert_eq!(extent(&clipped).0, 10);
        assert!(clipped.iter().all(|p| p.0 >= 160));
    }
}

#[cfg(test)]
mod illustration_tests {
    use super::*;

    #[test]
    fn thumbnails_are_small_and_cached_per_stage() {
        for id in ILLUSTRATION_IDS.iter().chain(wonders::WONDER_IDS.iter()) {
            let image = illustration_preview(id).unwrap();
            assert_eq!((image.w, image.h), (240, 135));
            assert!(Arc::ptr_eq(&image, &illustration_preview(id).unwrap()));
        }
        assert!(!Arc::ptr_eq(
            &illustration_preview("dojo").unwrap(),
            &illustration_preview("neon").unwrap()
        ));
        assert!(illustration_preview("unknown").is_none());
    }

    #[test]
    fn invalid_stage_image_is_an_error() {
        assert!(decode_illustration(b"invalid image").is_err());
    }
}
