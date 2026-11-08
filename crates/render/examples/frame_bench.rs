use std::time::Instant;
use ume_render::{Batch, Blend, Quad, Rect, Renderer, Target};

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        force_fallback_adapter: false,
        ..Default::default()
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

fn scene(per_quad: [u32; 2], quads: usize) -> Batch {
    let mut b = Batch::new();
    let mut palette = [[0u8; 4]; 256];
    for (i, c) in palette.iter_mut().enumerate() {
        *c = [i as u8, (255 - i) as u8, (i * 3) as u8, 255];
    }
    let sprite = Rect {
        x: 0,
        y: 0,
        w: per_quad[0],
        h: per_quad[1],
    };
    for i in 0..quads {
        let mut q = Quad::new(
            [(i % 80) as f32 * 16.0, (i / 80) as f32 * 24.0, 64.0, 96.0],
            sprite,
            [2048, 2048],
            (i % 8) as u32,
        );
        q.mul = [0.9, 0.9, 0.9, 1.0];
        if i % 7 == 0 {
            q.blend = Blend::Add;
            q.add = [0.2, 0.1, 0.0];
        }
        b.push(&q);
    }
    b
}

fn main() {
    let Some((device, queue)) = device() else {
        println!("no GPU adapter, skipping");
        return;
    };
    let (w, h) = (1280u32, 720u32);
    let mut renderer = Renderer::new(
        &device,
        &queue,
        wgpu::TextureFormat::Rgba8Unorm,
        [2048, 2048],
    );
    let mut palette = [[0u8; 4]; 256];
    for (i, c) in palette.iter_mut().enumerate() {
        *c = [i as u8, (255 - i) as u8, (i * 3) as u8, 255];
    }
    renderer.set_palette(&queue, 0, &palette);
    renderer.upload_indexed(
        &queue,
        Rect {
            x: 0,
            y: 0,
            w: 64,
            h: 96,
        },
        &vec![7u8; 64 * 96],
    );

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());

    for quads in [500usize, 2000, 5000] {
        let batch = scene([64, 96], quads);
        let frames = 120;
        let start = Instant::now();
        for _ in 0..frames {
            let mut enc = device.create_command_encoder(&Default::default());
            renderer.draw(
                &device,
                &queue,
                &mut enc,
                Target {
                    view: &view,
                    size: [w as f32, h as f32],
                    clear: Some(wgpu::Color::BLACK),
                },
                &batch,
            );
            queue.submit([enc.finish()]);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        }
        let per_frame = start.elapsed().as_secs_f64() * 1e3 / frames as f64;
        let index_bytes = batch.indices.len() * if batch.vertices.len() <= 65536 { 2 } else { 4 };
        println!(
            "quads={quads:5} calls={:3} bytes={:8} frame={per_frame:6.3} ms headroom={:5.1}% of 16.667",
            batch.calls.len(),
            batch.vertices.len() * std::mem::size_of::<ume_render::Vertex>() + index_bytes,
            (per_frame / 16.667) * 100.0
        );
    }
}
