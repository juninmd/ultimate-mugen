use ume_model3d::model::Posed;
use ume_model3d::{Camera, Facing, Model, Raster};

const CELL: usize = 240;
const BODY: f32 = 170.0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = Model::from_glb(&std::fs::read(&args[1]).unwrap()).unwrap();
    let out_path = &args[2];
    let frames: usize = args.get(4).and_then(|v| v.parse().ok()).unwrap_or(6);
    let yaw: f32 = args.get(5).and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let names: Vec<&str> = args[3].split(',').collect();
    let mut camera = Camera::fit_height(&model, BODY);
    camera.yaw_offset = yaw;
    let (w, h) = (CELL * frames, CELL * names.len());
    let mut sheet = vec![30u8; w * h * 4];
    let mut posed = Posed::default();
    let mut triangles = Vec::new();
    let mut raster = Raster::new(CELL, CELL, 1);
    let mut rgba = vec![0u8; CELL * CELL * 4];
    for (row, name) in names.iter().enumerate() {
        let clip = model.clip_index(name);
        let duration = clip.map(|c| model.clips[c].duration).unwrap_or(0.0);
        for col in 0..frames {
            let t = duration * col as f32 / (frames.max(2) - 1) as f32;
            model.pose(clip, t, &mut posed);
            let origin = [CELL as f32 / 2.0, CELL as f32 * 0.9];
            model.project(&posed, &camera, Facing::Right, origin, &mut triangles);
            raster.draw(&triangles, &model.images);
            raster.resolve(&mut rgba);
            for y in 0..CELL {
                for x in 0..CELL {
                    let s = (y * CELL + x) * 4;
                    if rgba[s + 3] == 0 {
                        continue;
                    }
                    let d = ((row * CELL + y) * w + col * CELL + x) * 4;
                    sheet[d..d + 4].copy_from_slice(&rgba[s..s + 4]);
                }
            }
        }
    }
    let file = std::fs::File::create(out_path).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().unwrap().write_image_data(&sheet).unwrap();
}
