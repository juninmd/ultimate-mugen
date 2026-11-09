pub const POST_SHADER: &str = include_str!("post.wgsl");

pub const MAX_GLASS: usize = 16;

use crate::grade::{Grade, Waves};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Glass { pub n: usize, pub rects: [[f32; 8]; MAX_GLASS] }

impl Glass {
    pub fn push(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, tint: [f32; 3]) -> bool {
        if self.n >= MAX_GLASS || w <= 0.0 || h <= 0.0 { return false; }
        self.rects[self.n] = [x, y, x + w, y + h, radius.min(w / 2.0).min(h / 2.0), tint[0], tint[1], tint[2]];
        self.n += 1;
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PostParams {
    pub chroma: f32,
    pub vignette: f32,
    pub bloom: f32,
    pub grain: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub flash: f32,
    pub time: f32,
    pub radial: f32,
    pub center: [f32; 2],
    pub tint: [f32; 3],
    pub invert: f32,
    pub glass: Glass,
    pub grade: Grade,
    pub waves: Waves,
}

impl Default for PostParams {
    fn default() -> Self {
        PostParams { chroma: 0.0, vignette: 0.0, bloom: 0.0, grain: 0.0, saturation: 1.0, contrast: 1.0, flash: 0.0, time: 0.0, radial: 0.0, center: [0.5, 0.5], tint: [1.0; 3], invert: 0.0, glass: Glass::default(), grade: Grade::default(), waves: Waves::default() }
    }
}

const SLOT: u64 = 1024;
const HEAD: usize = 68;
const UNIFORM_BYTES: u64 = HEAD as u64 * 4 + MAX_GLASS as u64 * 32;
const PASSES: usize = 6;

struct Targets { size: [u32; 2], scene: wgpu::Texture, scene_view: wgpu::TextureView, groups: [wgpu::BindGroup; PASSES], glass_groups: [wgpu::BindGroup; 4], half: [wgpu::TextureView; 2], glass: [wgpu::TextureView; 2], half_size: [u32; 2] }

pub struct PostFx {
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
    bright: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    copy: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    sampler: wgpu::Sampler,
    targets: Option<Targets>,
}

fn color_target(device: &wgpu::Device, label: &str, size: [u32; 2], format: wgpu::TextureFormat) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

impl PostFx {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let tex = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ume-post-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                tex(1),
                tex(2),
                tex(4),
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("ume-post"), source: wgpu::ShaderSource::Wgsl(POST_SHADER.into()) });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("ume-post-pl"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = |entry: &str| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(entry),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_full"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some(entry), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            multiview_mask: None,
            cache: None,
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor { label: Some("ume-post-uniforms"), size: SLOT * PASSES as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("ume-post-linear"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
        PostFx { format, layout, bright: pipeline("fs_bright"), blur: pipeline("fs_blur"), copy: pipeline("fs_copy"), composite: pipeline("fs_composite"), uniforms, sampler, targets: None }
    }

    pub fn scene_texture(&self) -> Option<&wgpu::Texture> { self.targets.as_ref().map(|t| &t.scene) }

    pub fn scene_view(&mut self, device: &wgpu::Device, size: [u32; 2]) -> &wgpu::TextureView {
        let size = [size[0].max(16), size[1].max(16)];
        if self.targets.as_ref().is_none_or(|t| t.size != size) { self.targets = Some(self.build(device, size)); }
        &self.targets.as_ref().unwrap().scene_view
    }

    fn build(&self, device: &wgpu::Device, size: [u32; 2]) -> Targets {
        let half_size = [(size[0] / 4).max(8), (size[1] / 4).max(8)];
        let scene = color_target(device, "ume-post-scene", size, self.format);
        let view = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let scene_view = view(&scene);
        let half = [view(&color_target(device, "ume-post-a", half_size, self.format)), view(&color_target(device, "ume-post-b", half_size, self.format))];
        let glass = [view(&color_target(device, "ume-glass-a", half_size, self.format)), view(&color_target(device, "ume-glass-b", half_size, self.format))];
        let group = |slot: usize, src: &wgpu::TextureView, bloom: &wgpu::TextureView, frost: &wgpu::TextureView| device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ume-post-bind"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &self.uniforms, offset: slot as u64 * SLOT, size: wgpu::BufferSize::new(UNIFORM_BYTES) }) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(src) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(bloom) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(frost) },
            ],
        });
        let [a, b] = &half;
        let [ga, gb] = &glass;
        let groups = [group(0, &scene_view, &scene_view, &scene_view), group(1, a, a, &scene_view), group(2, b, b, &scene_view), group(3, a, a, &scene_view), group(4, b, b, &scene_view), group(5, &scene_view, a, ga)];
        let glass_groups = [group(1, ga, ga, &scene_view), group(2, gb, gb, &scene_view), group(3, ga, ga, &scene_view), group(4, gb, gb, &scene_view)];
        Targets { size, scene, scene_view, groups, glass_groups, half, glass, half_size }
    }

    pub fn apply(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, p: &PostParams) {
        let Some(t) = self.targets.as_ref() else { return };
        let bloom_on = p.bloom > 0.001;
        let [hw, hh] = [t.half_size[0] as f32, t.half_size[1] as f32];
        let mut data = [0f32; (SLOT as usize / 4) * PASSES];
        let g = p.grade;
        let tail = [g.tonemap, g.exposure, g.split, g.heat, 0.0, g.threshold, 0.0, 0.0, g.shadows[0], g.shadows[1], g.shadows[2], p.waves.n as f32, g.highlights[0], g.highlights[1], g.highlights[2], 0.0];
        let mut slot = |i: usize, v: [f32; 20]| { let o = i * SLOT as usize / 4; data[o..o + 20].copy_from_slice(&v); data[o + 20..o + 36].copy_from_slice(&tail); };
        let glass_on = p.glass.n > 0;
        let dir = |x: f32, y: f32| { let mut v = [0f32; 20]; v[0] = x; v[1] = y; v };
        let mut copy = [0f32; 20];
        copy[2] = 1.5 / t.size[0] as f32;
        copy[3] = 1.5 / t.size[1] as f32;
        slot(0, copy);
        slot(1, dir(1.6 / hw, 0.0));
        slot(2, dir(0.0, 1.6 / hh));
        slot(3, dir(3.4 / hw, 0.0));
        slot(4, dir(0.0, 3.4 / hh));
        let aspect = t.size[0] as f32 / t.size[1] as f32;
        slot(5, [
            p.chroma, p.vignette, if bloom_on { p.bloom } else { 0.0 }, p.grain,
            p.saturation, p.contrast, p.flash, p.time,
            p.radial, p.center[0], p.center[1], aspect,
            p.tint[0], p.tint[1], p.tint[2], p.glass.n as f32,
            p.invert, t.size[0] as f32, t.size[1] as f32, (t.size[1] as f32 / 720.0).max(1.0),
        ]);
        for (i, r) in p.glass.rects[..p.glass.n].iter().enumerate() {
            let o = 5 * SLOT as usize / 4 + HEAD + i * 8;
            data[o..o + 8].copy_from_slice(r);
        }
        for (i, w) in p.waves.items[..p.waves.n].iter().enumerate() {
            let o = 5 * SLOT as usize / 4 + 36 + i * 8;
            data[o..o + 8].copy_from_slice(w);
        }
        queue.write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&data));
        let mut pass = |pipeline: &wgpu::RenderPipeline, group: &wgpu::BindGroup, dst: &wgpu::TextureView| {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ume-post-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: dst, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
                ..Default::default()
            });
            rp.set_pipeline(pipeline);
            rp.set_bind_group(0, group, &[]);
            rp.draw(0..3, 0..1);
        };
        let [a, b] = &t.half;
        if bloom_on {
            pass(&self.bright, &t.groups[0], a);
            pass(&self.blur, &t.groups[1], b);
            pass(&self.blur, &t.groups[2], a);
            pass(&self.blur, &t.groups[3], b);
            pass(&self.blur, &t.groups[4], a);
        }
        if glass_on {
            let [ga, gb] = &t.glass;
            pass(&self.copy, &t.groups[0], ga);
            pass(&self.blur, &t.glass_groups[0], gb);
            pass(&self.blur, &t.glass_groups[1], ga);
            pass(&self.blur, &t.glass_groups[2], gb);
            pass(&self.blur, &t.glass_groups[3], ga);
        }
        pass(&self.composite, &t.groups[5], target);
    }
}
