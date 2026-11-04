use crate::{text, Config, Frame, Game, Phase};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use ume_render::RGBA_SPRITE;

pub use crate::PostParams;
pub use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};
pub use ume_render::Blend;

thread_local! {
    static FRAME: Cell<*mut ()> = const { Cell::new(std::ptr::null_mut()) };
    static DEFAULT_FONT: RefCell<Option<Font>> = const { RefCell::new(None) };
    static RNG: Cell<u64> = const { Cell::new(0x9e37_79b9_7f4a_7c15) };
    static GENERATION: Cell<u32> = const { Cell::new(0) };
    static ATLAS_FULL: Cell<bool> = const { Cell::new(false) };
    static KEY_QUEUE: RefCell<Vec<KeyCode>> = const { RefCell::new(Vec::new()) };
}

fn generation() -> u32 { GENERATION.with(|g| g.get()) }

fn reset_atlas(f: &mut Frame) {
    f.gpu.rgba_atlas.clear();
    if let Some(c) = ume_render::Canvas::new(&f.gpu.renderer, &f.gpu.queue, &mut f.gpu.rgba_atlas) { *f.canvas = c; }
    GENERATION.with(|g| g.set(g.get().wrapping_add(1)));
    ATLAS_FULL.with(|a| a.set(false));
}

pub fn reset_textures() { ATLAS_FULL.with(|a| a.set(true)); }

fn with<R>(f: impl FnOnce(&mut Frame<'static>) -> R) -> R {
    let p = FRAME.with(|c| c.get());
    assert!(!p.is_null(), "ume_shell::quad called outside the frame loop");
    f(unsafe { &mut *(p as *mut Frame<'static>) })
}

struct AsyncGame { main: Option<Pin<Box<dyn Future<Output = ()>>>> }

impl Game for AsyncGame {
    fn init(&mut self, _: &mut crate::Gpu) { reset_textures(); }
    fn frame(&mut self, f: &mut Frame) {
        if ATLAS_FULL.with(|a| a.get()) { reset_atlas(f); }
        KEY_QUEUE.with(|q| { let mut q = q.borrow_mut(); q.clear(); q.extend(f.input.pressed_order.iter().filter_map(|k| KeyCode::from_winit(*k))); });
        FRAME.with(|c| c.set(f as *mut Frame as *mut ()));
        let done = match self.main.as_mut() {
            Some(m) => m.as_mut().poll(&mut Context::from_waker(Waker::noop())).is_ready(),
            None => true,
        };
        FRAME.with(|c| c.set(std::ptr::null_mut()));
        if done { self.main = None; f.quit = true; }
    }
}

pub fn run_async(cfg: Config, main: impl Future<Output = ()> + 'static) -> Result<u64, String> {
    crate::run(cfg, AsyncGame { main: Some(Box::pin(main)) })
}

pub struct NextFrame(bool);

impl Future for NextFrame {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        if self.0 { Poll::Ready(()) } else { self.0 = true; Poll::Pending }
    }
}

pub fn next_frame() -> NextFrame { NextFrame(false) }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color { Color { r, g, b, a } }
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color { Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0) }
    pub fn to_array(self) -> [f32; 4] { [self.r, self.g, self.b, self.a] }
    fn render(self) -> ume_render::Color { ume_render::Color(self.to_array()) }
}

impl From<[f32; 4]> for Color { fn from(c: [f32; 4]) -> Color { Color::new(c[0], c[1], c[2], c[3]) } }
impl From<Color> for [u8; 4] { fn from(c: Color) -> [u8; 4] { c.to_array().map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8) } }

pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
pub const BLANK: Color = Color::new(0.0, 0.0, 0.0, 0.0);
pub const RED: Color = Color::new(0.9, 0.16, 0.22, 1.0);
pub const ORANGE: Color = Color::new(1.0, 0.63, 0.0, 1.0);
pub const YELLOW: Color = Color::new(0.99, 0.98, 0.0, 1.0);
pub const GREEN: Color = Color::new(0.0, 0.89, 0.19, 1.0);
pub const BLUE: Color = Color::new(0.0, 0.47, 0.95, 1.0);
pub const GRAY: Color = Color::new(0.51, 0.51, 0.51, 1.0);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect { Rect { x, y, w, h } }
    pub fn contains(&self, p: Vec2) -> bool { p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h }
}

pub fn screen_width() -> f32 { with(|f| f.size[0]) }
pub fn screen_height() -> f32 { with(|f| f.size[1]) }
pub fn dpi_scale() -> f32 { 1.0 }
pub fn window_scale() -> f32 { with(|f| f.scale) }
pub fn get_time() -> f64 { with(|f| f.time) }
pub fn get_frame_time() -> f32 { with(|f| f.dt) }
pub fn get_fps() -> i32 { with(|f| if f.dt > 0.0 { (1.0 / f.dt).round() as i32 } else { 0 }) }
pub fn request_new_screen_size(w: f32, h: f32) { with(|f| f.resize = Some([w.max(1.0) as u32, h.max(1.0) as u32])) }
pub fn set_fullscreen(on: bool) { with(|f| f.fullscreen = Some(on)) }
pub fn set_post(p: Option<PostParams>) { with(|f| f.post = p) }
pub fn set_blend(b: Blend) { with(|f| f.canvas.blend = b) }
pub fn quit() { with(|f| f.quit = true) }
pub fn set_scissor(r: Option<(i32, i32, i32, i32)>) { with(|f| f.canvas.batch.scissor = r.map(|(x, y, w, h)| [x.max(0) as u32, y.max(0) as u32, w.max(0) as u32, h.max(0) as u32])) }
pub fn mouse_wheel() -> (f32, f32) { with(|f| (f.input.wheel[0], f.input.wheel[1])) }
pub fn mouse_delta_position() -> Vec2 { with(|f| Vec2::from(f.input.mouse_delta)) }

pub struct ScreenData;

impl ScreenData {
    pub fn export_png(&self, path: &str) { with(|f| f.screenshot = Some(path.to_string())) }
}

pub fn get_screen_data() -> ScreenData { ScreenData }

pub fn clear_background(c: Color) {
    with(|f| {
        f.canvas.clear();
        f.clear = [c.r as f64, c.g as f64, c.b as f64, c.a as f64];
    })
}

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, c: Color) { with(|f| f.canvas.rect(x, y, w, h, c.render())) }

pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, c: Color) {
    with(|f| f.canvas.rect_lines(x, y, w, h, thickness.min(w * 0.5).min(h * 0.5), c.render()))
}

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, c: Color) { with(|f| f.canvas.line([x1, y1], [x2, y2], thickness, c.render())) }
pub fn draw_circle(x: f32, y: f32, r: f32, c: Color) { with(|f| f.canvas.circle([x, y], r, c.render())) }
pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, c: Color) { with(|f| f.canvas.circle_lines([x, y], r, thickness, c.render())) }
pub fn draw_triangle(a: Vec2, b: Vec2, c: Vec2, color: Color) { with(|f| f.canvas.triangle(a.into(), b.into(), c.into(), color.render())) }

pub fn draw_poly(x: f32, y: f32, sides: u8, radius: f32, rotation: f32, c: Color) {
    draw_ellipse_poly(x, y, radius, radius, rotation.to_radians(), sides.max(3) as u32, c)
}

pub fn draw_ellipse(x: f32, y: f32, w: f32, h: f32, rotation: f32, c: Color) { draw_ellipse_poly(x, y, w, h, rotation.to_radians(), 32, c) }

fn draw_ellipse_poly(x: f32, y: f32, rx: f32, ry: f32, rot: f32, n: u32, c: Color) {
    let (s, co) = rot.sin_cos();
    let mut pts = Vec::with_capacity(n as usize + 1);
    pts.push([x, y]);
    for i in 0..n {
        let t = i as f32 / n as f32 * std::f32::consts::TAU;
        let (px, py) = (t.cos() * rx, t.sin() * ry);
        pts.push([x + px * co - py * s, y + px * s + py * co]);
    }
    let idx: Vec<u32> = (0..n).flat_map(|i| [0, 1 + i, 1 + (i + 1) % n]).collect();
    with(|f| f.canvas.mesh(&pts, &[c.render()], &idx));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterMode { Linear, Nearest }

struct TexInner { slot: Cell<Option<(u32, ume_render::Rect)>>, pixels: Rc<[u8]>, w: u32, h: u32 }

#[derive(Clone)]
pub struct Texture2D(Rc<TexInner>);

impl Texture2D {
    pub fn from_rgba8(w: u16, h: u16, rgba: &[u8]) -> Texture2D {
        let (w, h) = (w as u32, h as u32);
        Texture2D(Rc::new(TexInner { slot: Cell::new(None), pixels: rgba[..(w * h * 4) as usize].into(), w, h }))
    }

    fn rect(&self, f: &mut Frame) -> Option<ume_render::Rect> {
        let gen = generation();
        if let Some((g, r)) = self.0.slot.get() { if g == gen { return Some(r); } }
        let Some(r) = f.gpu.rgba_atlas.alloc(self.0.w, self.0.h) else { ATLAS_FULL.with(|a| a.set(true)); return None };
        f.gpu.renderer.upload_rgba(&f.gpu.queue, r, &self.0.pixels);
        self.0.slot.set(Some((gen, r)));
        Some(r)
    }

    pub fn width(&self) -> f32 { self.0.w as f32 }
    pub fn height(&self) -> f32 { self.0.h as f32 }
    pub fn size(&self) -> Vec2 { vec2(self.width(), self.height()) }
    pub fn set_filter(&self, _: FilterMode) {}
    pub fn is_resident(&self) -> bool { self.0.slot.get().is_some_and(|(g, _)| g == generation()) }
}

#[derive(Clone, Debug, Default)]
pub struct DrawTextureParams { pub dest_size: Option<Vec2>, pub source: Option<Rect>, pub rotation: f32, pub flip_x: bool, pub flip_y: bool, pub pivot: Option<Vec2> }

pub fn draw_texture(t: &Texture2D, x: f32, y: f32, c: Color) { draw_texture_ex(t, x, y, c, DrawTextureParams::default()) }

pub fn draw_texture_ex(t: &Texture2D, x: f32, y: f32, c: Color, p: DrawTextureParams) {
    let src = p.source.unwrap_or(Rect::new(0.0, 0.0, t.width(), t.height()));
    let size = p.dest_size.unwrap_or(vec2(src.w, src.h));
    with(|f| {
        let Some(r) = t.rect(f) else { return };
        let [aw, ah] = f.gpu.renderer.atlas_size.map(|v| v as f32);
        let (mut u0, mut v0) = ((r.x as f32 + src.x) / aw, (r.y as f32 + src.y) / ah);
        let (mut u1, mut v1) = ((r.x as f32 + src.x + src.w) / aw, (r.y as f32 + src.y + src.h) / ah);
        if p.flip_x { std::mem::swap(&mut u0, &mut u1); }
        if p.flip_y { std::mem::swap(&mut v0, &mut v1); }
        let pivot = p.pivot.unwrap_or(vec2(x + size.x * 0.5, y + size.y * 0.5));
        let (s, co) = p.rotation.sin_cos();
        let corner = |cx: f32, cy: f32| {
            let (dx, dy) = (cx - pivot.x, cy - pivot.y);
            [pivot.x + dx * co - dy * s, pivot.y + dx * s + dy * co]
        };
        let v = |pos: [f32; 2], uv: [f32; 2]| ume_render::Vertex { pos, uv, pal: RGBA_SPRITE, mul: c.to_array(), add: [0.0; 4] };
        let verts = [
            v(corner(x, y), [u0, v0]),
            v(corner(x + size.x, y), [u1, v0]),
            v(corner(x + size.x, y + size.y), [u1, v1]),
            v(corner(x, y + size.y), [u0, v1]),
        ];
        let blend = f.canvas.blend;
        f.canvas.batch.push_tris(&verts, &[0, 1, 2, 0, 2, 3], blend);
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex { pub position: Vec3, pub uv: Vec2, pub color: [u8; 4], pub normal: Vec4 }

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32, color: Color) -> Vertex { Vertex { position: vec3(x, y, z), uv: vec2(u, v), color: color.into(), normal: Vec4::ZERO } }
}

pub struct Mesh { pub vertices: Vec<Vertex>, pub indices: Vec<u16>, pub texture: Option<Texture2D> }

pub fn draw_mesh(m: &Mesh) {
    with(|f| {
        let [aw, ah] = f.gpu.renderer.atlas_size.map(|v| v as f32);
        let rect = match m.texture.as_ref() { Some(t) => match t.rect(f) { Some(r) => Some(r), None => return }, None => None };
        let white = f.canvas.white_uv();
        let verts: Vec<ume_render::Vertex> = m.vertices.iter().map(|v| {
            let uv = match rect { Some(r) => [(r.x as f32 + v.uv.x * r.w as f32) / aw, (r.y as f32 + v.uv.y * r.h as f32) / ah], None => white };
            ume_render::Vertex { pos: [v.position.x, v.position.y], uv, pal: RGBA_SPRITE, mul: v.color.map(|c| c as f32 / 255.0), add: [0.0; 4] }
        }).collect();
        let idx: Vec<u32> = m.indices.iter().map(|&i| i as u32).collect();
        let blend = f.canvas.blend;
        f.canvas.batch.push_tris(&verts, &idx, blend);
    })
}

#[derive(Clone)]
pub struct Font(Rc<RefCell<(text::Font, u32)>>);

impl Font {
    pub fn set_filter(&mut self, _: FilterMode) {}
}

pub fn load_ttf_font_from_bytes(bytes: &[u8]) -> Result<Font, String> { text::Font::from_bytes(bytes).map(|f| Font(Rc::new(RefCell::new((f, generation()))))) }

pub fn set_default_font(f: Font) { DEFAULT_FONT.with(|d| *d.borrow_mut() = Some(f)); }

fn font_or_default(f: Option<&Font>) -> Option<Font> { f.cloned().or_else(|| DEFAULT_FONT.with(|d| d.borrow().clone())) }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextDimensions { pub width: f32, pub height: f32, pub offset_y: f32 }

#[derive(Clone)]
pub struct TextParams<'a> { pub font: Option<&'a Font>, pub font_size: u16, pub font_scale: f32, pub font_scale_aspect: f32, pub rotation: f32, pub color: Color }

impl Default for TextParams<'_> {
    fn default() -> Self { TextParams { font: None, font_size: 20, font_scale: 1.0, font_scale_aspect: 1.0, rotation: 0.0, color: WHITE } }
}

pub fn measure_text(text: &str, font: Option<&Font>, font_size: u16, font_scale: f32) -> TextDimensions {
    let Some(f) = font_or_default(font) else { return TextDimensions::default() };
    let f = &f.0.borrow().0;
    let px = font_size as f32 * font_scale;
    let [width, height] = f.measure(text, px);
    TextDimensions { width, height, offset_y: f.ascent(px) }
}

pub fn draw_text_ex(text: &str, x: f32, y: f32, p: TextParams) -> TextDimensions {
    let Some(font) = font_or_default(p.font) else { return TextDimensions::default() };
    let px = p.font_size as f32 * p.font_scale;
    let width = with(|f| {
        let mut fb = font.0.borrow_mut();
        let gen = generation();
        if fb.1 != gen { fb.0.clear_cache(); fb.1 = gen; }
        let w = fb.0.draw(f.gpu, f.canvas, text, x, y, px, p.color.render());
        if fb.0.atlas_full() { ATLAS_FULL.with(|a| a.set(true)); }
        w
    });
    let fb = &font.0.borrow().0;
    TextDimensions { width, height: fb.measure("", px)[1], offset_y: fb.ascent(px) }
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
    draw_text_ex(text, x, y, TextParams { font_size: font_size.max(1.0) as u16, color, ..Default::default() })
}

macro_rules! keys {
    ($($ours:ident => $theirs:ident),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum KeyCode { $($ours),* }

        impl KeyCode {
            pub const ALL: &'static [KeyCode] = &[$(KeyCode::$ours),*];
            pub fn winit(self) -> crate::Key { match self { $(KeyCode::$ours => crate::Key::$theirs),* } }
            pub fn from_winit(k: crate::Key) -> Option<KeyCode> { match k { $(crate::Key::$theirs => Some(KeyCode::$ours),)* _ => None } }
        }
    };
}

keys! {
    A => KeyA, B => KeyB, C => KeyC, D => KeyD, E => KeyE, F => KeyF,
    G => KeyG, H => KeyH, I => KeyI, J => KeyJ, K => KeyK, L => KeyL,
    M => KeyM, N => KeyN, O => KeyO, P => KeyP, Q => KeyQ, R => KeyR,
    S => KeyS, T => KeyT, U => KeyU, V => KeyV, W => KeyW, X => KeyX,
    Y => KeyY, Z => KeyZ, Key0 => Digit0, Key1 => Digit1, Key2 => Digit2, Key3 => Digit3,
    Key4 => Digit4, Key5 => Digit5, Key6 => Digit6, Key7 => Digit7, Key8 => Digit8, Key9 => Digit9,
    Kp0 => Numpad0, Kp1 => Numpad1, Kp2 => Numpad2, Kp3 => Numpad3, Kp4 => Numpad4, Kp5 => Numpad5,
    Kp6 => Numpad6, Kp7 => Numpad7, Kp8 => Numpad8, Kp9 => Numpad9, F1 => F1, F2 => F2,
    F3 => F3, F4 => F4, F5 => F5, F6 => F6, F7 => F7, F8 => F8,
    F9 => F9, F10 => F10, F11 => F11, F12 => F12, Up => ArrowUp, Down => ArrowDown,
    Left => ArrowLeft, Right => ArrowRight, Enter => Enter, Escape => Escape, Space => Space, Tab => Tab,
    Backspace => Backspace, Delete => Delete, Insert => Insert, Home => Home, End => End, PageUp => PageUp,
    PageDown => PageDown, LeftShift => ShiftLeft, RightShift => ShiftRight, LeftControl => ControlLeft, RightControl => ControlRight, LeftAlt => AltLeft,
    RightAlt => AltRight, Apostrophe => Quote, Backslash => Backslash, Comma => Comma, Equal => Equal, Minus => Minus,
    Period => Period, Semicolon => Semicolon, Slash => Slash, LeftBracket => BracketLeft, RightBracket => BracketRight, GraveAccent => Backquote,
    KpAdd => NumpadAdd, KpSubtract => NumpadSubtract, KpMultiply => NumpadMultiply, KpDivide => NumpadDivide, KpDecimal => NumpadDecimal, KpEnter => NumpadEnter,
}

pub fn is_key_down(k: KeyCode) -> bool { with(|f| f.input.is_down(k.winit())) }
pub fn is_key_pressed(k: KeyCode) -> bool { with(|f| f.input.is_pressed(k.winit())) }
pub fn get_last_key_pressed() -> Option<KeyCode> { KEY_QUEUE.with(|q| q.borrow_mut().pop()) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton { Left }

pub fn is_mouse_button_down(_: MouseButton) -> bool { with(|f| f.input.mouse_down) }
pub fn is_mouse_button_pressed(_: MouseButton) -> bool { with(|f| f.input.mouse_pressed) }
pub fn mouse_position() -> (f32, f32) { with(|f| (f.input.mouse[0], f.input.mouse[1])) }

pub type TouchPhase = Phase;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Touch { pub id: u64, pub phase: TouchPhase, pub position: Vec2 }

pub fn touches() -> Vec<Touch> { with(|f| f.input.touches.iter().map(|t| Touch { id: t.id, phase: t.phase, position: Vec2::from(t.position) }).collect()) }

pub mod rand {
    use super::RNG;

    pub fn srand(seed: u64) { RNG.with(|r| r.set(seed.max(1))); }

    pub fn gen_range(lo: f32, hi: f32) -> f32 {
        let x = RNG.with(|r| {
            let mut x = r.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            r.set(x);
            x
        });
        lo + (hi - lo) * ((x >> 40) as f32 / (1u64 << 24) as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_frame_yields_once() {
        let mut nf = Box::pin(next_frame());
        let mut cx = Context::from_waker(Waker::noop());
        assert!(nf.as_mut().poll(&mut cx).is_pending());
        assert!(nf.as_mut().poll(&mut cx).is_ready());
    }

    #[test]
    fn rand_stays_in_range() {
        rand::srand(7);
        assert!((0..1000).map(|_| rand::gen_range(-0.5, 0.5)).all(|v| (-0.5..0.5).contains(&v)));
    }

    #[test]
    fn colors_convert() {
        let c = Color::from_rgba(255, 0, 128, 255);
        assert_eq!(<[u8; 4]>::from(c), [255, 0, 128, 255]);
        assert_eq!(KeyCode::Kp5.winit(), crate::Key::Numpad5);
        assert!(Rect::new(0.0, 0.0, 2.0, 2.0).contains(vec2(1.0, 1.0)));
    }
}
