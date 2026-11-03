use crate::atlas::Rect;
use crate::batch::{Batch, Blend, Vertex};
use crate::{PALETTE_ROWS, SHADER};
use wgpu::util::DeviceExt;

pub struct Target<'a> { pub view: &'a wgpu::TextureView, pub size: [f32; 2], pub clear: Option<wgpu::Color> }

pub struct Renderer {
    pub atlas_size: [u32; 2],
    pipelines: [wgpu::RenderPipeline; 3],
    bind_group: wgpu::BindGroup,
    globals: wgpu::Buffer,
    index_tex: wgpu::Texture,
    rgba_tex: wgpu::Texture,
    palette_tex: wgpu::Texture,
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    vcap: u64,
    icap: u64,
}

fn texture(device: &wgpu::Device, label: &str, w: u32, h: u32, format: wgpu::TextureFormat) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn blend_state(b: Blend) -> wgpu::BlendState {
    let c = |src, dst, operation| wgpu::BlendComponent { src_factor: src, dst_factor: dst, operation };
    use wgpu::{BlendFactor as F, BlendOperation as O};
    match b {
        Blend::Alpha => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        Blend::Add => wgpu::BlendState { color: c(F::One, F::One, O::Add), alpha: c(F::Zero, F::One, O::Add) },
        Blend::Sub => wgpu::BlendState { color: c(F::One, F::One, O::ReverseSubtract), alpha: c(F::Zero, F::One, O::Add) },
    }
}

fn write_region(queue: &wgpu::Queue, tex: &wgpu::Texture, r: Rect, bpp: u32, data: &[u8]) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d { x: r.x, y: r.y, z: 0 }, aspect: wgpu::TextureAspect::All },
        data,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(r.w * bpp), rows_per_image: Some(r.h) },
        wgpu::Extent3d { width: r.w, height: r.h, depth_or_array_layers: 1 },
    );
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, atlas_size: [u32; 2]) -> Self {
        let [aw, ah] = atlas_size;
        let index_tex = texture(device, "ume-index-atlas", aw, ah, wgpu::TextureFormat::R8Unorm);
        let rgba_tex = texture(device, "ume-rgba-atlas", aw, ah, wgpu::TextureFormat::Rgba8Unorm);
        let palette_tex = texture(device, "ume-palettes", 256, PALETTE_ROWS, wgpu::TextureFormat::Rgba8Unorm);
        let globals = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("ume-globals"), contents: &[0u8; 16], usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("ume-nearest"), ..Default::default() });
        let tex_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ume-sprite-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                tex_entry(1),
                tex_entry(2),
                tex_entry(3),
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let view = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let (iv, rv, pv) = (view(&index_tex), view(&rgba_tex), view(&palette_tex));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ume-sprite-bind"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&iv) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&rv) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&pv) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("ume-sprite"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("ume-sprite-pl"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let attrs = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32, 3 => Float32x4, 4 => Float32x4];
        let pipeline = |b: Blend| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ume-sprite-pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<Vertex>() as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &attrs })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(blend_state(b)), write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let pipelines = [pipeline(Blend::Alpha), pipeline(Blend::Add), pipeline(Blend::Sub)];
        let (vcap, icap) = (4096u64, 6144u64);
        let vbuf = Self::buffer(device, vcap * std::mem::size_of::<Vertex>() as u64, wgpu::BufferUsages::VERTEX);
        let ibuf = Self::buffer(device, icap * 4, wgpu::BufferUsages::INDEX);
        Renderer { atlas_size, pipelines, bind_group, globals, index_tex, rgba_tex, palette_tex, vbuf, ibuf, vcap, icap }
    }

    fn buffer(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor { label: Some("ume-batch"), size, usage: usage | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
    }

    pub fn upload_indexed(&self, queue: &wgpu::Queue, r: Rect, pixels: &[u8]) { write_region(queue, &self.index_tex, r, 1, pixels); }

    pub fn upload_rgba(&self, queue: &wgpu::Queue, r: Rect, pixels: &[u8]) { write_region(queue, &self.rgba_tex, r, 4, pixels); }

    pub fn set_palette(&self, queue: &wgpu::Queue, row: u32, colors: &[[u8; 4]; 256]) {
        write_region(queue, &self.palette_tex, Rect { x: 0, y: row, w: 256, h: 1 }, 4, bytemuck::cast_slice(colors));
    }

    pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: Target, batch: &Batch) {
        let Target { view: target, size: screen, clear } = target;
        if batch.vertices.len() as u64 > self.vcap {
            self.vcap = (batch.vertices.len() as u64).next_power_of_two();
            self.vbuf = Self::buffer(device, self.vcap * std::mem::size_of::<Vertex>() as u64, wgpu::BufferUsages::VERTEX);
        }
        if batch.indices.len() as u64 > self.icap {
            self.icap = (batch.indices.len() as u64).next_power_of_two();
            self.ibuf = Self::buffer(device, self.icap * 4, wgpu::BufferUsages::INDEX);
        }
        queue.write_buffer(&self.globals, 0, bytemuck::cast_slice(&[screen[0], screen[1], 0.0, 0.0]));
        if !batch.vertices.is_empty() {
            queue.write_buffer(&self.vbuf, 0, bytemuck::cast_slice(&batch.vertices));
            queue.write_buffer(&self.ibuf, 0, bytemuck::cast_slice(&batch.indices));
        }
        let load = clear.map(wgpu::LoadOp::Clear).unwrap_or(wgpu::LoadOp::Load);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ume-sprites"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: target, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load, store: wgpu::StoreOp::Store } })],
            ..Default::default()
        });
        if batch.calls.is_empty() { return; }
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vbuf.slice(..));
        pass.set_index_buffer(self.ibuf.slice(..), wgpu::IndexFormat::Uint32);
        let (tw, th) = (screen[0].max(1.0) as u32, screen[1].max(1.0) as u32);
        for c in &batch.calls {
            let [x, y, w, h] = c.scissor.map(|[x, y, w, h]| { let (x, y) = (x.min(tw), y.min(th)); [x, y, w.min(tw - x), h.min(th - y)] }).unwrap_or([0, 0, tw, th]);
            if w == 0 || h == 0 { continue; }
            pass.set_scissor_rect(x, y, w, h);
            pass.set_pipeline(&self.pipelines[c.blend as usize]);
            pass.draw_indexed(c.first_index..c.first_index + c.index_count, 0, 0..1);
        }
    }
}
