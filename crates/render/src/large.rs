use std::collections::HashMap;

pub struct Large {
    layout: wgpu::BindGroupLayout,
    globals: wgpu::Buffer,
    samplers: [wgpu::Sampler; 2],
    index: wgpu::TextureView,
    palette: wgpu::TextureView,
    slots: HashMap<u32, Slot>,
    next: u32,
}

struct Slot {
    tex: wgpu::Texture,
    groups: [wgpu::BindGroup; 2],
}

impl Large {
    pub fn new(layout: wgpu::BindGroupLayout, globals: wgpu::Buffer, samplers: [wgpu::Sampler; 2], index: wgpu::TextureView, palette: wgpu::TextureView) -> Self {
        Large { layout, globals, samplers, index, palette, slots: HashMap::new(), next: 1 }
    }

    pub fn alloc(&mut self, device: &wgpu::Device, w: u32, h: u32) -> u32 {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ume-large-texture"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let group = |sampler: &wgpu::Sampler| device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ume-large-bind"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.globals.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&self.index) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&self.palette) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(sampler) },
            ],
        });
        let groups = [group(&self.samplers[0]), group(&self.samplers[1])];
        let id = self.next;
        self.next += 1;
        self.slots.insert(id, Slot { tex, groups });
        id
    }

    pub fn write(&self, queue: &wgpu::Queue, id: u32, w: u32, h: u32, pixels: &[u8]) {
        let Some(slot) = self.slots.get(&id) else { return };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &slot.tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            pixels,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
    }

    pub fn group(&self, id: u32, linear: bool) -> Option<&wgpu::BindGroup> {
        self.slots.get(&id).map(|s| &s.groups[linear as usize])
    }

    pub fn free(&mut self, id: u32) {
        self.slots.remove(&id);
    }
}
