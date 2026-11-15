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

#[test]
fn dumps_demo_frame_when_asked() {
    let Ok(path) = std::env::var("UME_HUD_DUMP") else { return };
    let Some((device, queue)) = device() else { return };
    let (w, h) = (640u32, 200u32);
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
    let mut tween = Tween::settled(1.0);
    tween.retarget(0.42, 0.6);
    tween.advance(0.2);
    let trail = tween.value() as f32;
    let bar = |y: f64, fraction: f32, trail: f32| BarSpec {
        rect: Rect::new(20.0, y, 620.0, y + 28.0),
        fraction,
        trail,
        fill: Color::from_rgb8(230, 60, 60),
        trail_fill: Color::from_rgb8(255, 210, 80),
        track: Color::from_rgb8(30, 30, 40),
        radius: 6.0,
    };
    hud.render(&device, &queue, &view, w, h, &[bar(20.0, 0.42, trail), bar(120.0, 0.7, 0.9)]).expect("render");
    let bytes_per_row = 2560u32;
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
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for row in 0..h as usize {
        let start = row * bytes_per_row as usize;
        out.extend_from_slice(&data[start..start + (w * 4) as usize]);
    }
    std::fs::write(path, out).unwrap();
}

#[test]
fn dumps_damage_sequence_and_timing_when_asked() {
    let Ok(dir) = std::env::var("UME_HUD_SEQ") else { return };
    let Some((device, queue)) = device() else { return };
    let (w, h) = (640u32, 120u32);
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
    let bytes_per_row = 2560u32;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bytes_per_row * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut trail = Tween::settled(1.0);
    let mut life = 1.0f32;
    let mut build_us = Vec::new();
    let mut total_us = Vec::new();
    std::fs::create_dir_all(&dir).unwrap();
    for frame in 0..60u32 {
        if frame == 10 {
            life = 0.42;
            trail.retarget(life, 0.6);
        }
        if frame == 30 {
            life = 0.18;
            trail.retarget(life, 0.6);
        }
        trail.advance(1.0 / 60.0);
        let spec = |y: f64, fraction: f32, trail: f32| BarSpec {
            rect: Rect::new(20.0, y, 620.0, y + 28.0),
            fraction,
            trail,
            fill: Color::from_rgb8(230, 60, 60),
            trail_fill: Color::from_rgb8(255, 210, 80),
            track: Color::from_rgb8(30, 30, 40),
            radius: 6.0,
        };
        let bars = [spec(20.0, life, trail.value()), spec(70.0, 0.7, 0.7)];
        let t0 = std::time::Instant::now();
        hud.build(&bars);
        let built = t0.elapsed();
        hud.render(&device, &queue, &view, w, h, &bars).expect("render");
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
        total_us.push(t0.elapsed().as_micros());
        build_us.push(built.as_micros());
        let data = slice.get_mapped_range().unwrap();
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for row in 0..h as usize {
            let start = row * bytes_per_row as usize;
            out.extend_from_slice(&data[start..start + (w * 4) as usize]);
        }
        drop(data);
        buf.unmap();
        std::fs::write(format!("{dir}/f{frame:03}.raw"), out).unwrap();
    }
    build_us.sort();
    total_us.sort();
    let med = |v: &Vec<u128>| v[v.len() / 2];
    std::fs::write(
        format!("{dir}/timing.json"),
        format!("{{\"build_us_p50\":{},\"total_us_p50\":{},\"frames\":60}}", med(&build_us), med(&total_us)),
    )
    .unwrap();
}
