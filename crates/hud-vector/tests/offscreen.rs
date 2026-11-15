use ume_hud_vector::{BarSpec, Tween, VectorHud};
use vello::kurbo::Rect;
use vello::peniko::Color;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .ok()
}

#[test]
fn tween_settles_to_target() {
    let mut t = Tween::settled(1.0);
    t.retarget(0.25, 0.5);
    t.advance(0.25);
    assert!(t.value() < 1.0 && t.value() > 0.25);
    t.advance(0.5);
    assert!(t.finished());
    assert!((t.value() - 0.25).abs() < 1e-6);
}

#[test]
fn renders_bar_pixels_offscreen() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let (w, h) = (128u32, 16u32);
    let mut hud = VectorHud::new(&device).expect("vello renderer");
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let spec = BarSpec {
        rect: Rect::new(0.0, 0.0, 128.0, 16.0),
        fraction: 0.5,
        trail: 0.75,
        fill: Color::from_rgb8(0, 255, 0),
        trail_fill: Color::from_rgb8(255, 255, 0),
        track: Color::from_rgb8(40, 40, 40),
        radius: 0.0,
    };
    hud.render(&device, &queue, &view, w, h, &[spec]).expect("render");

    let bytes_per_row = 512u32;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bytes_per_row * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bytes_per_row), rows_per_image: None },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = slice.get_mapped_range().unwrap();
    let px = |x: usize| {
        let i = 8 * bytes_per_row as usize + x * 4;
        [data[i], data[i + 1], data[i + 2], data[i + 3]]
    };
    assert_eq!(px(20), [0, 255, 0, 255], "filled part");
    assert_eq!(px(80), [255, 255, 0, 255], "trail part");
    assert_eq!(px(120), [40, 40, 40, 255], "track part");
}
