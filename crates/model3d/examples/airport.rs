use std::time::Instant;
use ume_model3d::model::Posed;
use ume_model3d::{Lens, Look, Model, Raster};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = Model::from_glb(&std::fs::read(&args[1]).unwrap()).unwrap();
    let (w, h): (usize, usize) = (
        args.get(3).and_then(|v| v.parse().ok()).unwrap_or(960),
        args.get(4).and_then(|v| v.parse().ok()).unwrap_or(540),
    );
    let samples: usize = args.get(5).and_then(|v| v.parse().ok()).unwrap_or(2);
    let time: f32 = args.get(6).and_then(|v| v.parse().ok()).unwrap_or(9.0);
    let lens = Lens {
        eye: [0.0, 12.0, 105.0],
        target: [0.0, 15.0, -60.0],
        fov_y: 0.66,
        width: w as f32,
        height: h as f32,
    };
    let look = Look {
        toward_light: [-0.5, 0.7, 0.5],
        light: [0.62, 0.52, 0.38],
        ambient: [0.55, 0.58, 0.72],
        fog: [0.99, 0.74, 0.60],
        fog_density: 0.0016,
        sky: [[0.20, 0.24, 0.52], [0.99, 0.72, 0.52]],
    };
    let clip = model.clip_index("ambient");
    let (mut posed, mut tris) = (Posed::default(), Vec::new());
    let mut raster = Raster::new(w, h, samples);
    let mut rgba = vec![0u8; w * h * 4];
    let frames = 120;
    let mut spent = [0.0f64; 3];
    for i in 0..frames {
        let t0 = Instant::now();
        model.pose(clip, time + i as f32 / 60.0, &mut posed);
        model.project_scene(&posed, &lens, &look, &mut tris);
        let t1 = Instant::now();
        raster.draw(&tris, &model.images);
        let t2 = Instant::now();
        raster.resolve(&mut rgba);
        spent[0] += (t1 - t0).as_secs_f64();
        spent[1] += (t2 - t1).as_secs_f64();
        spent[2] += t2.elapsed().as_secs_f64();
    }
    let ms = spent.map(|s| s * 1000.0 / frames as f64);
    println!(
        "{w}x{h} x{samples}: {} triangles, pose+project {:.2} ms, draw {:.2} ms, resolve {:.2} ms",
        tris.len(),
        ms[0],
        ms[1],
        ms[2]
    );
    let file = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    let mut enc = png::Encoder::new(file, w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&rgba).unwrap();
}
