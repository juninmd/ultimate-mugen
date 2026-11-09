use std::sync::Arc;
use winit::window::Window;

pub struct Gfx {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub renderer: egui_wgpu::Renderer,
}

impl Gfx {
    pub fn new(
        instance: &wgpu::Instance,
        window: Arc<Window>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("surface unsupported")?;
        let caps = surface.get_capabilities(&adapter);
        if let Some(f) = caps.formats.iter().copied().find(|f| !f.is_srgb()) {
            config.format = f;
        }
        config.usage |= wgpu::TextureUsages::COPY_SRC;
        surface.configure(&device, &config);
        let renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            egui_wgpu::RendererOptions::default(),
        );
        Ok(Self {
            window,
            surface,
            config,
            device,
            queue,
            renderer,
        })
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.config.width = w.max(1);
        self.config.height = h.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        out: egui::FullOutput,
        clear: wgpu::Color,
        shot: Option<&str>,
    ) {
        let mut out = out;
        let jobs = ctx.tessellate(out.shapes, out.pixels_per_point);
        for (id, deltas) in &out.textures_delta.set {
            for delta in deltas {
                self.renderer
                    .update_texture(&self.device, &self.queue, *id, delta);
            }
        }
        let free = std::mem::take(&mut out.textures_delta.free);
        out.textures_delta.clear();
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            _ => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: out.pixels_per_point,
        };
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let extra =
            self.renderer
                .update_buffers(&self.device, &self.queue, &mut enc, &jobs, &screen);
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.renderer
                .render(&mut pass.forget_lifetime(), &jobs, &screen);
        }
        let readback = shot.map(|_| crate::shot::copy(&self.device, &mut enc, &frame.texture));
        self.queue
            .submit(extra.into_iter().chain(std::iter::once(enc.finish())));
        if let (Some(path), Some(rb)) = (shot, readback) {
            if let Err(e) = rb.save(&self.device, path) {
                eprintln!("screenshot {path}: {e}");
            }
        }
        self.window.pre_present_notify();
        self.queue.present(frame);
        for id in &free {
            self.renderer.free_texture(id);
        }
    }
}
