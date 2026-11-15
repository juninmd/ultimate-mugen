pub mod bars;
pub mod tween;

use vello::peniko::{Color, Fill};
use vello::kurbo::Affine;
use vello::{AaConfig, AaSupport, Renderer, RendererOptions, RenderParams, Scene};

pub use bars::{bar_shapes, colors, BarSpec};
pub use tween::{ease_out_cubic, Tween};

pub struct VectorHud {
    renderer: Renderer,
    scene: Scene,
}

impl VectorHud {
    pub fn new(device: &wgpu::Device) -> Result<Self, vello::Error> {
        let renderer = Renderer::new(
            device,
            RendererOptions {
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                ..RendererOptions::default()
            },
        )?;
        Ok(Self { renderer, scene: Scene::new() })
    }

    pub fn build(&mut self, bars: &[BarSpec]) -> &Scene {
        self.scene.reset();
        for spec in bars {
            let shapes = bar_shapes(spec);
            let cols = colors(spec);
            for (shape, color) in shapes.iter().zip(cols) {
                self.scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, shape);
            }
        }
        &self.scene
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        bars: &[BarSpec],
    ) -> Result<(), vello::Error> {
        self.build(bars);
        self.renderer.render_to_texture(
            device,
            queue,
            &self.scene,
            view,
            &RenderParams {
                base_color: Color::TRANSPARENT,
                width,
                height,
                antialiasing_method: AaConfig::Area,
            },
        )
    }
}
