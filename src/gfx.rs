#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod imp {
    pub use ume_shell::audio;
    pub use ume_shell::quad::*;
    pub use ume_shell::quad::Blend as GfxBlend;
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
mod imp {
    pub use macroquad::audio;
    pub use macroquad::models::{draw_mesh, Mesh, Vertex};
    pub use macroquad::prelude::*;
    use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams};

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum GfxBlend { Alpha, Add, Sub }

    const VS: &str = "#version 100
attribute vec3 position; attribute vec2 texcoord; attribute vec4 color0;
varying lowp vec2 uv; varying lowp vec4 color;
uniform mat4 Model; uniform mat4 Projection;
void main() { gl_Position = Projection * Model * vec4(position, 1); color = color0 / 255.0; uv = texcoord; }";
    const FS: &str = "#version 100
varying lowp vec4 color; varying lowp vec2 uv; uniform sampler2D Texture;
void main() { gl_FragColor = color * texture2D(Texture, uv); }";

    fn material(eq: Equation) -> Material {
        load_material(ShaderSource::Glsl { vertex: VS, fragment: FS }, MaterialParams {
            pipeline_params: PipelineParams { color_blend: Some(BlendState::new(eq, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One)), ..Default::default() },
            ..Default::default()
        }).unwrap()
    }

    thread_local! {
        static MATERIALS: (Material, Material) = (material(Equation::Add), material(Equation::ReverseSubtract));
    }

    pub fn set_blend(b: GfxBlend) {
        match b {
            GfxBlend::Alpha => gl_use_default_material(),
            GfxBlend::Add => MATERIALS.with(|m| gl_use_material(&m.0)),
            GfxBlend::Sub => MATERIALS.with(|m| gl_use_material(&m.1)),
        }
    }

    pub fn set_scissor(r: Option<(i32, i32, i32, i32)>) {
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        gl.quad_gl.scissor(r);
    }

    pub fn dpi_scale() -> f32 { macroquad::miniquad::window::dpi_scale() }
}

pub use imp::*;
