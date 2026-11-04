pub fn target(device: &wgpu::Device, like: &wgpu::Texture) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ume-screenshot-target"),
        size: like.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: like.format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

pub struct Readback {
    buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    row: u32,
    bgra: bool,
}

pub fn copy(
    device: &wgpu::Device,
    enc: &mut wgpu::CommandEncoder,
    tex: &wgpu::Texture,
) -> Option<Readback> {
    if !tex.usage().contains(wgpu::TextureUsages::COPY_SRC) {
        return None;
    }
    let (width, height) = (tex.width(), tex.height());
    let row = (width * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ume-screenshot"),
        size: (row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let bgra = matches!(
        tex.format(),
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    );
    Some(Readback {
        buffer,
        width,
        height,
        row,
        bgra,
    })
}

pub trait Save {
    fn save(self, device: &wgpu::Device, path: &str) -> Result<(), String>;
}

impl Save for Option<Readback> {
    fn save(self, device: &wgpu::Device, path: &str) -> Result<(), String> {
        let r = self.ok_or("surface does not support copies")?;
        r.buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        let data = r
            .buffer
            .slice(..)
            .get_mapped_range()
            .map_err(|e| e.to_string())?;
        let mut rgba = Vec::with_capacity((r.width * r.height * 4) as usize);
        for y in 0..r.height {
            for px in data[(y * r.row) as usize..][..(r.width * 4) as usize]
                .as_chunks::<4>()
                .0
            {
                if r.bgra {
                    rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
                } else {
                    rgba.extend_from_slice(&[px[0], px[1], px[2], 255]);
                }
            }
        }
        let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), r.width, r.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()
            .and_then(|mut w| w.write_image_data(&rgba))
            .map_err(|e| e.to_string())
    }
}
