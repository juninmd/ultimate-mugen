mod art;
mod gfx;
mod menu;
mod shot;

use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    gfx: Option<gfx::Gfx>,
    ctx: egui::Context,
    state: Option<egui_winit::State>,
    menu: menu::Menu,
    start: Instant,
    frames: u32,
    max_frames: Option<u32>,
    shot: Option<String>,
    press_down_at: Option<u32>,
}

fn fonts() -> egui::FontDefinitions {
    let mut f = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut f, egui_phosphor::Variant::Regular);
    f.font_data.insert(
        "russo".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../../assets/fonts/RussoOne.ttf"
        ))),
    );
    f.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "russo".into());
    f
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.gfx.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("ume egui spike")
            .with_inner_size(PhysicalSize::new(1280, 720));
        let window = Arc::new(el.create_window(attrs).unwrap());
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
                Box::new(el.owned_display_handle()),
            ));
        let gfx = gfx::Gfx::new(&instance, window.clone()).unwrap();
        self.ctx.set_fonts(fonts());
        self.state = Some(egui_winit::State::new(
            self.ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(gfx.device.limits().max_texture_dimension_2d as usize),
        ));
        window.request_redraw();
        self.gfx = Some(gfx);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, ev: WindowEvent) {
        let (Some(gfx), Some(state)) = (self.gfx.as_mut(), self.state.as_mut()) else {
            return;
        };
        let _ = state.on_window_event(&gfx.window, &ev);
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => gfx.resize(s.width, s.height),
            WindowEvent::RedrawRequested => {
                let mut input = state.take_egui_input(&gfx.window);
                if Some(self.frames) == self.press_down_at {
                    input.events.push(egui::Event::Key {
                        key: egui::Key::ArrowDown,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                let now = self.start.elapsed().as_secs_f64();
                let menu = &mut self.menu;
                let out = self.ctx.run_ui(input, |ui| menu.ui(ui, now));
                state.handle_platform_output(&gfx.window, out.platform_output.clone());
                let last = self.max_frames.is_some_and(|m| self.frames + 1 >= m);
                let shot = if last { self.shot.as_deref() } else { None };
                gfx.render(&self.ctx, out, wgpu::Color::BLACK, shot);
                self.frames += 1;
                if self.menu.quit || last {
                    el.exit();
                }
                gfx.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let max_frames = std::env::var("UME_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    let mut app = App {
        gfx: None,
        ctx: egui::Context::default(),
        state: None,
        menu: menu::Menu::new(),
        start: Instant::now(),
        frames: 0,
        max_frames,
        shot: std::env::var("UME_SHOT").ok(),
        press_down_at: std::env::var("UME_PRESS_DOWN")
            .ok()
            .and_then(|v| v.parse().ok()),
    };
    EventLoop::new().unwrap().run_app(&mut app).unwrap();
}
