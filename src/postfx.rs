//! Post-processing: scene → render target → (bright-pass → 2×separable blur at ¼ res) → composite shader with
//! bloom, chromatic aberration, radial zoom blur, vignette, colour grading, hit flash and film grain.
use crate::vfx::FxParams;
use macroquad::miniquad::{UniformDesc, UniformType};
use macroquad::prelude::*;

const VS: &str = "#version 100
attribute vec3 position; attribute vec2 texcoord; attribute vec4 color0;
varying lowp vec2 uv; varying lowp vec4 color;
uniform mat4 Model; uniform mat4 Projection;
void main() { gl_Position = Projection * Model * vec4(position, 1); color = color0 / 255.0; uv = texcoord; }";

const BRIGHT: &str = "#version 100
precision mediump float;
varying lowp vec2 uv; uniform sampler2D Texture;
void main() {
  vec3 c = texture2D(Texture, uv).rgb;
  float l = max(c.r, max(c.g, c.b));
  float k = smoothstep(0.58, 0.95, l);
  gl_FragColor = vec4(c * k * 1.15, 1.0);
}";

const BLUR: &str = "#version 100
precision mediump float;
varying lowp vec2 uv; uniform sampler2D Texture; uniform vec2 Dir;
void main() {
  vec3 s = texture2D(Texture, uv).rgb * 0.2270270270;
  s += (texture2D(Texture, uv + Dir * 1.3846153846).rgb + texture2D(Texture, uv - Dir * 1.3846153846).rgb) * 0.3162162162;
  s += (texture2D(Texture, uv + Dir * 3.2307692308).rgb + texture2D(Texture, uv - Dir * 3.2307692308).rgb) * 0.0702702703;
  gl_FragColor = vec4(s, 1.0);
}";

const COMPOSITE: &str = "#version 100
precision highp float;
varying lowp vec2 uv; uniform sampler2D Texture; uniform sampler2D Bloom;
uniform vec4 P0; // chroma, vignette, bloom, grain
uniform vec4 P1; // saturation, contrast, flash, time
uniform vec4 P2; // radial, cx, cy, aspect
uniform vec4 P3; // tint rgb, bloom flip
uniform vec4 P4; // invert (anime impact frame)
float hash(vec2 p) { return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453); }
void main() {
  vec2 c = uv - P2.yz;
  float r = length(c * vec2(P2.w, 1.0));
  vec3 col;
  if (P2.x > 0.01 || P0.x > 0.01) {
    vec2 dir = c / max(length(c), 0.0001);
    float ca = P0.x * 0.0045 * (0.25 + r);
    vec3 acc = vec3(0.0);
    for (int i = 0; i < 7; i++) {
      float t = float(i) / 6.0;
      vec2 p = uv - c * P2.x * 0.10 * t;
      acc += vec3(texture2D(Texture, p + dir * ca).r, texture2D(Texture, p).g, texture2D(Texture, p - dir * ca).b);
    }
    col = acc / 7.0;
  } else {
    col = texture2D(Texture, uv).rgb;
  }
  vec2 buv = vec2(uv.x, mix(uv.y, 1.0 - uv.y, P3.w));
  col += texture2D(Bloom, buv).rgb * P0.z;
  float luma = dot(col, vec3(0.299, 0.587, 0.114));
  col = mix(vec3(luma), col, P1.x);
  col = (col - 0.5) * P1.y + 0.5;
  col *= P3.xyz;
  col *= 1.0 - smoothstep(0.30, 0.95, r) * P0.y;
  col += vec3(P1.z);
  if (P4.x > 0.5) { float l2 = dot(col, vec3(0.299, 0.587, 0.114)); col = mix(vec3(1.0) - col, vec3(smoothstep(0.35, 0.65, 1.0 - l2)), 0.55); }
  col += (hash(uv * 1024.0 + P1.w) - 0.5) * P0.w;
  gl_FragColor = vec4(col, 1.0);
}";

pub struct PostFx {
    scene: Option<RenderTarget>,
    half: Option<(RenderTarget, RenderTarget)>,
    size: (u32, u32),
    bright: Material, blur: Material, comp: Material,
    pub level: u8,
    active: bool,
}

fn mat(frag: &str, uniforms: Vec<(&str, UniformType)>, textures: Vec<&str>) -> Material {
    load_material(ShaderSource::Glsl { vertex: VS, fragment: frag }, MaterialParams {
        uniforms: uniforms.into_iter().map(|(n, t)| UniformDesc::new(n, t)).collect(),
        textures: textures.into_iter().map(String::from).collect(),
        ..Default::default()
    }).unwrap()
}

impl PostFx {
    pub fn new() -> Self {
        PostFx {
            scene: None, half: None, size: (0, 0),
            bright: mat(BRIGHT, vec![], vec![]),
            blur: mat(BLUR, vec![("Dir", UniformType::Float2)], vec![]),
            comp: mat(COMPOSITE, vec![("P0", UniformType::Float4), ("P1", UniformType::Float4), ("P2", UniformType::Float4), ("P3", UniformType::Float4), ("P4", UniformType::Float4)], vec!["Bloom"]),
            level: 2, active: false,
        }
    }
    fn phys() -> (u32, u32) {
        let d = macroquad::miniquad::window::dpi_scale();
        (((screen_width() * d) as u32).max(16), ((screen_height() * d) as u32).max(16))
    }
    fn cam(rt: &RenderTarget, w: f32, h: f32) -> Camera2D { Camera2D { render_target: Some(rt.clone()), ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h)) } }

    /// Start rendering the scene into the offscreen target (no-op when post-processing is off).
    pub fn begin(&mut self) {
        self.active = self.level > 0;
        if !self.active { return; }
        let size = Self::phys();
        if self.scene.is_none() || self.size != size {
            self.size = size;
            let rt = render_target_msaa(size.0, size.1);
            rt.texture.set_filter(FilterMode::Linear);
            self.scene = Some(rt);
            let (hw, hh) = ((size.0 / 4).max(8), (size.1 / 4).max(8));
            let (a, b) = (render_target(hw, hh), render_target(hw, hh));
            a.texture.set_filter(FilterMode::Linear); b.texture.set_filter(FilterMode::Linear);
            self.half = Some((a, b));
        }
        let (w, h) = (screen_width(), screen_height());
        set_camera(&Self::cam(self.scene.as_ref().unwrap(), w, h));
        clear_background(BLACK);
    }

    /// Finish: bloom passes + composite onto the screen.
    pub fn end(&mut self, p: &FxParams, t: f32) {
        if !self.active { return; }
        let (w, h) = (screen_width(), screen_height());
        let scene = self.scene.clone().unwrap();
        let bloom_on = self.level >= 2 && p.bloom > 0.001;
        let (hw, hh) = ((self.size.0 / 4).max(8) as f32, (self.size.1 / 4).max(8) as f32);
        if bloom_on {
            let (a, b) = self.half.clone().unwrap();
            let pass = |src: &Texture2D, dst: &RenderTarget, m: &Material| {
                set_camera(&Self::cam(dst, hw, hh));
                clear_background(BLACK);
                gl_use_material(m);
                draw_texture_ex(src, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(hw, hh)), ..Default::default() });
                gl_use_default_material();
            };
            pass(&scene.texture, &a, &self.bright);
            self.blur.set_uniform("Dir", vec2(1.6 / hw, 0.0));
            pass(&a.texture, &b, &self.blur);
            self.blur.set_uniform("Dir", vec2(0.0, 1.6 / hh));
            pass(&b.texture, &a, &self.blur);
            // wider second pair for a soft halo
            self.blur.set_uniform("Dir", vec2(3.4 / hw, 0.0));
            pass(&a.texture, &b, &self.blur);
            self.blur.set_uniform("Dir", vec2(0.0, 3.4 / hh));
            pass(&b.texture, &a, &self.blur);
            self.comp.set_texture("Bloom", a.texture.clone());
        } else {
            self.comp.set_texture("Bloom", scene.texture.clone());
        }
        set_default_camera();
        self.comp.set_uniform("P0", vec4(p.chroma, p.vignette, if bloom_on { p.bloom } else { 0.0 }, 0.035));
        self.comp.set_uniform("P1", vec4(p.saturation, 1.07, p.flash, (t * 60.0).floor() % 97.0));
        self.comp.set_uniform("P2", vec4(p.radial, p.center[0], p.center[1], w / h));
        self.comp.set_uniform("P3", vec4(p.tint[0], p.tint[1], p.tint[2], BLOOM_FLIP));
        self.comp.set_uniform("P4", vec4(p.invert, 0.0, 0.0, 0.0));
        gl_use_material(&self.comp);
        draw_texture_ex(&scene.texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), flip_y: SCENE_FLIP, ..Default::default() });
        gl_use_default_material();
    }
}

/// Render-target textures are bottom-up in GL: flags verified against real screenshots.
const SCENE_FLIP: bool = true;
const BLOOM_FLIP: f32 = 0.0;
