use ume_render::{Batch, Blend, Quad, Rect, Renderer, Target, RGBA_SPRITE};

fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { force_fallback_adapter: false, ..Default::default() })).ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

fn render(device: &wgpu::Device, queue: &wgpu::Queue, r: &mut Renderer, batch: &Batch, clear: wgpu::Color) -> Vec<u8> {
    let (w, h) = (64u32, 64u32);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let out = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (w * h * 4) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = device.create_command_encoder(&Default::default());
    r.draw(device, queue, &mut enc, Target { view: &view, size: [w as f32, h as f32], clear: Some(clear) }, batch);
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &out, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    out.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = out.slice(..).get_mapped_range().unwrap().to_vec();
    data
}

fn px(img: &[u8], x: u32, y: u32) -> [u8; 4] { let i = ((y * 64 + x) * 4) as usize; [img[i], img[i + 1], img[i + 2], img[i + 3]] }

#[test]
fn draws_palettes_rgba_and_blending() {
    let Some((device, queue)) = gpu() else { eprintln!("no GPU adapter, skipping"); return; };
    let mut r = Renderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm, [256, 256]);
    let mut pal = [[0u8; 4]; 256];
    pal[1] = [255, 0, 0, 255];
    pal[2] = [0, 0, 255, 255];
    r.set_palette(&queue, 0, &pal);
    pal[1] = [0, 255, 0, 255];
    r.set_palette(&queue, 5, &pal);
    let idx = Rect { x: 1, y: 1, w: 2, h: 2 };
    r.upload_indexed(&queue, idx, &[1, 2, 0, 1]);
    let rgba = Rect { x: 8, y: 8, w: 1, h: 1 };
    r.upload_rgba(&queue, rgba, &[200, 200, 200, 255]);

    let mut b = Batch::new();
    b.push(&Quad::new([0.0, 0.0, 16.0, 16.0], idx, [256, 256], 0));
    b.push(&Quad::new([16.0, 0.0, 16.0, 16.0], idx, [256, 256], 5));
    let mut flipped = Quad::new([32.0, 0.0, 16.0, 16.0], idx, [256, 256], 0);
    flipped.flip_h = true;
    b.push(&flipped);
    b.push(&Quad::new([0.0, 32.0, 16.0, 16.0], rgba, [256, 256], RGBA_SPRITE));
    let mut add = Quad::new([0.0, 32.0, 16.0, 16.0], rgba, [256, 256], RGBA_SPRITE);
    add.blend = Blend::Add;
    add.mul = [0.1, 0.1, 0.1, 1.0];
    b.push(&add);
    let mut fx = Quad::new([32.0, 32.0, 16.0, 16.0], rgba, [256, 256], RGBA_SPRITE);
    fx.mul = [0.0, 0.0, 0.0, 1.0];
    fx.add = [0.0, 1.0, 0.0];
    b.push(&fx);

    let img = render(&device, &queue, &mut r, &b, wgpu::Color::BLACK);
    assert_eq!(px(&img, 4, 4), [255, 0, 0, 255]);
    assert_eq!(px(&img, 12, 4), [0, 0, 255, 255]);
    assert_eq!(px(&img, 4, 12), [0, 0, 0, 255]);
    assert_eq!(px(&img, 20, 4), [0, 255, 0, 255]);
    assert_eq!(px(&img, 36, 4), [0, 0, 255, 255]);
    let lit = px(&img, 4, 36);
    assert!((218..=222).contains(&lit[0]), "{lit:?}");
    assert_eq!(px(&img, 36, 36), [0, 255, 0, 255]);
    assert_eq!(px(&img, 60, 60), [0, 0, 0, 255]);
}

#[test]
fn canvas_shapes_render() {
    let Some((device, queue)) = gpu() else { eprintln!("no GPU adapter, skipping"); return; };
    let mut r = Renderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm, [64, 64]);
    let mut atlas = ume_render::Atlas::new(64, 64);
    let mut c = ume_render::Canvas::new(&r, &queue, &mut atlas).unwrap();
    c.rect(2.0, 2.0, 10.0, 10.0, ume_render::Color::rgba(255, 0, 0, 255));
    c.circle([40.0, 10.0], 6.0, ume_render::Color::rgba(0, 255, 0, 255));
    c.line([0.0, 40.0], [64.0, 40.0], 4.0, ume_render::Color::rgba(0, 0, 255, 255));
    c.triangle([10.0, 50.0], [30.0, 50.0], [10.0, 63.0], ume_render::Color::rgba(255, 255, 255, 128));
    let img = render(&device, &queue, &mut r, &c.batch, wgpu::Color::BLACK);
    assert_eq!(px(&img, 6, 6), [255, 0, 0, 255]);
    assert_eq!(px(&img, 40, 10), [0, 255, 0, 255]);
    assert_eq!(px(&img, 46, 2), [0, 0, 0, 255]);
    assert_eq!(px(&img, 20, 40), [0, 0, 255, 255]);
    assert_eq!(px(&img, 20, 45), [0, 0, 0, 255]);
    let half = px(&img, 12, 52);
    assert!((126..=130).contains(&half[0]), "{half:?}");
}

fn post_render(device: &wgpu::Device, queue: &wgpu::Queue, p: &ume_render::PostParams) -> Vec<u8> {
    let (w, h) = (64u32, 64u32);
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let mut r = Renderer::new(device, queue, fmt, [64, 64]);
    let mut atlas = ume_render::Atlas::new(64, 64);
    let mut canvas = ume_render::Canvas::new(&r, queue, &mut atlas).unwrap();
    canvas.rect(0.0, 0.0, 32.0, 64.0, ume_render::Color::rgba(255, 0, 0, 255));
    let mut post = ume_render::PostFx::new(device, fmt);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: fmt,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let out = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (w * h * 4) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = device.create_command_encoder(&Default::default());
    let scene = post.scene_view(device, [w, h]).clone();
    r.draw(device, queue, &mut enc, Target { view: &scene, size: [w as f32, h as f32], clear: Some(wgpu::Color::BLACK) }, &canvas.batch);
    post.apply(queue, &mut enc, &view, p);
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &out, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    out.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = out.slice(..).get_mapped_range().unwrap().to_vec();
    data
}

#[test]
fn post_effects_composite() {
    let Some((device, queue)) = gpu() else { eprintln!("no GPU adapter, skipping"); return; };
    let near = |a: [u8; 4], b: [u8; 3]| (0..3).all(|i| (a[i] as i32 - b[i] as i32).abs() <= 3);
    let plain = post_render(&device, &queue, &ume_render::PostParams::default());
    assert!(near(px(&plain, 8, 32), [255, 0, 0]), "{:?}", px(&plain, 8, 32));
    assert!(near(px(&plain, 56, 32), [0, 0, 0]), "{:?}", px(&plain, 56, 32));
    let flash = post_render(&device, &queue, &ume_render::PostParams { flash: 0.5, ..Default::default() });
    assert!(near(px(&flash, 56, 32), [128, 128, 128]), "{:?}", px(&flash, 56, 32));
    let grey = post_render(&device, &queue, &ume_render::PostParams { saturation: 0.0, ..Default::default() });
    let g = px(&grey, 8, 32);
    assert!(g[0] == g[1] && g[1] == g[2] && (74..=80).contains(&g[0]), "{g:?}");
    let glow = post_render(&device, &queue, &ume_render::PostParams { bloom: 1.0, ..Default::default() });
    assert!(px(&glow, 36, 32)[0] > px(&plain, 36, 32)[0], "bloom should spill past the edge");
}

#[test]
fn palfx_in_shader() {
    let Some((device, queue)) = gpu() else { eprintln!("no GPU adapter, skipping"); return; };
    let mut r = Renderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm, [16, 16]);
    let rgba = Rect { x: 4, y: 4, w: 1, h: 1 };
    r.upload_rgba(&queue, rgba, &[200, 100, 0, 255]);
    let uv = [4.5 / 16.0, 4.5 / 16.0];
    let mut b = Batch::new();
    let fxs = [
        ume_render::PalFx { invert: true, ..Default::default() },
        ume_render::PalFx { color: 0.0, ..Default::default() },
        ume_render::PalFx { add: [0.2, 0.2, 0.2], mul: [0.5, 0.5, 2.0], ..Default::default() },
        ume_render::PalFx { hue: 120.0, ..Default::default() },
    ];
    for (k, fx) in fxs.iter().enumerate() {
        let (mul, add, hue) = fx.vertex([1.0; 4]);
        let x = k as f32 * 16.0;
        let v = |pos: [f32; 2]| ume_render::Vertex::new(pos, uv, RGBA_SPRITE, mul, add, hue);
        b.push_tris(&[v([x, 0.0]), v([x + 16.0, 0.0]), v([x + 16.0, 16.0]), v([x, 16.0])], &[0, 1, 2, 0, 2, 3], Blend::Alpha);
    }
    let img = render(&device, &queue, &mut r, &b, wgpu::Color::BLACK);
    let near = |a: [u8; 4], e: [u8; 3]| (0..3).all(|i| (a[i] as i32 - e[i] as i32).abs() <= 2);
    assert!(near(px(&img, 8, 8), [55, 155, 255]), "{:?}", px(&img, 8, 8));
    assert!(near(px(&img, 24, 8), [100, 100, 100]), "{:?}", px(&img, 24, 8));
    assert!(near(px(&img, 40, 8), [126, 76, 102]), "{:?}", px(&img, 40, 8));
    assert!(near(px(&img, 56, 8), [0, 200, 100]), "hue 120 rotates the channels: {:?}", px(&img, 56, 8));
}

#[test]
fn glass_panel_blurs_and_leaves_the_rest_alone() {
    let Some((device, queue)) = gpu() else { eprintln!("no GPU adapter, skipping"); return; };
    let plain = post_render(&device, &queue, &ume_render::PostParams::default());
    let mut p = ume_render::PostParams::default();
    assert!(p.glass.push(20.0, 16.0, 24.0, 32.0, 4.0, [0.5, 0.6, 0.9]));
    let glass = post_render(&device, &queue, &p);
    assert_eq!(px(&glass, 2, 2), px(&plain, 2, 2), "far outside the panel nothing changes");
    assert!(px(&glass, 38, 32)[0] > px(&plain, 38, 32)[0] + 20, "red bleeds across the sharp edge inside the panel: {:?}", px(&glass, 38, 32));
    assert!(px(&glass, 26, 32)[0] < px(&plain, 26, 32)[0], "and the sharp red is softened: {:?}", px(&glass, 26, 32));
    assert!(px(&glass, 18, 32)[0] < px(&plain, 18, 32)[0], "the panel casts a soft shadow just outside its edge");
    assert!(!{ let mut g = ume_render::Glass::default(); (0..ume_render::MAX_GLASS + 1).all(|_| g.push(0.0, 0.0, 4.0, 4.0, 1.0, [0.0; 3])) }, "capacity is bounded");
}

#[test]
fn shockwave_bends_the_edge_and_tonemap_compresses_highlights() {
    let Some((device, queue)) = gpu() else { eprintln!("no GPU adapter, skipping"); return; };
    let plain = post_render(&device, &queue, &ume_render::PostParams::default());
    let mut p = ume_render::PostParams::default();
    assert!(p.waves.push(46.0, 32.0, 14.0, 10.0, 1.0));
    let wave = post_render(&device, &queue, &p);
    assert_eq!(px(&wave, 2, 2), px(&plain, 2, 2), "far from the ring nothing moves");
    assert_ne!(px(&wave, 33, 32), px(&plain, 33, 32), "pixels on the ring are displaced");
    let toned = post_render(&device, &queue, &ume_render::PostParams { grade: ume_render::Grade { tonemap: 1.0, ..Default::default() }, ..Default::default() });
    assert!(px(&toned, 8, 32)[0] < 255, "ACES rolls the clipped red off: {:?}", px(&toned, 8, 32));
    assert!(px(&toned, 56, 32)[0] <= 2, "black stays black");
    let tinted = post_render(&device, &queue, &ume_render::PostParams { grade: ume_render::Grade { split: 1.0, highlights: [0.5, 1.0, 1.0], ..Default::default() }, ..Default::default() });
    assert!(px(&tinted, 8, 32)[0] < px(&plain, 8, 32)[0], "highlight tint pulls red down");
}
