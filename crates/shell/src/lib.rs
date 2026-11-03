pub mod sprites;
pub mod text;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use ume_render::{Atlas, Canvas, PostFx, Renderer, Target};
pub use ume_render::PostParams;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

pub use winit::keyboard::KeyCode as Key;

#[derive(Clone, Debug)]
pub struct Config { pub title: String, pub width: u32, pub height: u32, pub fullscreen: bool, pub max_frames: Option<u64> }

impl Default for Config {
    fn default() -> Self { Config { title: "Ultimate MUGEN".into(), width: 1280, height: 720, fullscreen: false, max_frames: None } }
}

#[derive(Default)]
pub struct Input { down: HashSet<KeyCode>, pressed: HashSet<KeyCode> }

impl Input {
    pub fn is_down(&self, k: KeyCode) -> bool { self.down.contains(&k) }
    pub fn is_pressed(&self, k: KeyCode) -> bool { self.pressed.contains(&k) }
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
    pub quit: bool,
}

pub trait Game {
    fn init(&mut self, _gpu: &mut Gpu) {}
    fn frame(&mut self, f: &mut Frame);
}

struct Live { window: Arc<Window>, surface: wgpu::Surface<'static>, config: wgpu::SurfaceConfiguration, gpu: Gpu, canvas: Canvas, post: PostFx }

struct Shell<G: Game> { cfg: Config, game: G, live: Option<Live>, input: Input, start: Instant, last: Instant, frames: u64, error: Option<String> }

pub const ATLAS_SIZE: u32 = 4096;

fn init_live(event_loop: &ActiveEventLoop, cfg: &Config) -> Result<Live, String> {
    let mut attrs = Window::default_attributes().with_title(cfg.title.clone()).with_inner_size(PhysicalSize::new(cfg.width, cfg.height));
    if cfg.fullscreen { attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(None))); }
    let window = Arc::new(event_loop.create_window(attrs).map_err(|e| e.to_string())?);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(event_loop.owned_display_handle())));
    let surface = instance.create_surface(window.clone()).map_err(|e| e.to_string())?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() })).map_err(|e| e.to_string())?;
    let limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { required_limits: limits, ..Default::default() })).map_err(|e| e.to_string())?;
    let size = window.inner_size();
    let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).ok_or("surface unsupported by adapter")?;
    if let Some(f) = surface.get_capabilities(&adapter).formats.into_iter().find(|f| !f.is_srgb()) { config.format = f; }
    surface.configure(&device, &config);
    let side = ATLAS_SIZE.min(device.limits().max_texture_dimension_2d);
    let renderer = Renderer::new(&device, config.format, [side, side]);
    let mut rgba_atlas = Atlas::new(side, side);
    let canvas = Canvas::new(&renderer, &queue, &mut rgba_atlas).ok_or("atlas too small")?;
    let post = PostFx::new(&device, config.format);
    let gpu = Gpu { device, queue, renderer, rgba_atlas, index_atlas: Atlas::new(side, side) };
    Ok(Live { window, surface, config, gpu, canvas, post })
}

impl<G: Game> Shell<G> {
    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        let Some(live) = self.live.as_mut() else { return };
        let frame = match live.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            _ => { live.surface.configure(&live.gpu.device, &live.config); return; }
        };
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32();
        self.last = now;
        live.canvas.clear();
        let size = [live.config.width as f32, live.config.height as f32];
        let mut f = Frame { gpu: &mut live.gpu, canvas: &mut live.canvas, input: &self.input, size, dt, time: (now - self.start).as_secs_f64(), clear: [0.0, 0.0, 0.0, 1.0], post: None, quit: false };
        self.game.frame(&mut f);
        let (quit, c, post) = (f.quit, f.clear, f.post);
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let gpu = &mut live.gpu;
        let mut enc = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let clear = Some(wgpu::Color { r: c[0], g: c[1], b: c[2], a: c[3] });
        match post {
            Some(p) => {
                let scene = live.post.scene_view(&gpu.device, [live.config.width, live.config.height]).clone();
                gpu.renderer.draw(&gpu.device, &gpu.queue, &mut enc, Target { view: &scene, size, clear }, &live.canvas.batch);
                live.post.apply(&gpu.queue, &mut enc, &view, &p);
            }
            None => gpu.renderer.draw(&gpu.device, &gpu.queue, &mut enc, Target { view: &view, size, clear }, &live.canvas.batch),
        }
        gpu.queue.submit([enc.finish()]);
        live.window.pre_present_notify();
        live.gpu.queue.present(frame);
        self.input.pressed.clear();
        self.frames += 1;
        if quit || self.cfg.max_frames.is_some_and(|m| self.frames >= m) { event_loop.exit(); return; }
        live.window.request_redraw();
    }
}

impl<G: Game> ApplicationHandler for Shell<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() { return; }
        match init_live(event_loop, &self.cfg) {
            Ok(mut live) => { self.game.init(&mut live.gpu); live.window.request_redraw(); self.live = Some(live); }
            Err(e) => { self.error = Some(e); event_loop.exit(); }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(s) => if let Some(live) = self.live.as_mut() {
                live.config.width = s.width.max(1);
                live.config.height = s.height.max(1);
                live.surface.configure(&live.gpu.device, &live.config);
            },
            WindowEvent::KeyboardInput { event, .. } => if let PhysicalKey::Code(k) = event.physical_key {
                match event.state {
                    ElementState::Pressed => { if !event.repeat { self.input.pressed.insert(k); } self.input.down.insert(k); }
                    ElementState::Released => { self.input.down.remove(&k); }
                }
            },
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            _ => {}
        }
    }
}

pub fn run<G: Game>(cfg: Config, game: G) -> Result<u64, String> {
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let now = Instant::now();
    let mut shell = Shell { cfg, game, live: None, input: Input::default(), start: now, last: now, frames: 0, error: None };
    event_loop.run_app(&mut shell).map_err(|e| e.to_string())?;
    match shell.error { Some(e) => Err(e), None => Ok(shell.frames) }
}
