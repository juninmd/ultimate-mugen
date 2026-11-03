//! Original procedural stages (parallax layers rasterized with tiny-skia) + MUGEN stage adapter.
use std::sync::Arc;
use ume_engine::chardef::ImageData;
use ume_engine::matchs::StageInfo;
use ume_engine::raster::*;
use ume_formats::stage::MugenStage;


pub struct Layer { pub image: Arc<ImageData>, pub w: f32, pub h: f32, pub y: f32, pub delta: f32, pub dy: f32 }
#[derive(Clone)]
pub struct Particle { pub x: f32, pub y: f32, pub vx: f32, pub vy: f32, pub s: f32, pub a: f32, pub c: [u8; 3] }

pub struct MugenBg { pub stage: MugenStage, pub images: std::collections::HashMap<(i32, i32), Arc<ImageData>>, pub scl: f32 }

pub enum StageArt {
    Procedural { back: Vec<Layer>, particles: Vec<Particle>, tint: Option<[f32; 4]> },
    Mugen(Box<MugenBg>),
}
pub struct Stage { pub info: StageInfo, pub art: StageArt, pub music: u32 }

struct Rng(u32);
impl Rng { fn f(&mut self) -> f32 { self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223); self.0 as f32 / 4294967296.0 } }

type Painter<'a> = &'a dyn Fn(&mut Pixmap, Transform, f32, f32, &mut Rng);
fn layer(w: f32, h: f32, y: f32, delta: f32, seed: u32, dy: f32, paint: Painter) -> Layer {
    // pixels per game unit follows the art-quality setting (4K → 2x), capped so no layer exceeds 8192 px (GPU texture limit)
    let r = ume_engine::native::rig::art_res().min(8192.0 / w.max(h).max(1.0));
    let mut pm = Pixmap::new(((w * r) as u32).max(1), ((h * r) as u32).max(1)).unwrap();
    let mut rng = Rng(seed);
    paint(&mut pm, Transform::from_scale(r, r), w, h, &mut rng);
    Layer { image: pixmap_to_image(&pm), w, h, y, delta, dy }
}

fn lin(y0: f32, y1: f32, stops: &[(f32, &str)]) -> Paint<'static> {
    let st = stops.iter().map(|(o, c)| GradientStop::new(*o, rgba(c, 1.0))).collect();
    let mut p = Paint::default();
    p.shader = LinearGradient::new(Point::from_xy(0.0, y0), Point::from_xy(0.0, y1), st, SpreadMode::Pad, Transform::identity()).unwrap();
    p.anti_alias = true; p
}
fn rect(pm: &mut Pixmap, t: Transform, x: f32, y: f32, w: f32, h: f32, p: &Paint) {
    if let Some(r) = tiny_skia::Rect::from_xywh(x, y, w.max(0.01), h.max(0.01)) { pm.fill_rect(r, p, t, None); }
}
fn fill(pm: &mut Pixmap, t: Transform, path: Option<tiny_skia::Path>, p: &Paint) { if let Some(pa) = path { pm.fill_path(&pa, p, FillRule::Winding, t, None); } }
fn stroke(pm: &mut Pixmap, t: Transform, path: Option<tiny_skia::Path>, p: &Paint, w: f32) { if let Some(pa) = path { pm.stroke_path(&pa, p, &Stroke { width: w, line_cap: LineCap::Round, ..Default::default() }, t, None); } }
fn ellipse(x: f32, y: f32, rx: f32, ry: f32) -> Option<tiny_skia::Path> { tiny_skia::Rect::from_xywh(x - rx, y - ry, rx * 2.0, ry * 2.0).and_then(PathBuilder::from_oval) }
fn line(x0: f32, y0: f32, x1: f32, y1: f32) -> Option<tiny_skia::Path> { let mut b = PathBuilder::new(); b.move_to(x0, y0); b.line_to(x1, y1); b.finish() }
fn radial(cx: f32, cy: f32, r: f32, stops: &[(f32, &str, f32)]) -> Paint<'static> {
    let st = stops.iter().map(|(o, c, a)| GradientStop::new(*o, rgba(c, *a))).collect();
    let mut p = Paint::default();
    p.shader = RadialGradient::new(Point::from_xy(cx, cy), Point::from_xy(cx, cy), r, st, SpreadMode::Pad, Transform::identity()).unwrap();
    p.anti_alias = true; p
}
fn mountains(pm: &mut Pixmap, t: Transform, w: f32, base: f32, amp: f32, color: &str, r: &mut Rng, step: f32) {
    let mut b = PathBuilder::new();
    b.move_to(0.0, base + 100.0);
    let mut y = base - amp * r.f();
    let mut x = 0.0;
    while x <= w + step { y = (y + (r.f() - 0.5) * amp * 0.8).clamp(base - amp, base); b.line_to(x, y); x += step; }
    b.line_to(w, base + 100.0); b.close();
    fill(pm, t, b.finish(), &solid(rgba(color, 1.0)));
}
fn floor_planks(pm: &mut Pixmap, t: Transform, w: f32, h: f32, c1: &str, c2: &str, lines: Color) {
    rect(pm, t, 0.0, 0.0, w, h, &lin(0.0, h, &[(0.0, c1), (1.0, c2)]));
    let p = solid(lines);
    for i in 0..12 { let y = (i as f32 / 12.0).powf(1.6) * h; stroke(pm, t, line(0.0, y, w, y), &p, 0.6); }
    let mut x = -w;
    while x < w * 2.0 { stroke(pm, t, line(w / 2.0 + (x - w / 2.0) * 0.55, 0.0, w / 2.0 + (x - w / 2.0) * 1.6, h), &p, 0.6); x += 28.0; }
}
fn parts(seed: u32, n: usize, f: impl Fn(&mut Rng) -> Particle) -> Vec<Particle> { let mut r = Rng(seed); (0..n).map(|_| f(&mut r)).collect() }
fn info(name: &str) -> StageInfo { StageInfo { name: name.into(), bound_left: -230.0, bound_right: 230.0, bound_high: -80.0, bound_low: 0.0, p1x: -70.0, p2x: 70.0, zoffset: 205.0, vertical_follow: 0.25, tension: 60.0 } }

fn dojo() -> Stage {
    let back = vec![
        layer(520.0, 240.0, 0.0, 0.05, 3, 0.05, &|pm, t, w, h, _| {
            rect(pm, t, 0.0, 0.0, w, h, &lin(0.0, h, &[(0.0, "#2a1e5c"), (0.45, "#d4507a"), (0.7, "#ff9a4a"), (1.0, "#ffd38a")]));
            fill(pm, t, circle(w / 2.0, 150.0, 70.0), &radial(w / 2.0, 150.0, 70.0, &[(0.0, "#fffad2", 1.0), (0.3, "#ffdc8c", 0.8), (1.0, "#ffa050", 0.0)]));
            fill(pm, t, circle(w / 2.0, 150.0, 22.0), &solid(rgba("#fff0c8", 0.9)));
            for i in 0..6 { rect(pm, t, 0.0, 60.0 + i as f32 * 14.0, w, 3.0, &solid(rgba("#782a5a", 0.25))); }
        }),
        layer(700.0, 240.0, 0.0, 0.18, 7, 0.18, &|pm, t, w, _, r| { mountains(pm, t, w, 175.0, 70.0, "#6a3570", r, 25.0); mountains(pm, t, w, 190.0, 40.0, "#4a2558", r, 18.0); }),
        layer(900.0, 240.0, 0.0, 0.4, 11, 0.4, &|pm, t, _, _, r| {
            let dark = solid(rgba("#2b1736", 1.0));
            for i in 0..9 {
                let x = 40.0 + i as f32 * 100.0 + r.f() * 30.0; let by = 200.0 - r.f() * 15.0; let bw = 60.0 + r.f() * 30.0;
                rect(pm, t, x - bw * 0.35, by - 30.0, bw * 0.7, 30.0, &dark);
                let tiers = 2 + (r.f() * 2.0) as i32;
                for k in 0..tiers {
                    let ty = by - 30.0 - k as f32 * 22.0; let tw = bw * (1.0 - k as f32 * 0.2);
                    let mut b = PathBuilder::new();
                    b.move_to(x - tw / 2.0 - 8.0, ty + 4.0); b.quad_to(x - tw / 4.0, ty - 4.0, x, ty - 12.0); b.quad_to(x + tw / 4.0, ty - 4.0, x + tw / 2.0 + 8.0, ty + 4.0); b.close();
                    fill(pm, t, b.finish(), &dark);
                    rect(pm, t, x - tw * 0.3, ty, tw * 0.6, 18.0, &dark);
                    for q in 0..3 { rect(pm, t, x - tw * 0.22 + q as f32 * tw * 0.18, ty + 6.0, 3.0, 5.0, &solid(rgba("#ffbe5a", 0.6))); }
                }
            }
        }),
        layer(1100.0, 120.0, 175.0, 1.0, 13, 1.0, &|pm, t, w, h, _| {
            rect(pm, t, 0.0, 22.0, w, h, &solid(rgba("#3b2420", 1.0)));
            floor_planks(pm, t.pre_translate(0.0, 30.0), w, h - 30.0, "#a8694a", "#5a3022", rgba("#280f0a", 0.45));
            rect(pm, t, 0.0, 26.0, w, 4.0, &solid(rgba("#5c3a2c", 1.0)));
            let mut x = 10.0;
            while x < w {
                rect(pm, t, x, 0.0, 5.0, 28.0, &solid(rgba("#2a1712", 1.0)));
                fill(pm, t, ellipse(x + 2.5, 8.0, 6.0, 8.0), &solid(rgba("#e64632", 1.0)));
                fill(pm, t, ellipse(x + 2.5, 8.0, 3.0, 5.0), &solid(rgba("#ffe696", 0.55)));
                x += 70.0;
            }
        }),
    ];
    let particles = parts(5, 40, |r| Particle { x: r.f() * 470.0, y: r.f() * 240.0, vx: 0.3 + r.f() * 0.5, vy: 0.25 + r.f() * 0.4, s: 0.8 + r.f() * 1.1, a: 0.6 + r.f() * 0.4, c: if r.f() < 0.5 { [255, 192, 216] } else { [255, 158, 192] } });
    Stage { info: info("Telhado do Dojo — Pôr do Sol"), art: StageArt::Procedural { back, particles, tint: None }, music: 0 }
}

fn neon() -> Stage {
    let back = vec![
        layer(520.0, 240.0, 0.0, 0.05, 2, 0.05, &|pm, t, w, h, r| {
            rect(pm, t, 0.0, 0.0, w, h, &lin(0.0, h, &[(0.0, "#05030f"), (0.6, "#1d0b3a"), (1.0, "#4a1460")]));
            for _ in 0..80 { let a = r.f() * 0.6; rect(pm, t, r.f() * w, r.f() * 120.0, 0.7, 0.7, &solid(rgba("#ffffff", a))); }
            fill(pm, t, circle(w * 0.7, 60.0, 30.0), &solid(rgba("#ff3cc8", 0.15)));
        }),
        layer(760.0, 240.0, 0.0, 0.2, 4, 0.2, &|pm, t, w, _, r| {
            let mut x = 0.0;
            while x < w {
                let bh = 60.0 + r.f() * 110.0; let bw = 16.0 + r.f() * 18.0;
                rect(pm, t, x, 200.0 - bh, bw, bh + 40.0, &solid(rgba("#120a26", 1.0)));
                let mut y = 200.0 - bh + 4.0;
                while y < 200.0 { let mut xx = x + 2.0; while xx < x + bw - 2.0 { if r.f() < 0.3 { let c = if r.f() < 0.5 { rgba("#78c8ff", 0.5) } else { rgba("#ff8cdc", 0.4) }; rect(pm, t, xx, y, 1.6, 2.0, &solid(c)); } xx += 4.0; } y += 5.0; }
                x += 18.0 + r.f() * 14.0;
            }
        }),
        layer(1000.0, 240.0, 0.0, 0.5, 9, 0.5, &|pm, t, w, _, r| {
            let neon = ["#ff2fd0", "#22e8ff", "#ffe23a", "#7d5bff", "#3dff8a"];
            let mut x = 0.0;
            while x < w {
                let bh = 90.0 + r.f() * 90.0; let bw = 30.0 + r.f() * 30.0;
                rect(pm, t, x, 205.0 - bh, bw, bh + 40.0, &solid(rgba("#0b0618", 1.0)));
                if r.f() < 0.8 {
                    let c = neon[(r.f() * 5.0) as usize % 5];
                    let sy = 205.0 - bh + 10.0 + r.f() * 40.0; let sw = 8.0 + r.f() * 6.0; let sh = 20.0 + r.f() * 30.0;
                    let rp = tiny_skia::Rect::from_xywh(x + bw / 2.0 - sw / 2.0, sy, sw, sh).map(PathBuilder::from_rect);
                    stroke(pm, t, rp.clone(), &solid(rgba(c, 0.25)), 4.0);
                    stroke(pm, t, rp, &solid(rgba(c, 1.0)), 1.2);
                    for q in 0..3 { stroke(pm, t, line(x + bw / 2.0 - 2.0, sy + 4.0 + q as f32 * sh / 3.3, x + bw / 2.0 + 2.0, sy + 6.0 + q as f32 * sh / 3.3), &solid(rgba(c, 1.0)), 1.2); }
                }
                x += 40.0 + r.f() * 40.0;
            }
        }),
        layer(1100.0, 120.0, 175.0, 1.0, 12, 1.0, &|pm, t, w, h, _| {
            rect(pm, t, 0.0, 30.0, w, h, &lin(30.0, h, &[(0.0, "#1a1030"), (1.0, "#05020c")]));
            rect(pm, t, 0.0, 28.0, w, 3.0, &solid(rgba("#2a1a48", 1.0)));
            let cs = [rgba("#ff2fd0", 0.25), rgba("#22e8ff", 0.22), rgba("#ffe23a", 0.15)];
            let mut x = 0.0; let mut i = 0;
            while x < w { rect(pm, t, x, 34.0, 3.0 + (i % 7) as f32, 40.0 + (i * 13 % 30) as f32, &solid(cs[i % 3])); x += 23.0; i += 1; }
            let mut x = 0.0;
            while x < w { rect(pm, t, x, 6.0, 1.5, 22.0, &solid(rgba("#33255a", 1.0))); x += 16.0; }
            rect(pm, t, 0.0, 6.0, w, 2.0, &solid(rgba("#4c3a80", 1.0)));
        }),
    ];
    let particles = parts(9, 70, |r| Particle { x: r.f() * 470.0, y: r.f() * 240.0, vx: -0.6, vy: 5.0 + r.f() * 2.0, s: 0.35, a: 0.35, c: [154, 216, 255] });
    Stage { info: info("Neo-Metrópole 2099"), art: StageArt::Procedural { back, particles, tint: None }, music: 1 }
}

fn bamboo() -> Stage {
    let mut back = vec![layer(520.0, 240.0, 0.0, 0.04, 2, 0.04, &|pm, t, w, h, _| {
        rect(pm, t, 0.0, 0.0, w, h, &lin(0.0, h, &[(0.0, "#04101c"), (0.6, "#123a4a"), (1.0, "#2c6a6a")]));
        fill(pm, t, circle(w * 0.35, 55.0, 60.0), &radial(w * 0.35, 55.0, 60.0, &[(0.0, "#f0fffa", 1.0), (0.25, "#c8f0e6", 0.6), (1.0, "#64b4b4", 0.0)]));
        fill(pm, t, circle(w * 0.35, 55.0, 16.0), &solid(rgba("#f2fffa", 1.0)));
    })];
    for (li, d) in [0.2f32, 0.45, 0.75].iter().enumerate() {
        let col = ["#0e3a3a", "#0a2c2a", "#06201c"][li];
        back.push(layer(700.0 + li as f32 * 300.0, 260.0, -10.0, *d, 20 + li as u32, *d, &move |pm, t, w, _, r| {
            let mut x = 0.0;
            while x < w {
                let bw = 3.0 + li as f32 * 2.0 + r.f() * 2.0;
                rect(pm, t, x, 0.0, bw, 260.0, &solid(rgba(col, 1.0)));
                rect(pm, t, x + bw * 0.6, 0.0, bw * 0.25, 260.0, &solid(rgba("#a0e6c8", 0.12)));
                let mut y = 20.0 + r.f() * 30.0;
                while y < 230.0 {
                    rect(pm, t, x - 1.0, y, bw + 2.0, 1.6, &solid(rgba(col, 1.0)));
                    if r.f() < 0.4 { fill(pm, t.pre_translate(x + bw + 8.0, y - 3.0).pre_rotate(-23.0), ellipse(0.0, 0.0, 9.0, 2.0), &solid(rgba(col, 1.0))); }
                    y += 26.0 + r.f() * 10.0;
                }
                x += 10.0 + r.f() * 22.0;
            }
        }));
    }
    back.push(layer(1100.0, 120.0, 175.0, 1.0, 5, 1.0, &|pm, t, w, h, _| {
        rect(pm, t, 0.0, 28.0, w, h, &lin(28.0, h, &[(0.0, "#2e4a3a"), (1.0, "#0c1a14")]));
        for i in 0..6 { rect(pm, t, 0.0, 30.0 + i as f32 * 3.0, w, 1.0, &solid(rgba("#c8fff0", 0.08))); }
        let mut x = 0.0;
        while x < w { let mut b = PathBuilder::new(); b.move_to(x, 30.0); b.line_to(x + 2.0, 22.0 - (x as i32 % 5) as f32); b.line_to(x + 4.0, 30.0); b.close(); fill(pm, t, b.finish(), &solid(rgba("#1b3328", 1.0))); x += 7.0; }
    }));
    let particles = parts(3, 30, |r| Particle { x: r.f() * 470.0, y: 80.0 + r.f() * 140.0, vx: (r.f() - 0.5) * 0.3, vy: (r.f() - 0.5) * 0.2, s: 0.9, a: 0.8, c: [217, 255, 122] });
    Stage { info: info("Bambuzal ao Luar"), art: StageArt::Procedural { back, particles, tint: Some([0.16, 0.47, 0.55, 0.06]) }, music: 2 }
}

fn plains() -> Stage {
    let back = vec![
        layer(520.0, 240.0, 0.0, 0.04, 8, 0.04, &|pm, t, w, h, r| {
            rect(pm, t, 0.0, 0.0, w, h, &lin(0.0, h, &[(0.0, "#2c6fd6"), (0.6, "#8cc6f2"), (1.0, "#e8f2f0")]));
            for _ in 0..9 { let x = r.f() * w; let y = 30.0 + r.f() * 70.0; for k in 0..5 { let rr = 8.0 + r.f() * 6.0; fill(pm, t, circle(x + k as f32 * 9.0, y + (k as f32).sin() * 3.0, rr), &solid(rgba("#ffffff", 0.8))); } }
        }),
        layer(760.0, 240.0, 0.0, 0.15, 31, 0.15, &|pm, t, w, _, r| mountains(pm, t, w, 170.0, 60.0, "#8aa0b8", r, 30.0)),
        layer(1000.0, 240.0, 0.0, 0.4, 33, 0.4, &|pm, t, w, _, r| {
            let mut x = 0.0;
            while x < w {
                let mh = 50.0 + r.f() * 70.0; let mw = 40.0 + r.f() * 50.0;
                let mut b = PathBuilder::new(); b.move_to(x, 205.0); b.line_to(x + 6.0, 200.0 - mh); b.line_to(x + mw - 4.0, 196.0 - mh); b.line_to(x + mw, 205.0); b.close();
                fill(pm, t, b.finish(), &lin(200.0 - mh, 205.0, &[(0.0, "#c98a5a"), (1.0, "#7a4a30")]));
                rect(pm, t, x + 6.0, 200.0 - mh, 6.0, mh, &solid(rgba("#ffe6be", 0.25)));
                let mut y = 205.0 - mh + 10.0;
                while y < 205.0 { stroke(pm, t, line(x + 3.0, y, x + mw - 2.0, y + 1.0), &solid(rgba("#502814", 0.4)), 0.8); y += 9.0; }
                x += 80.0 + r.f() * 90.0;
            }
        }),
        layer(1100.0, 120.0, 175.0, 1.0, 35, 1.0, &|pm, t, w, h, r| {
            rect(pm, t, 0.0, 28.0, w, h, &lin(28.0, h, &[(0.0, "#d8b080"), (1.0, "#8a6040")]));
            for _ in 0..120 { let a = 0.2 + r.f() * 0.3; fill(pm, t, ellipse(r.f() * w, 34.0 + r.f() * 80.0, 1.0 + r.f() * 4.0, 0.6 + r.f()), &solid(rgba("#5a3c28", a))); }
            for _ in 0..4 { let x = r.f() * w; let y = 60.0 + r.f() * 30.0; stroke(pm, t, ellipse(x, y, 30.0, 6.0), &solid(rgba("#462814", 0.5)), 0.8); }
        }),
    ];
    let particles = parts(4, 18, |r| Particle { x: r.f() * 470.0, y: 150.0 + r.f() * 80.0, vx: 0.8 + r.f(), vy: -0.05, s: 0.6, a: 0.4, c: [232, 200, 160] });
    Stage { info: info("Planalto Rochoso"), art: StageArt::Procedural { back, particles, tint: None }, music: 3 }
}

fn spirit() -> Stage {
    let back = vec![
        layer(520.0, 240.0, 0.0, 0.04, 40, 0.04, &|pm, t, w, h, r| {
            rect(pm, t, 0.0, 0.0, w, h, &lin(0.0, h, &[(0.0, "#0a1420"), (0.5, "#1d4a52"), (1.0, "#9ad0c8")]));
            for _ in 0..50 { let a = r.f() * 0.5; fill(pm, t, circle(r.f() * w, r.f() * 140.0, r.f() * 1.2), &solid(rgba("#b4fff0", a))); }
        }),
        layer(800.0, 240.0, 0.0, 0.25, 41, 0.25, &|pm, t, w, _, _| {
            let cx = w / 2.0; let c = solid(rgba("#0d2228", 1.0));
            rect(pm, t, cx - 110.0, 60.0, 22.0, 150.0, &c); rect(pm, t, cx + 88.0, 60.0, 22.0, 150.0, &c);
            let mut b = PathBuilder::new(); b.move_to(cx - 150.0, 52.0); b.quad_to(cx, 30.0, cx + 150.0, 52.0); b.line_to(cx + 140.0, 64.0); b.quad_to(cx, 46.0, cx - 140.0, 64.0); b.close();
            fill(pm, t, b.finish(), &c);
            rect(pm, t, cx - 120.0, 84.0, 240.0, 10.0, &c);
            rect(pm, t, cx - 90.0, 94.0, 180.0, 120.0, &radial(cx, 140.0, 90.0, &[(0.0, "#c8fffa", 0.55), (1.0, "#78dcdc", 0.0)]));
        }),
        layer(1000.0, 240.0, 0.0, 0.5, 43, 0.5, &|pm, t, w, _, r| {
            let mut x = 20.0;
            while x < w {
                rect(pm, t, x, 150.0, 10.0, 60.0, &solid(rgba("#16323a", 1.0))); rect(pm, t, x - 5.0, 145.0, 20.0, 6.0, &solid(rgba("#16323a", 1.0)));
                fill(pm, t, circle(x + 5.0, 140.0, 14.0), &radial(x + 5.0, 140.0, 14.0, &[(0.0, "#a0fff0", 0.9), (1.0, "#a0fff0", 0.0)]));
                x += 60.0 + r.f() * 30.0;
            }
        }),
        layer(1100.0, 120.0, 175.0, 1.0, 44, 1.0, &|pm, t, w, h, _| {
            rect(pm, t, 0.0, 28.0, w, h, &lin(28.0, h, &[(0.0, "#c8d4d0"), (1.0, "#5a6a68")]));
            let p = solid(rgba("#283c3c", 0.35));
            for i in 0..8 { let y = 30.0 + (i as f32 / 8.0).powf(1.5) * 90.0; stroke(pm, t, line(0.0, y, w, y), &p, 0.6); }
            let mut x = 0.0; while x < w { stroke(pm, t, line(x, 30.0, w / 2.0 + (x - w / 2.0) * 1.8, h), &p, 0.6); x += 40.0; }
        }),
    ];
    let particles = parts(6, 35, |r| Particle { x: r.f() * 470.0, y: r.f() * 240.0, vx: (r.f() - 0.5) * 0.2, vy: -0.3 - r.f() * 0.4, s: 0.7 + r.f(), a: 0.5, c: [184, 255, 240] });
    Stage { info: info("Portão dos Espíritos"), art: StageArt::Procedural { back, particles, tint: None }, music: 4 }
}

fn volcano() -> Stage {
    let back = vec![
        layer(520.0, 240.0, 0.0, 0.04, 50, 0.04, &|pm, t, w, h, _| rect(pm, t, 0.0, 0.0, w, h, &lin(0.0, h, &[(0.0, "#120404"), (0.5, "#4a0e08"), (1.0, "#c8381a")]))),
        layer(760.0, 240.0, 0.0, 0.18, 51, 0.18, &|pm, t, w, _, r| {
            mountains(pm, t, w, 180.0, 90.0, "#240806", r, 35.0);
            rect(pm, t, 0.0, 100.0, w, 100.0, &{ let mut p = lin(100.0, 200.0, &[(0.0, "#ff5a14"), (1.0, "#ff781e")]); if let tiny_skia::Shader::LinearGradient(_) = p.shader {} p.blend_mode = tiny_skia::BlendMode::Screen; p });
        }),
        layer(1000.0, 240.0, 0.0, 0.45, 52, 0.45, &|pm, t, w, _, r| {
            let mut x = 0.0;
            while x < w {
                let hh = 30.0 + r.f() * 60.0;
                let mut b = PathBuilder::new(); b.move_to(x, 210.0); b.line_to(x + 15.0, 205.0 - hh); b.line_to(x + 30.0, 210.0); b.close();
                fill(pm, t, b.finish(), &solid(rgba("#1a0605", 1.0)));
                let mut b = PathBuilder::new(); b.move_to(x + 15.0, 210.0 - hh); b.line_to(x + 12.0, 205.0 - hh / 2.0); b.line_to(x + 17.0, 208.0);
                let path = b.finish();
                stroke(pm, t, path.clone(), &solid(rgba("#ff4a00", 0.3)), 3.0);
                stroke(pm, t, path, &solid(rgba("#ff6a1a", 1.0)), 0.8);
                x += 50.0 + r.f() * 70.0;
            }
        }),
        layer(1100.0, 120.0, 175.0, 1.0, 53, 1.0, &|pm, t, w, h, r| {
            rect(pm, t, 0.0, 28.0, w, h, &lin(28.0, h, &[(0.0, "#3a1610"), (1.0, "#120504")]));
            for _ in 0..25 {
                let mut x = r.f() * w; let mut y = 34.0 + r.f() * 70.0;
                let mut b = PathBuilder::new(); b.move_to(x, y);
                for _ in 0..4 { x += 6.0 + r.f() * 10.0; y += (r.f() - 0.5) * 8.0; b.line_to(x, y); }
                let p = b.finish();
                stroke(pm, t, p.clone(), &solid(rgba("#ff5000", 0.3)), 3.0);
                stroke(pm, t, p, &solid(rgba("#ff7a1a", 1.0)), 1.0);
            }
        }),
    ];
    let particles = parts(8, 45, |r| Particle { x: r.f() * 470.0, y: r.f() * 240.0, vx: (r.f() - 0.5) * 0.4, vy: -0.5 - r.f() * 0.8, s: 0.5 + r.f() * 0.8, a: 0.9, c: if r.f() < 0.5 { [255, 176, 48] } else { [255, 90, 16] } });
    Stage { info: info("Cratera Vulcânica"), art: StageArt::Procedural { back, particles, tint: Some([1.0, 0.24, 0.0, 0.05]) }, music: 5 }
}

pub struct StageEntry { pub id: String, pub name: String, pub thumb: [[u8; 3]; 3], pub build: Box<dyn Fn() -> Stage> }

pub fn builtin() -> Vec<StageEntry> {
    vec![
        StageEntry { id: "dojo".into(), name: "Telhado do Dojo — Pôr do Sol".into(), thumb: [[42, 30, 92], [212, 80, 122], [255, 154, 74]], build: Box::new(dojo) },
        StageEntry { id: "neon".into(), name: "Neo-Metrópole 2099".into(), thumb: [[5, 3, 15], [29, 11, 58], [255, 47, 208]], build: Box::new(neon) },
        StageEntry { id: "bamboo".into(), name: "Bambuzal ao Luar".into(), thumb: [[4, 16, 28], [18, 58, 74], [44, 106, 106]], build: Box::new(bamboo) },
        StageEntry { id: "plains".into(), name: "Planalto Rochoso".into(), thumb: [[44, 111, 214], [140, 198, 242], [216, 176, 128]], build: Box::new(plains) },
        StageEntry { id: "spirit".into(), name: "Portão dos Espíritos".into(), thumb: [[10, 20, 32], [29, 74, 82], [154, 208, 200]], build: Box::new(spirit) },
        StageEntry { id: "volcano".into(), name: "Cratera Vulcânica".into(), thumb: [[18, 4, 4], [74, 14, 8], [200, 56, 26]], build: Box::new(volcano) },
    ]
}

pub fn from_mugen(st: MugenStage) -> Stage {
    let scl = 320.0 / if st.local_w > 0.0 { st.local_w } else { 320.0 };
    let info = StageInfo {
        name: st.name.clone(), bound_left: st.bound_left * scl, bound_right: st.bound_right * scl, bound_high: st.bound_high * scl, bound_low: st.bound_low * scl,
        p1x: st.p1x * scl, p2x: st.p2x * scl, zoffset: st.zoffset * scl, vertical_follow: st.vertical_follow, tension: st.tension * scl,
    };
    let images = st.sprites.iter().filter(|(_, im)| im.w > 0 && im.h > 0).map(|(k, im)| (*k, ImageData::new(im.w, im.h, im.rgba.clone()))).collect();
    Stage { info, art: StageArt::Mugen(Box::new(MugenBg { stage: st, images, scl })), music: 0 }
}
