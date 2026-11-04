use crate::gfx::*;
use crate::ui::*;

#[derive(Clone, Copy)]
struct Theme { top: u32, bottom: u32, accent: u32, glow: u32 }

const THEMES: [Theme; 11] = [
    Theme { top: 0x1a0a05, bottom: 0x5a1c08, accent: 0xffb21f, glow: 0xff6a1f },
    Theme { top: 0x06142a, bottom: 0x0b3a63, accent: 0x22e8ff, glow: 0x2f7bff },
    Theme { top: 0x2a0614, bottom: 0x0a1a4a, accent: 0xff4d6d, glow: 0x3d8bff },
    Theme { top: 0x04180f, bottom: 0x0a4a2a, accent: 0x6dff9e, glow: 0x22c26a },
    Theme { top: 0x120726, bottom: 0x3a1470, accent: 0xb48cff, glow: 0x8a2be2 },
    Theme { top: 0x24060a, bottom: 0x5a0d20, accent: 0xff5a5a, glow: 0xff9a3d },
    Theme { top: 0x061a24, bottom: 0x0c4a52, accent: 0x5af0d0, glow: 0x1fb6c8 },
    Theme { top: 0x241405, bottom: 0x4a2a08, accent: 0xffcc33, glow: 0xff8a1f },
    Theme { top: 0x0a0a24, bottom: 0x1d1f5a, accent: 0x8ab4ff, glow: 0x5a6bff },
    Theme { top: 0x0a1224, bottom: 0x123a5a, accent: 0x7fe3ff, glow: 0x2fa8ff },
    Theme { top: 0x120a14, bottom: 0x2a1030, accent: 0xff7aa8, glow: 0x7a2a5a },
];

pub struct MenuArt { prev: usize, cur: usize, fade: f32 }

fn col(h: u32) -> Color { Color::new(((h >> 16) & 255) as f32 / 255.0, ((h >> 8) & 255) as f32 / 255.0, (h & 255) as f32 / 255.0, 1.0) }

fn mix(a: Color, b: Color, t: f32) -> Color { Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t) }

impl MenuArt {
    pub fn new() -> Self { MenuArt { prev: 0, cur: 0, fade: 1.0 } }

    pub fn select(&mut self, i: usize) {
        if i == self.cur { return; }
        self.prev = self.cur;
        self.cur = i;
        self.fade = 0.0;
    }

    pub fn fade_in(&self) -> f32 { self.fade }

    pub fn tick(&mut self, dt: f32) { self.fade = (self.fade + dt / 0.38).min(1.0); }

    pub fn draw(&self, ui: &Ui) {
        let e = self.fade * self.fade * (3.0 - 2.0 * self.fade);
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), hex("#07040f"));
        if e < 1.0 { scene(ui, self.prev, 1.0, -e * 0.06); }
        scene(ui, self.cur, e, (1.0 - e) * 0.06);
        vignette(ui);
    }
}

fn vignette(ui: &Ui) {
    let (w, h) = (screen_width(), screen_height());
    let band = w * 0.55;
    for i in 0..10 {
        let t = i as f32 / 10.0;
        let a = (1.0 - t) * (1.0 - t) * 0.55;
        draw_rectangle(0.0, 0.0, band * (1.0 - t), h, Color::new(0.02, 0.01, 0.06, a * 0.5));
    }
    draw_rectangle(0.0, h * 0.86, w, h * 0.14, Color::new(0.0, 0.0, 0.0, 0.35));
    let _ = ui;
}

fn scene(ui: &Ui, idx: usize, a: f32, slide: f32) {
    if a <= 0.0 { return; }
    let th = THEMES[idx % THEMES.len()];
    let (w, h) = (screen_width(), screen_height());
    let ox = slide * w;
    let top = col(th.top);
    let bot = col(th.bottom);
    let acc = col(th.accent);
    let glw = col(th.glow);
    let steps = 16;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let c = mix(top, bot, t);
        draw_rectangle(0.0, h * t, w, h / steps as f32 + 1.0, Color::new(c.r, c.g, c.b, a));
    }
    let fx = w * 0.68 + ox;
    let fy = h * 0.46;
    rays(ui, fx, fy, w.max(h) * 1.2, acc, a * 0.10, idx);
    glow(fx, fy, h * 0.75, Color::new(glw.r, glw.g, glw.b, 0.42 * a));
    motif(ui, idx, fx, fy, w, h, acc, glw, a);
    ui.motes(14, 0.8 * a);
}

fn rays(ui: &Ui, fx: f32, fy: f32, len: f32, c: Color, a: f32, idx: usize) {
    let n = 14;
    let spin = ui.t * 0.05 * if idx % 2 == 0 { 1.0 } else { -1.0 };
    for i in 0..n {
        let ang = spin + i as f32 / n as f32 * std::f32::consts::TAU;
        let wid = 0.07;
        draw_triangle(vec2(fx, fy), vec2(fx + (ang - wid).cos() * len, fy + (ang - wid).sin() * len), vec2(fx + (ang + wid).cos() * len, fy + (ang + wid).sin() * len), Color::new(c.r, c.g, c.b, a * if i % 2 == 0 { 1.0 } else { 0.5 }));
    }
}

fn ring(ui: &Ui, x: f32, y: f32, r: f32, th: f32, c: Color) { draw_circle_lines(x, y, r, th * ui.s.max(1.0), c); }

fn motif(ui: &Ui, idx: usize, fx: f32, fy: f32, w: f32, h: f32, acc: Color, glw: Color, a: f32) {
    let t = ui.t;
    let s = ui.s.max(0.5);
    let ca = |k: f32| Color::new(acc.r, acc.g, acc.b, k * a);
    let cg = |k: f32| Color::new(glw.r, glw.g, glw.b, k * a);
    match idx {
        0 => {
            for i in 0..7 {
                let k = i as f32;
                let bw = 150.0 * s;
                let bh = (90.0 + k * 55.0) * s;
                let x = w * 0.42 + k * bw * 0.78;
                draw_rectangle(x, h - bh, bw * 0.7, bh, cg(0.10 + 0.04 * k));
                draw_rectangle(x, h - bh, bw * 0.7, 4.0 * s, ca(0.55));
            }
            ring(ui, fx, fy, 160.0 * s + (t * 2.0).sin() * 6.0 * s, 3.0, ca(0.7));
            ring(ui, fx, fy, 230.0 * s, 1.5, ca(0.35));
        }
        1 => {
            for i in 0..9 {
                let y = h * (0.12 + i as f32 * 0.1);
                let x0 = w * 0.45;
                let l = w * (0.18 + 0.12 * ((i * 5 % 7) as f32 / 7.0));
                draw_line(x0, y, x0 + l, y, 2.0 * s, ca(0.5));
                draw_line(x0 + l, y, x0 + l + 40.0 * s, y + 40.0 * s, 2.0 * s, ca(0.5));
                draw_circle(x0 + l + 40.0 * s, y + 40.0 * s, 6.0 * s, ca(0.9));
            }
            let pulse = ((t * 1.6) % 1.0) * w * 0.4;
            draw_rectangle(w * 0.45 + pulse, 0.0, 3.0 * s, h, ca(0.35));
        }
        2 => {
            let k = (t * 3.0).sin() * 8.0 * s;
            draw_triangle(vec2(fx - 20.0 * s, 0.0), vec2(fx + 20.0 * s + k, 0.0), vec2(fx + 90.0 * s, h), cg(0.35));
            draw_triangle(vec2(fx + 20.0 * s, 0.0), vec2(fx - 20.0 * s - k, h), vec2(fx - 90.0 * s, h), ca(0.30));
            draw_line(fx, 0.0, fx, h, 4.0 * s, Color::new(1.0, 1.0, 1.0, 0.5 * a));
            ring(ui, fx, fy, 120.0 * s + k, 4.0, Color::new(1.0, 1.0, 1.0, 0.6 * a));
        }
        3 => {
            let hor = h * 0.58;
            for i in 0..14 {
                let u = (i as f32 + (t * 0.6) % 1.0) / 14.0;
                let y = hor + (h - hor) * u * u;
                draw_line(0.0, y, w, y, 1.5 * s, ca(0.35 * u));
            }
            for i in -12..=12 {
                let x = w * 0.5 + i as f32 * w * 0.09;
                draw_line(w * 0.5 + i as f32 * w * 0.012, hor, x * 1.0 + (x - w * 0.5) * 0.9, h, 1.5 * s, ca(0.28));
            }
            ring(ui, fx, hor - 90.0 * s, 70.0 * s, 3.0, ca(0.8));
            ring(ui, fx, hor - 90.0 * s, 28.0 * s, 3.0, ca(0.8));
        }
        4 => {
            for i in 0..6 {
                let r = (60.0 + i as f32 * 55.0 + (t * 24.0) % 55.0) * s;
                ring(ui, fx, fy, r, 2.0, ca(0.5 * (1.0 - i as f32 / 6.0)));
            }
            draw_circle(fx, fy, 38.0 * s, ca(0.8));
            draw_circle(fx + (t * 0.8).sin() * 12.0 * s, fy, 16.0 * s, Color::new(0.05, 0.02, 0.12, a));
        }
        5 => {
            for i in 0..5 {
                let k = i as f32;
                let x = w * 0.4 + k * 135.0 * s;
                let y = h * (0.78 - k * 0.1);
                draw_triangle(vec2(x, y), vec2(x + 60.0 * s, y - 120.0 * s), vec2(x + 120.0 * s, y), cg(0.30));
                draw_line(x, y, x + 60.0 * s, y - 120.0 * s, 3.0 * s, ca(0.7));
                draw_line(x + 60.0 * s, y - 120.0 * s, x + 120.0 * s, y, 3.0 * s, ca(0.7));
                if k < 4.0 { draw_line(x + 90.0 * s, y - 60.0 * s, x + 165.0 * s, y - 70.0 * s, 2.0 * s, ca(0.5)); }
            }
        }
        6 => {
            for i in 0..5 {
                let k = i as f32;
                let x = w * 0.45 + k * 120.0 * s;
                let bh = h * (0.45 + 0.08 * ((t * 1.3 + k).sin()));
                draw_rectangle(x, h - bh, 70.0 * s, bh, cg(0.22));
                draw_rectangle(x, h - bh, 70.0 * s, 5.0 * s, ca(0.8));
                draw_rectangle(x + 8.0 * s, h - bh - 70.0 * s, 54.0 * s, 54.0 * s, ca(0.25));
            }
        }
        7 => {
            for r in 0..6 {
                for c in 0..8 {
                    let x = w * 0.38 + c as f32 * 90.0 * s + if r % 2 == 1 { 45.0 * s } else { 0.0 };
                    let y = h * 0.14 + r as f32 * 78.0 * s;
                    let pulse = 0.5 + 0.5 * (t * 1.2 + (r * 3 + c) as f32 * 0.7).sin();
                    draw_poly(x, y, 6, 40.0 * s, 0.0, ca(0.04 + 0.12 * pulse));
                    draw_poly(x, y, 6, 36.0 * s, 0.0, cg(0.05 + 0.10 * pulse));
                }
            }
        }
        8 => {
            let pos = [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)];
            for (i, p) in pos.iter().enumerate() {
                let (x, y) = (fx - 190.0 * s + p.0 * 62.0 * s, fy + p.1 * 62.0 * s);
                ring(ui, x, y, 26.0 * s, 3.0, ca(0.6 + 0.3 * ((t * 2.0 + i as f32).sin())));
                let (x2, y2) = (fx + 190.0 * s + p.0 * 62.0 * s, fy + p.1 * 62.0 * s);
                draw_circle(x2, y2, 26.0 * s, ca(0.12 + 0.2 * (0.5 + 0.5 * (t * 2.0 + i as f32 * 1.6).sin())));
                ring(ui, x2, y2, 26.0 * s, 3.0, ca(0.7));
            }
            draw_rectangle_lines(fx - 90.0 * s, fy - 40.0 * s, 180.0 * s, 80.0 * s, 3.0 * s, ca(0.5));
        }
        9 => {
            let (bw, bh) = (520.0 * s, 300.0 * s);
            draw_rectangle_lines(fx - bw / 2.0, fy - bh / 2.0, bw, bh, 4.0 * s, ca(0.8));
            draw_rectangle(fx - bw / 2.0, fy - bh / 2.0, bw, bh, cg(0.16));
            let sweep = ((t * 0.5) % 1.0) * bh;
            draw_rectangle(fx - bw / 2.0, fy - bh / 2.0 + sweep, bw, 6.0 * s, ca(0.5));
            for i in 1..4 { draw_line(fx - bw / 2.0 + bw * i as f32 / 4.0, fy - bh / 2.0, fx - bw / 2.0 + bw * i as f32 / 4.0, fy + bh / 2.0, 1.5 * s, ca(0.2)); }
            draw_rectangle(fx - 40.0 * s, fy + bh / 2.0, 80.0 * s, 30.0 * s, ca(0.4));
        }
        _ => {
            let hor = h * 0.62;
            draw_rectangle(0.0, hor, w, h - hor, Color::new(0.04, 0.02, 0.07, 0.7 * a));
            draw_circle(fx, hor - 20.0 * s, 150.0 * s, ca(0.35));
            draw_circle(fx, hor - 20.0 * s, 110.0 * s, ca(0.55));
            for i in 0..6 { draw_rectangle(0.0, hor - 130.0 * s + i as f32 * 22.0 * s, w, (3.0 + i as f32 * 1.2) * s, Color::new(0.06, 0.02, 0.1, 0.8 * a)); }
        }
    }
}

pub fn glass(ui: &Ui, x: f32, y: f32, w: f32, h: f32, a: f32) {
    ui.rect(x, y, w, h, Color::new(0.04, 0.03, 0.10, 0.55 * a));
    ui.gradient_v(x, y, w, h * 0.5, Color::new(1.0, 1.0, 1.0, 0.07 * a), Color::new(1.0, 1.0, 1.0, 0.0));
    ui.rect(x, y, w, 1.5, Color::new(1.0, 1.0, 1.0, 0.22 * a));
    ui.rect(x, y + h - 1.0, w, 1.0, Color::new(1.0, 1.0, 1.0, 0.08 * a));
}

pub fn accent_of(idx: usize) -> Color { col(THEMES[idx % THEMES.len()].accent) }
