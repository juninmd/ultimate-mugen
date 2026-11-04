//! UI helpers: fonts, virtual-resolution layout (1280x720 letterboxed), text & widgets.
use crate::gfx::*;

pub const VW: f32 = 1280.0;
pub const VH: f32 = 720.0;

pub struct Ui { pub display: Font, pub body: Font, pub s: f32, pub ox: f32, pub oy: f32, pub t: f32 }

#[derive(Clone, Copy, PartialEq)]
pub enum Align { Left, Center, Right }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mark { Square, Triangle, Cross, Circle }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FaceShape { Chip, Round, Key }

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Face { pub text: &'static str, pub color: Color, pub shape: FaceShape, pub mark: Option<Mark> }

impl Face {
    pub fn chip(text: &'static str, color: Color) -> Self { Face { text, color, shape: FaceShape::Chip, mark: None } }
    pub fn round(text: &'static str, color: Color) -> Self { Face { text, color, shape: FaceShape::Round, mark: None } }
    pub fn key(text: &'static str) -> Self { Face { text, color: Color::new(0.92, 0.92, 0.96, 1.0), shape: FaceShape::Key, mark: None } }
    pub fn mark(mark: Mark, color: Color) -> Self { Face { text: "", color, shape: FaceShape::Round, mark: Some(mark) } }
}

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
        let mut display = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/RussoOne.ttf")).unwrap();
        let mut body = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Rajdhani.ttf")).unwrap();
        display.set_filter(FilterMode::Linear);
        body.set_filter(FilterMode::Linear);
        set_default_font(display.clone());
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
    /// Font size snapped to ~4% steps: the glyph atlas stays small (4K!) and animated sizes do not re-rasterize every frame.
    fn quant(fs: f32) -> (u16, f32) {
        let fs = fs.max(4.0);
        let q = (2f32.powf((fs.log2() * 12.0).round() / 12.0)).round().max(4.0);
        (q as u16, fs / q)
    }
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Color, align: Align, display: bool) -> f32 {
        let font = if display { &self.display } else { &self.body };
        let (fq, sc) = Self::quant(size * self.s);
        let dim = measure_text(s, Some(font), fq, sc);
        let px = match align { Align::Left => self.x(x), Align::Center => self.x(x) - dim.width / 2.0, Align::Right => self.x(x) - dim.width };
        draw_text_ex(s, px, self.y(y), TextParams { font: Some(font), font_size: fq, font_scale: sc, color: c, ..Default::default() });
        dim.width / self.s
    }
    pub fn arrow(&self, cx: f32, cy: f32, size: f32, dir: (f32, f32), c: Color) {
        let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt().max(0.001);
        let (dx, dy) = (dir.0 / len, dir.1 / len);
        let (px, py) = (-dy, dx);
        let h = size * 0.45;
        let tip = vec2(self.x(cx + dx * h), self.y(cy + dy * h));
        let base = (cx - dx * h * 0.1, cy - dy * h * 0.1);
        let l = vec2(self.x(base.0 + px * h * 0.75), self.y(base.1 + py * h * 0.75));
        let r = vec2(self.x(base.0 - px * h * 0.75), self.y(base.1 - py * h * 0.75));
        draw_line(self.x(cx - dx * h), self.y(cy - dy * h), self.x(base.0), self.y(base.1), size * 0.22 * self.s, c);
        draw_triangle(tip, l, r, c);
    }
    pub fn face(&self, f: Face, x: f32, y: f32, size: f32) -> f32 {
        let fs = size * 0.62;
        match f.shape {
            FaceShape::Round => {
                let r = size / 2.0;
                let (cx, cy) = (x + r, y - r);
                draw_circle(self.x(cx), self.y(cy), r * self.s, Color::new(0.08, 0.07, 0.12, 0.95));
                draw_circle_lines(self.x(cx), self.y(cy), r * self.s, 2.0 * self.s, f.color);
                let m = r * 0.48;
                let th = 2.6 * self.s;
                match f.mark {
                    Some(Mark::Square) => {
                        let (a, b) = (self.x(cx - m), self.y(cy - m));
                        let (c, d) = (self.x(cx + m), self.y(cy + m));
                        draw_line(a, b, c, b, th, f.color); draw_line(c, b, c, d, th, f.color); draw_line(c, d, a, d, th, f.color); draw_line(a, d, a, b, th, f.color);
                    }
                    Some(Mark::Triangle) => {
                        let p = [(cx, cy - m * 1.1), (cx + m * 1.05, cy + m * 0.75), (cx - m * 1.05, cy + m * 0.75)];
                        for i in 0..3 { let (a, b) = (p[i], p[(i + 1) % 3]); draw_line(self.x(a.0), self.y(a.1), self.x(b.0), self.y(b.1), th, f.color); }
                    }
                    Some(Mark::Cross) => {
                        draw_line(self.x(cx - m), self.y(cy - m), self.x(cx + m), self.y(cy + m), th, f.color);
                        draw_line(self.x(cx + m), self.y(cy - m), self.x(cx - m), self.y(cy + m), th, f.color);
                    }
                    Some(Mark::Circle) => draw_circle_lines(self.x(cx), self.y(cy), m * self.s, th, f.color),
                    None => { self.text(f.text, cx, y - size * 0.28, fs, f.color, Align::Center, true); }
                }
                size
            }
            FaceShape::Chip | FaceShape::Key => {
                let w = (measure_text(f.text, Some(&self.display), (fs * self.s) as u16, 1.0).width / self.s + size * 0.6).max(size);
                if f.shape == FaceShape::Key {
                    self.rect(x, y - size, w, size, Color::new(0.85, 0.85, 0.9, 0.95));
                    self.rect(x + 2.0, y - size + 2.0, w - 4.0, size - 6.0, Color::new(0.97, 0.97, 1.0, 1.0));
                    self.text(f.text, x + w / 2.0, y - size * 0.3, fs, hex("#1b1726"), Align::Center, false);
                } else {
                    self.rect(x, y - size, w, size, f.color);
                    self.text(f.text, x + w / 2.0, y - size * 0.28, fs, WHITE, Align::Center, true);
                }
                w
            }
        }
    }
    pub fn input_icons(&self, input: &str, x: f32, y: f32, size: f32, face: impl Fn(u32) -> Face) -> f32 {
        use ume_engine::input::k;
        let mut cx = x;
        for tok in input.replace(['~', ' ', '/', '$', '>'], "").split(',').filter(|t| !t.is_empty()) {
            let mut btns = vec![];
            for p in tok.split('+') {
                let d = match p { "F" => Some((1.0, 0.0)), "B" => Some((-1.0, 0.0)), "U" => Some((0.0, -1.0)), "D" => Some((0.0, 1.0)), "DF" => Some((1.0, 1.0)), "DB" => Some((-1.0, 1.0)), "UF" => Some((1.0, -1.0)), "UB" => Some((-1.0, -1.0)), _ => None };
                if let Some(d) = d {
                    self.rect(cx, y - size, size, size, Color::new(1.0, 1.0, 1.0, 0.1));
                    self.arrow(cx + size / 2.0, y - size / 2.0, size, d, WHITE);
                    cx += size + 4.0;
                    continue;
                }
                let bit = match p { "x" => k::X, "y" => k::Y, "z" => k::Z, "a" => k::A, "b" => k::B, "c" => k::C, "s" => k::S, _ => continue };
                btns.push(face(bit));
            }
            for (n, f) in btns.into_iter().enumerate() {
                if n > 0 || cx > x { cx += self.text("+", cx + 2.0, y - size * 0.2, size * 0.8, hex("#a49cc0"), Align::Left, true) + 6.0; }
                cx += self.face(f, cx, y, size) + 4.0;
            }
        }
        cx - x
    }
    /// Text with outline + drop shadow (fighting-game style).
    pub fn text_bold(&self, s: &str, x: f32, y: f32, size: f32, c: Color, outline: Color, align: Align) {
        let o = (size * 0.06).max(1.5);
        let outline = Color { a: outline.a.clamp(0.0, 1.0).powi(3), ..outline };
        for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (-o, -o), (o, o), (-o, o), (o, -o), (o * 0.6, o * 2.0)] { self.text(s, x + dx, y + dy, size, outline, align, true); }
        self.text(s, x, y, size, c, align, true);
    }
    pub fn measure(&self, s: &str, size: f32, display: bool) -> f32 {
        let font = if display { &self.display } else { &self.body };
        let (fq, sc) = Self::quant(size * self.s);
        measure_text(s, Some(font), fq, sc).width / self.s
    }
    pub fn quad(&self, p: [Vec2; 4], c: Color) {
        let v = p.map(|q| vec2(self.x(q.x), self.y(q.y)));
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
        let gap = 40.0 * self.s;
        let mut x = -((t * self.s) % gap) - h;
        while x < w { draw_line(x, h, x + h * 0.47, 0.0, self.s.max(1.0), Color::new(1.0, 1.0, 1.0, 0.025)); x += gap; }
        self.motes(46, 1.0);
        // slow rotating light beams
        for i in 0..3 {
            let a = self.t * 0.12 + i as f32 * 2.1;
            let (bx, by) = (w * (0.2 + 0.3 * i as f32), -20.0);
            let sp = (a.sin()) * w * 0.25;
            draw_triangle(vec2(bx, by), vec2(bx + sp - h * 0.13, h), vec2(bx + sp + h * 0.13, h), Color::new(1.0, 0.85, 0.6, 0.018));
        }
        // scanlines scale with the display so they stay subtle at 4K
        let (step, th) = ((3.0 * self.s).max(2.0), self.s.max(1.0));
        let mut y = 0.0;
        while y < h { draw_line(0.0, y, w, y, th, Color::new(0.0, 0.0, 0.0, 0.1)); y += step; }
    }
    /// Floating light motes drifting upward (deterministic; `warm` 1 = gold/pink, 0 = cool).
    pub fn motes(&self, n: usize, strength: f32) {
        let (w, h) = (screen_width(), screen_height());
        for i in 0..n {
            let f = i as f32;
            let h1 = ((f * 12.9898).sin() * 43758.5453).fract().abs();
            let h2 = ((f * 78.233 + 3.0).sin() * 12543.1).fract().abs();
            let h3 = ((f * 39.346 + 7.0).sin() * 9871.7).fract().abs();
            let sp = (14.0 + h2 * 34.0) * self.s;
            let y = h - ((self.t * sp + h3 * h) % (h * 1.1)) + h * 0.05;
            let x = h1 * w + (self.t * 0.7 + f).sin() * 18.0 * self.s;
            let tw = 0.5 + 0.5 * (self.t * (1.5 + h2 * 2.0) + f).sin();
            let c = if i % 3 == 0 { (0.35, 0.9, 1.0) } else if i % 3 == 1 { (1.0, 0.45, 0.6) } else { (1.0, 0.85, 0.4) };
            let r = (1.0 + h2 * 2.2) * self.s;
            draw_circle(x, y, r * 3.2, Color::new(c.0, c.1, c.2, 0.05 * tw * strength));
            draw_circle(x, y, r, Color::new(c.0, c.1, c.2, (0.2 + 0.4 * tw) * strength));
        }
    }
    /// Diagonal wipe used between screens: `q` goes 1 → 0 (covering → revealed).
    pub fn wipe(&self, q: f32) {
        if q <= 0.0 { return; }
        let (w, h) = (screen_width(), screen_height());
        let e = q * q * (3.0 - 2.0 * q);
        let slant = h * 0.28;
        let x0 = (1.0 - e) * (w + slant * 2.0) - slant;
        for (k, (off, col)) in [(0.0, Color::new(0.03, 0.02, 0.09, 1.0)), (34.0, Color::new(1.0, 0.3, 0.43, 1.0)), (62.0, Color::new(0.13, 0.9, 1.0, 1.0))].iter().enumerate().rev() {
            // the cyan / magenta edges lead the dark panel
            let lead = *off * self.s * (1.0 - (1.0 - q).powi(2));
            let a = x0 - lead; let b = w + slant * 2.0;
            let _ = k;
            draw_triangle(vec2(a + slant, 0.0), vec2(b, 0.0), vec2(b, h), *col);
            draw_triangle(vec2(a + slant, 0.0), vec2(b, h), vec2(a, h), *col);
        }
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

    /// Progress bar + spinner (virtual coordinates).
    pub fn progress_bar(&self, x: f32, y: f32, w: f32, p: f32, label: &str) {
        let p = p.clamp(0.0, 1.0);
        self.rect(x - 2.0, y - 2.0, w + 4.0, 16.0, Color::new(0.0, 0.0, 0.0, 0.55));
        self.rect(x, y, w, 12.0, Color::new(1.0, 1.0, 1.0, 0.10));
        self.rect(x, y, w * p, 12.0, hex("#ff4d6d"));
        self.rect(x, y, w * p, 5.0, Color::new(1.0, 1.0, 1.0, 0.25));
        let a = self.t * 6.0;
        for i in 0..8 {
            let ang = a - i as f32 * 0.5;
            let r = 12.0;
            let c = Color::new(1.0, 0.8, 0.2, 1.0 - i as f32 / 8.0);
            draw_circle(self.x(x - 30.0 + ang.cos() * r), self.y(y + 6.0 + ang.sin() * r), (3.2 - i as f32 * 0.25).max(1.0) * self.s, c);
        }
        self.text(label, x, y + 36.0, 18.0, hex("#d8d0f0"), Align::Left, false);
        self.text(&format!("{}%", (p * 100.0) as i32), x + w, y + 36.0, 18.0, hex("#ffcc33"), Align::Right, true);
    }
    /// Full-screen loading view with rotating tips.
    pub fn loading_screen(&self, p: f32, label: &str) {
        self.background();
        self.logo(VW / 2.0, 300.0, 150.0);
        let tip = LOADING_TIPS[((self.t / 3.5) as usize) % LOADING_TIPS.len()];
        self.text(tip, VW / 2.0, 480.0, 20.0, hex("#a49cc0"), Align::Center, false);
        self.progress_bar(VW / 2.0 - 260.0, 560.0, 520.0, p, label);
    }
}

pub const LOADING_TIPS: [&str; 8] = [
    "Dica: F11 alterna tela cheia · F3 mostra o FPS.",
    "Dica: arraste um .zip de personagens M.U.G.E.N para a janela para importar.",
    "Dica: no Treino, F2 exibe as hitboxes.",
    "Dica: ESPECIAL e SUPER executam golpes com um único toque.",
    "Dica: em Vídeo, a Qualidade da arte aumenta a nitidez em telas 4K.",
    "Dica: PageUp/PageDown e a roda do mouse rolam a lista de lutadores.",
    "Dica: DualShock/DualSense vibram em golpes, defesa e K.O.",
    "Dica: Tab filtra a seleção entre todos, originais e M.U.G.E.N.",
];
