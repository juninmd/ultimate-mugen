pub const POST_SHADER: &str = include_str!("post.wgsl");

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
}

impl Default for PostParams {
    fn default() -> Self {
        PostParams { chroma: 0.0, vignette: 0.0, bloom: 0.0, grain: 0.0, saturation: 1.0, contrast: 1.0, flash: 0.0, time: 0.0, radial: 0.0, center: [0.5, 0.5], tint: [1.0; 3], invert: 0.0 }
    }
}

const SLOT: u64 = 256;
const PASSES: usize = 6;

struct Targets { size: [u32; 2], scene: wgpu::Texture, scene_view: wgpu::TextureView, groups: [wgpu::BindGroup; PASSES], half: [wgpu::TextureView; 2], half_size: [u32; 2] }

pub struct PostFx {
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
    bright: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
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
        PostFx { format, layout, bright: pipeline("fs_bright"), blur: pipeline("fs_blur"), composite: pipeline("fs_composite"), uniforms, sampler, targets: None }
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
        let group = |slot: usize, src: &wgpu::TextureView, bloom: &wgpu::TextureView| device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ume-post-bind"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &self.uniforms, offset: slot as u64 * SLOT, size: wgpu::BufferSize::new(80) }) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(src) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(bloom) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });
        let [a, b] = &half;
        let groups = [group(0, &scene_view, &scene_view), group(1, a, a), group(2, b, b), group(3, a, a), group(4, b, b), group(5, &scene_view, a)];
        Targets { size, scene, scene_view, groups, half, half_size }
    }

    pub fn apply(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, p: &PostParams) {
        let Some(t) = self.targets.as_ref() else { return };
        let bloom_on = p.bloom > 0.001;
        let [hw, hh] = [t.half_size[0] as f32, t.half_size[1] as f32];
        let mut data = [0f32; (SLOT as usize / 4) * PASSES];
        let mut slot = |i: usize, v: [f32; 20]| data[i * SLOT as usize / 4..][..20].copy_from_slice(&v);
        let dir = |x: f32, y: f32| { let mut v = [0f32; 20]; v[0] = x; v[1] = y; v };
        slot(1, dir(1.6 / hw, 0.0));
        slot(2, dir(0.0, 1.6 / hh));
        slot(3, dir(3.4 / hw, 0.0));
        slot(4, dir(0.0, 3.4 / hh));
        let aspect = t.size[0] as f32 / t.size[1] as f32;
        slot(5, [
            p.chroma, p.vignette, if bloom_on { p.bloom } else { 0.0 }, p.grain,
            p.saturation, p.contrast, p.flash, p.time,
            p.radial, p.center[0], p.center[1], aspect,
            p.tint[0], p.tint[1], p.tint[2], 0.0,
            p.invert, 0.0, 0.0, 0.0,
        ]);
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
        pass(&self.composite, &t.groups[5], target);
    }
}
