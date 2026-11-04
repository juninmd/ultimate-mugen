pub mod sprites;
pub mod audio;
pub mod quad;
mod screenshot;
use screenshot::Save;
pub mod text;
pub mod watchdog;

use std::collections::HashSet;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use web_time::Instant;
use ume_render::{Atlas, Canvas, PostFx, Renderer, Target};
pub use ume_render::PostParams;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

pub use winit::keyboard::KeyCode as Key;

#[derive(Clone, Debug)]
pub struct Config { pub title: String, pub width: u32, pub height: u32, pub fullscreen: bool, pub max_frames: Option<u64> }

impl Default for Config {
    fn default() -> Self { Config { title: "Ultimate MUGEN".into(), width: 1280, height: 720, fullscreen: false, max_frames: None } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase { Started, Moved, Ended, Cancelled }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Touch { pub id: u64, pub phase: Phase, pub position: [f32; 2] }

#[derive(Default)]
pub struct Input { down: HashSet<KeyCode>, pressed: HashSet<KeyCode>, pub pressed_order: Vec<KeyCode>, pub mouse: [f32; 2], pub mouse_delta: [f32; 2], pub wheel: [f32; 2], pub mouse_down: bool, pub mouse_pressed: bool, pub touches: Vec<Touch>, primary_touch: Option<u64>, pub typed: String, pub pointer_scale: [f32; 2] }

impl Input {
    pub fn is_down(&self, k: KeyCode) -> bool { self.down.contains(&k) }
    pub fn is_pressed(&self, k: KeyCode) -> bool { self.pressed.contains(&k) }

    fn end_frame(&mut self) {
        self.pressed.clear();
        self.pressed_order.clear();
        self.typed.clear();
        self.mouse_pressed = false;
        self.mouse_delta = [0.0; 2];
        self.wheel = [0.0; 2];
        self.touches.retain(|t| !matches!(t.phase, Phase::Ended | Phase::Cancelled));
        for t in &mut self.touches { if t.phase == Phase::Started { t.phase = Phase::Moved; } }
    }

    fn touch(&mut self, id: u64, phase: Phase, position: [f32; 2]) {
        if phase == Phase::Started && self.primary_touch.is_none() { self.primary_touch = Some(id); self.mouse_pressed = true; self.mouse_down = true; }
        if self.primary_touch == Some(id) {
            self.mouse = position;
            if matches!(phase, Phase::Ended | Phase::Cancelled) { self.primary_touch = None; self.mouse_down = false; }
        }
        match self.touches.iter_mut().find(|t| t.id == id) {
            Some(t) => { t.position = position; if t.phase != Phase::Started || phase != Phase::Moved { t.phase = phase; } }
            None => self.touches.push(Touch { id, phase, position }),
        }
    }
}

pub struct Gpu { pub device: wgpu::Device, pub queue: wgpu::Queue, pub renderer: Renderer, pub rgba_atlas: Atlas, pub index_atlas: Atlas }

pub struct Frame<'a> {
    pub gpu: &'a mut Gpu,
    pub canvas: &'a mut Canvas,
    pub input: &'a Input,
    pub size: [f32; 2],
    pub dt: f32,
    pub time: f64,
    pub clear: [f64; 4],
    pub post: Option<PostParams>,
    pub scale: f32,
    pub window: [f32; 2],
    pub stretch: Option<f32>,
    pub linear: bool,
    pub resize: Option<[u32; 2]>,
    pub fullscreen: Option<bool>,
    pub vsync: Option<bool>,
    pub frame_cap: Option<u32>,
    pub screenshot: Option<String>,
    pub quit: bool,
}

pub trait Game {
    fn init(&mut self, _gpu: &mut Gpu) {}
    fn frame(&mut self, f: &mut Frame);
}

struct Live { window: Arc<Window>, surface: wgpu::Surface<'static>, config: wgpu::SurfaceConfiguration, gpu: Gpu, canvas: Canvas, post: PostFx }

pub const STRETCH_ASPECT: f32 = 16.0 / 9.0;

pub fn logical_size(surface: [f32; 2], stretch: Option<f32>) -> [f32; 2] {
    match stretch { Some(aspect) => [surface[1] * aspect, surface[1]], None => surface }
}

struct Shell<G: Game> { stretch: Option<f32>, linear: bool, cfg: Config, game: G, live: Option<Live>, pending: Option<Pending>, input: Input, start: Instant, last: Instant, frames: u64, error: Option<String>, pacer: Pacer, frame_cap: u32, watchdog: watchdog::Watchdog }

#[derive(Default)]
pub struct Pacer { debt: f32 }

impl Pacer {
    const REFRESH_HZ: [f32; 9] = [30.0, 50.0, 60.0, 75.0, 90.0, 120.0, 144.0, 165.0, 240.0];

    pub fn pace(&mut self, raw: f32) -> f32 {
        let raw = raw.clamp(0.0, 0.25);
        let Some(period) = Self::REFRESH_HZ.iter().map(|hz| 1.0 / hz).find(|p| (raw - p).abs() < p * 0.04) else { self.debt = 0.0; return raw };
        self.debt += raw - period;
        if self.debt.abs() < period { return period; }
        let paid = period + self.debt;
        self.debt = 0.0;
        paid
    }
}

pub const ATLAS_SIZE: u32 = 4096;

fn create_window(event_loop: &ActiveEventLoop, cfg: &Config) -> Result<Arc<Window>, String> {
    let mut attrs = Window::default_attributes().with_title(cfg.title.clone());
    #[cfg(not(target_arch = "wasm32"))]
    { attrs = attrs.with_inner_size(PhysicalSize::new(cfg.width, cfg.height)); }
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;
        let canvas = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id("glcanvas")).and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
        attrs = attrs.with_append(canvas.is_none()).with_canvas(canvas).with_prevent_default(true).with_focusable(true);
    }
    if cfg.fullscreen { attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(None))); }
    event_loop.create_window(attrs).map(Arc::new).map_err(|e| e.to_string())
}

fn instance(event_loop: &ActiveEventLoop) -> wgpu::Instance {
    #[cfg(not(target_arch = "wasm32"))]
    let desc = wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(event_loop.owned_display_handle()));
    #[cfg(target_arch = "wasm32")]
    let desc = { let _ = event_loop; let mut d = wgpu::InstanceDescriptor::new_without_display_handle_from_env(); d.backends = wgpu::Backends::GL; d };
    wgpu::Instance::new(desc)
}

async fn init_live(instance: wgpu::Instance, window: Arc<Window>) -> Result<Live, String> {
    let surface = instance.create_surface(window.clone()).map_err(|e| e.to_string())?;
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() }).await.map_err(|e| e.to_string())?;
    let limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor { required_limits: limits, ..Default::default() }).await.map_err(|e| e.to_string())?;
    let size = window.inner_size();
    let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).ok_or("surface unsupported by adapter")?;
    let caps = surface.get_capabilities(&adapter);
    if let Some(f) = caps.formats.iter().copied().find(|f| !f.is_srgb()) { config.format = f; }
    if caps.usages.contains(wgpu::TextureUsages::COPY_SRC) { config.usage |= wgpu::TextureUsages::COPY_SRC; }
    surface.configure(&device, &config);
    let side = ATLAS_SIZE.min(device.limits().max_texture_dimension_2d);
    let renderer = Renderer::new(&device, &queue, config.format, [side, side]);
    let mut rgba_atlas = Atlas::new(side, side);
    let canvas = Canvas::new(&renderer, &queue, &mut rgba_atlas).ok_or("atlas too small")?;
    let post = PostFx::new(&device, config.format);
    let gpu = Gpu { device, queue, renderer, rgba_atlas, index_atlas: Atlas::new(side, side) };
    Ok(Live { window, surface, config, gpu, canvas, post })
}

type Pending = Rc<RefCell<Option<Result<Live, String>>>>;

impl<G: Game> Shell<G> {
    fn adopt(&mut self, event_loop: &ActiveEventLoop) {
        let Some(ready) = self.pending.as_ref().and_then(|p| p.borrow_mut().take()) else { return };
        self.pending = None;
        match ready {
            Ok(mut live) => { self.game.init(&mut live.gpu); live.window.request_redraw(); self.live = Some(live); }
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(&e.clone().into());
                self.error = Some(e);
                event_loop.exit();
            }
        }
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_none() { self.adopt(event_loop); }
        let Some(live) = self.live.as_mut() else { return };
        let actual = live.window.inner_size();
        if actual.width > 0 && actual.height > 0 && (actual.width != live.config.width || actual.height != live.config.height) {
            live.config.width = actual.width;
            live.config.height = actual.height;
            live.surface.configure(&live.gpu.device, &live.config);
        }
        let frame = match live.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => { live.window.request_redraw(); return; }
            _ => { live.surface.configure(&live.gpu.device, &live.config); live.window.request_redraw(); return; }
        };
        let now = Instant::now();
        let dt = self.pacer.pace((now - self.last).as_secs_f32());
        self.last = now;
        live.canvas.clear();
        let surface = [live.config.width as f32, live.config.height as f32];
        let size = logical_size(surface, self.stretch);
        self.input.pointer_scale = [size[0] / surface[0], size[1] / surface[1]];
        let mut f = Frame { gpu: &mut live.gpu, canvas: &mut live.canvas, input: &self.input, size, dt, time: (now - self.start).as_secs_f64(), clear: [0.0, 0.0, 0.0, 1.0], post: None, scale: live.window.scale_factor() as f32, window: surface, stretch: self.stretch, linear: self.linear, resize: None, fullscreen: None, vsync: None, frame_cap: None, screenshot: None, quit: false };
        self.game.frame(&mut f);
        let (quit, c, post, resize, fullscreen, shot, vsync, cap) = (f.quit, f.clear, f.post, f.resize, f.fullscreen, f.screenshot.take(), f.vsync, f.frame_cap);
        if let Some(c) = cap { self.frame_cap = c; }
        (self.stretch, self.linear) = (f.stretch, f.linear);
        let size = logical_size(surface, self.stretch);
        live.gpu.renderer.set_linear(self.linear);
        live.gpu.renderer.set_scissor_scale([surface[0] / size[0], surface[1] / size[1]]);
        if let Some([w, h]) = resize { let _ = live.window.request_inner_size(PhysicalSize::new(w, h)); }
        if let Some(fs) = fullscreen { live.window.set_fullscreen(fs.then_some(Fullscreen::Borderless(None))); }
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let gpu = &mut live.gpu;
        let mut enc = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let clear = Some(wgpu::Color { r: c[0], g: c[1], b: c[2], a: c[3] });
        let offscreen = shot.as_ref().filter(|_| !frame.texture.usage().contains(wgpu::TextureUsages::COPY_SRC)).map(|_| screenshot::target(&gpu.device, &frame.texture));
        let offscreen_view = offscreen.as_ref().map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
        for v in std::iter::once(&view).chain(offscreen_view.as_ref()) {
            match post {
                Some(p) => {
                    let scene = live.post.scene_view(&gpu.device, [live.config.width, live.config.height]).clone();
                    let batch = &live.canvas.batch;
                    gpu.renderer.draw_calls(&gpu.device, &gpu.queue, &mut enc, Target { view: &scene, size, clear }, batch, 0..batch.split());
                    live.post.apply(&gpu.queue, &mut enc, v, &p);
                    gpu.renderer.draw_calls(&gpu.device, &gpu.queue, &mut enc, Target { view: v, size, clear: None }, batch, batch.split()..batch.calls.len());
                }
                None => gpu.renderer.draw(&gpu.device, &gpu.queue, &mut enc, Target { view: v, size, clear }, &live.canvas.batch),
            }
        }
        let readback = shot.map(|path| (path, screenshot::copy(&gpu.device, &mut enc, offscreen.as_ref().unwrap_or(&frame.texture))));
        gpu.queue.submit([enc.finish()]);
        if let Some((path, rb)) = readback {
            if let Err(e) = rb.save(&gpu.device, &path) { eprintln!("screenshot {path}: {e}"); }
        }
        live.window.pre_present_notify();
        live.gpu.queue.present(frame);
        if let Some(on) = vsync {
            let mode = if on { wgpu::PresentMode::AutoVsync } else { wgpu::PresentMode::AutoNoVsync };
            if live.config.present_mode != mode { live.config.present_mode = mode; live.surface.configure(&live.gpu.device, &live.config); }
        }
        self.input.end_frame();
        self.frames += 1;
        self.watchdog.record((Instant::now() - now).as_secs_f32() * 1000.0);
        #[cfg(not(target_arch = "wasm32"))]
        if self.frame_cap > 0 {
            let budget = std::time::Duration::from_secs_f32(1.0 / self.frame_cap as f32);
            let spent = Instant::now() - now;
            if spent < budget { std::thread::sleep(budget - spent); }
        }
        if quit || self.cfg.max_frames.is_some_and(|m| self.frames >= m) { if self.watchdog.log { eprintln!("[watchdog] {}", self.watchdog.summary()); } event_loop.exit(); return; }
        live.window.request_redraw();
    }
}

impl<G: Game> ApplicationHandler for Shell<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() || self.pending.is_some() { return; }
        let window = match create_window(event_loop, &self.cfg) { Ok(w) => w, Err(e) => { self.error = Some(e); event_loop.exit(); return; } };
        let pending: Pending = Rc::new(RefCell::new(None));
        let (slot, instance, w) = (pending.clone(), instance(event_loop), window.clone());
        let task = async move {
            let r = init_live(instance, w.clone()).await;
            *slot.borrow_mut() = Some(r);
            w.request_redraw();
        };
        self.pending = Some(pending);
        #[cfg(not(target_arch = "wasm32"))]
        { pollster::block_on(task); self.adopt(event_loop); }
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        if cfg!(target_os = "android") { self.live = None; self.pending = None; }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(s) => if let Some(live) = self.live.as_mut() {
                let (w, h) = (s.width.max(1), s.height.max(1));
                if (w, h) != (live.config.width, live.config.height) {
                    live.config.width = w;
                    live.config.height = h;
                    live.surface.configure(&live.gpu.device, &live.config);
                }
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Some(t) = &event.text { self.input.typed.extend(t.chars().filter(|c| !c.is_control())); }
                }
                if let PhysicalKey::Code(k) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => { if !event.repeat { self.input.pressed.insert(k); self.input.pressed_order.push(k); } self.input.down.insert(k); }
                        ElementState::Released => { self.input.down.remove(&k); }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let p = [position.x as f32, position.y as f32];
                self.input.mouse_delta = [self.input.mouse_delta[0] + p[0] - self.input.mouse[0], self.input.mouse_delta[1] + p[1] - self.input.mouse[1]];
                self.input.mouse = p;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let [x, y] = match delta { MouseScrollDelta::LineDelta(x, y) => [x, y], MouseScrollDelta::PixelDelta(p) => [p.x as f32 / 40.0, p.y as f32 / 40.0] };
                self.input.wheel = [self.input.wheel[0] + x, self.input.wheel[1] + y];
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let down = state == ElementState::Pressed;
                if down && !self.input.mouse_down { self.input.mouse_pressed = true; }
                self.input.mouse_down = down;
            }
            WindowEvent::Touch(t) => {
                let phase = match t.phase { TouchPhase::Started => Phase::Started, TouchPhase::Moved => Phase::Moved, TouchPhase::Ended => Phase::Ended, TouchPhase::Cancelled => Phase::Cancelled };
                self.input.touch(t.id, phase, [t.location.x as f32, t.location.y as f32]);
            }
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            _ => {}
        }
    }
}

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
pub static ANDROID_APP: std::sync::OnceLock<AndroidApp> = std::sync::OnceLock::new();

fn event_loop() -> Result<EventLoop<()>, String> {
    #[cfg(target_os = "android")]
    {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        let app = ANDROID_APP.get().ok_or("ANDROID_APP not set")?.clone();
        return EventLoop::builder().with_android_app(app).build().map_err(|e| e.to_string());
    }
    #[allow(unreachable_code)]
    EventLoop::new().map_err(|e| e.to_string())
}

pub fn run<G: Game + 'static>(cfg: Config, game: G) -> Result<u64, String> {
    let event_loop = event_loop()?;
    // on the web `Poll` makes winit spin a postTask/AbortController loop next to requestAnimationFrame (GC churn, starved frames); redraws are already self-requested
    event_loop.set_control_flow(if cfg!(target_arch = "wasm32") { ControlFlow::Wait } else { ControlFlow::Poll });
    let now = Instant::now();
    let mut shell = Shell { stretch: None, linear: false, cfg, game, live: None, pending: None, input: Input { pointer_scale: [1.0, 1.0], ..Input::default() }, start: now, last: now, frames: 0, error: None, pacer: Pacer::default(), frame_cap: 0, watchdog: watchdog::Watchdog::from_env() };
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        std::panic::set_hook(Box::new(|info| web_sys::console::error_1(&info.to_string().into())));
        let _ = &mut shell;
        event_loop.spawn_app(shell);
        Ok(0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        event_loop.run_app(&mut shell).map_err(|e| e.to_string())?;
        match shell.error { Some(e) => Err(e), None => Ok(shell.frames) }
    }
}

#[cfg(test)]
mod tests {
    use super::Pacer;

    #[test]
    fn stretch_forces_sixteen_nine_logical_space_on_any_window() {
        assert_eq!(super::logical_size([1000.0, 500.0], None), [1000.0, 500.0]);
        let [w, h] = super::logical_size([1000.0, 900.0], Some(super::STRETCH_ASPECT));
        assert_eq!(h, 900.0);
        assert!((w / h - 16.0 / 9.0).abs() < 1e-5);
    }

    #[test]
    fn jittery_vsync_frames_become_exact_periods() {
        let mut p = Pacer::default();
        for raw in [0.0163, 0.0171, 0.0165, 0.0168, 0.0166] { assert_eq!(p.pace(raw), 1.0 / 60.0); }
    }

    #[test]
    fn long_run_keeps_real_time() {
        let mut p = Pacer::default();
        let raw = 1.0 / 59.5;
        let total: f32 = (0..6000).map(|_| p.pace(raw)).sum();
        assert!((total - raw * 6000.0).abs() < 1.0 / 60.0 + 1e-3);
    }

    #[test]
    fn hitches_pass_through() {
        let mut p = Pacer::default();
        assert_eq!(p.pace(0.05), 0.05);
        assert_eq!(p.pace(1.0), 0.25);
    }
}
