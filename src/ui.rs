//! UI helpers: fonts, virtual-resolution layout (1280x720 letterboxed), text & widgets.
use macroquad::prelude::*;

pub const VW: f32 = 1280.0;
pub const VH: f32 = 720.0;

pub struct Ui { pub display: Font, pub body: Font, pub s: f32, pub ox: f32, pub oy: f32, pub t: f32 }

#[derive(Clone, Copy, PartialEq)]
pub enum Align { Left, Center, Right }

pub fn hex(h: &str) -> Color {
    let v = u32::from_str_radix(h.trim_start_matches('#'), 16).unwrap_or(0);
    Color::from_rgba((v >> 16) as u8, (v >> 8) as u8, v as u8, 255)
}
pub fn rgb(c: [u8; 3]) -> Color { Color::from_rgba(c[0], c[1], c[2], 255) }
pub fn alpha(mut c: Color, a: f32) -> Color { c.a = a; c }
/// Soft radial glow (concentric translucent discs).
pub fn glow(x: f32, y: f32, r: f32, c: Color) {
    let n = 14;
    for i in 0..n {
        let t = 1.0 - i as f32 / n as f32;
        draw_poly(x, y, 64, r * t, 0.0, Color::new(c.r, c.g, c.b, c.a / n as f32 * 1.4 * (1.0 - t * 0.4)));
    }
}

impl Ui {
    pub fn new() -> Self {
        let display = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/RussoOne.ttf")).unwrap();
        let body = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Rajdhani.ttf")).unwrap();
        Ui { display, body, s: 1.0, ox: 0.0, oy: 0.0, t: 0.0 }
    }
    pub fn begin(&mut self) {
        let (w, h) = (screen_width(), screen_height());
        self.s = (w / VW).min(h / VH);
        self.ox = (w - VW * self.s) / 2.0;
        self.oy = (h - VH * self.s) / 2.0;
        self.t += get_frame_time();
    }
    pub fn x(&self, v: f32) -> f32 { self.ox + v * self.s }
    pub fn y(&self, v: f32) -> f32 { self.oy + v * self.s }
    /// Convert screen point to virtual coords.
    pub fn to_virtual(&self, p: Vec2) -> Vec2 { vec2((p.x - self.ox) / self.s, (p.y - self.oy) / self.s) }
    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color) { draw_rectangle(self.x(x), self.y(y), w * self.s, h * self.s, c); }
    pub fn rect_lines(&self, x: f32, y: f32, w: f32, h: f32, th: f32, c: Color) { draw_rectangle_lines(self.x(x), self.y(y), w * self.s, h * self.s, th * self.s, c); }
    pub fn gradient_v(&self, x: f32, y: f32, w: f32, h: f32, top: Color, bot: Color) {
        let steps = 24;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let c = Color::new(top.r + (bot.r - top.r) * t, top.g + (bot.g - top.g) * t, top.b + (bot.b - top.b) * t, top.a + (bot.a - top.a) * t);
            self.rect(x, y + h * t, w, h / steps as f32 + 0.6, c);
        }
    }
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Color, align: Align, display: bool) -> f32 {
        let font = if display { &self.display } else { &self.body };
        let fs = (size * self.s).max(4.0) as u16;
        let dim = measure_text(s, Some(font), fs, 1.0);
        let px = match align { Align::Left => self.x(x), Align::Center => self.x(x) - dim.width / 2.0, Align::Right => self.x(x) - dim.width };
        draw_text_ex(s, px, self.y(y), TextParams { font: Some(font), font_size: fs, color: c, ..Default::default() });
        dim.width / self.s
    }
    /// Text with outline + drop shadow (fighting-game style).
    pub fn text_bold(&self, s: &str, x: f32, y: f32, size: f32, c: Color, outline: Color, align: Align) {
        let o = (size * 0.06).max(1.5);
        for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (-o, -o), (o, o), (-o, o), (o, -o), (o * 0.6, o * 2.0)] { self.text(s, x + dx, y + dy, size, outline, align, true); }
        self.text(s, x, y, size, c, align, true);
    }
    pub fn measure(&self, s: &str, size: f32, display: bool) -> f32 {
        let font = if display { &self.display } else { &self.body };
        measure_text(s, Some(font), (size * self.s).max(4.0) as u16, 1.0).width / self.s
    }
    pub fn quad(&self, p: [Vec2; 4], c: Color) {
        let v: Vec<Vec2> = p.iter().map(|q| vec2(self.x(q.x), self.y(q.y))).collect();
        draw_triangle(v[0], v[1], v[2], c);
        draw_triangle(v[0], v[2], v[3], c);
    }
    pub fn background(&self) {
        let (w, h) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, w, h, hex("#0b0716"));
        let steps = 30;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            draw_rectangle(0.0, h * t, w, h / steps as f32 + 1.0, Color::new(0.04 + 0.05 * t, 0.03 + 0.02 * t, 0.09 + 0.08 * t, 1.0));
        }
        glow(w * 0.5, h * 1.15, h * 0.8, Color::new(1.0, 0.3, 0.43, 0.35));
        glow(w * 0.1, h * 0.05, h * 0.5, Color::new(0.13, 0.9, 1.0, 0.22));
        glow(w * 0.9, h * 0.12, h * 0.45, Color::new(1.0, 0.8, 0.2, 0.18));
        let t = self.t * 20.0;
        let mut x = -(t % 40.0) - h;
        while x < w { draw_line(x, h, x + h * 0.47, 0.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.025)); x += 40.0; }
        for y in (0..h as i32).step_by(3) { draw_line(0.0, y as f32, w, y as f32, 1.0, Color::new(0.0, 0.0, 0.0, 0.1)); }
    }
    pub fn logo(&self, cx: f32, cy: f32, size: f32) {
        let pulse = (self.t * 2.0).sin() * 0.05 + 1.0;
        self.text_bold("ULTIMATE", cx, cy - size * 0.95, size * 0.32, hex("#22e8ff"), Color::new(0.0, 0.1, 0.2, 0.9), Align::Center);
        let s = size * pulse;
        for (i, c) in ["#d81b4a", "#ff6a1f", "#ffcc33"].iter().enumerate() {
            let o = (2 - i) as f32 * size * 0.02;
            self.text("MUGEN", cx + o, cy + o * 1.5, s, hex(c), Align::Center, true);
        }
        self.text("MUGEN", cx - size * 0.01, cy - size * 0.01, s, hex("#fff7c0"), Align::Center, true);
    }
}
