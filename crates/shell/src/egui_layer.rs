use winit::event::WindowEvent;
use winit::window::Window;

pub struct Layer {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    jobs: Vec<egui::ClippedPrimitive>,
    screen: egui_wgpu::ScreenDescriptor,
}

impl Layer {
    pub fn new(window: &Window, device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let ctx = egui::Context::default();
        let state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        Self {
            ctx,
            state,
            renderer: egui_wgpu::Renderer::new(
                device,
                format,
                egui_wgpu::RendererOptions::default(),
            ),
            jobs: Vec::new(),
            screen: egui_wgpu::ScreenDescriptor {
                size_in_pixels: [1, 1],
                pixels_per_point: 1.0,
            },
        }
    }

    pub fn context(&self) -> &egui::Context {
        &self.ctx
    }

    pub fn on_event(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn run(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
        mut ui: impl FnMut(&egui::Context),
    ) {
        let input = self.state.take_egui_input(window);
        let mut out = self.ctx.run_ui(input, |root| ui(root.ctx()));
        self.state
            .handle_platform_output(window, std::mem::take(&mut out.platform_output));
        for (id, deltas) in &out.textures_delta.set {
            for delta in deltas {
                self.renderer.update_texture(device, queue, *id, delta);
            }
        }
        let free = std::mem::take(&mut out.textures_delta.free);
        out.textures_delta.clear();
        for id in &free {
            self.renderer.free_texture(id);
        }
        self.jobs = self.ctx.tessellate(out.shapes, out.pixels_per_point);
        self.screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: out.pixels_per_point,
        };
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Vec<wgpu::CommandBuffer> {
        let extra = self
            .renderer
            .update_buffers(device, queue, enc, &self.jobs, &self.screen);
        let pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        self.renderer
            .render(&mut pass.forget_lifetime(), &self.jobs, &self.screen);
        extra
    }
}
