use ume_model3d::model::Posed;
use ume_model3d::{Camera, Facing, Model, Raster, StateMap};

const BODY_HEIGHT: f32 = 1600.0;
const OUT_EDGE: usize = 320;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = Model::from_glb(&std::fs::read(&args[1]).unwrap()).unwrap();
    let map = StateMap::from_json(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let tick: i32 = args.get(4).and_then(|v| v.parse().ok()).unwrap_or(-1);
    let yaw: f32 = args.get(5).and_then(|v| v.parse().ok()).unwrap_or(-1.3);
    let share: f32 = args.get(6).and_then(|v| v.parse().ok()).unwrap_or(0.19);
    let mut camera = Camera::fit_height(&model, BODY_HEIGHT);
    camera.yaw_offset = yaw;
    let mut posed = Posed::default();
    if tick < 0 {
        model.pose(None, 0.0, &mut posed);
    } else {
        model.pose_for_state(&map, 0, tick, &mut posed);
    }
    let size = (BODY_HEIGHT * 1.3) as usize;
    let mut triangles = Vec::new();
    let origin = [size as f32 / 2.0, size as f32 * 0.95];
    model.project(&posed, &camera, Facing::Right, origin, &mut triangles);
    let mut raster = Raster::new(size, size, 2);
    raster.draw(&triangles, &model.images);
    let mut rgba = vec![0u8; size * size * 4];
    raster.resolve(&mut rgba);
    let alpha = |x: usize, y: usize| rgba[(y * size + x) * 4 + 3];
    let top = (0..size)
        .find(|&y| (0..size).any(|x| alpha(x, y) > 0))
        .unwrap_or(0);
    let side = (BODY_HEIGHT * share) as usize;
    let cols: Vec<usize> = (0..size)
        .filter(|&x| (top..(top + side / 2).min(size)).any(|y| alpha(x, y) > 0))
        .collect();
    let center = (cols.first().unwrap_or(&0) + cols.last().unwrap_or(&size)) / 2;
    let (left, top) = (
        center.saturating_sub(side / 2),
        top.saturating_sub(side / 14),
    );
    let k = side as f32 / OUT_EDGE as f32;
    let mut out = Vec::with_capacity(OUT_EDGE * OUT_EDGE * 4);
    for oy in 0..OUT_EDGE {
        for ox in 0..OUT_EDGE {
            let (x0, y0) = (
                left + (ox as f32 * k) as usize,
                top + (oy as f32 * k) as usize,
            );
            let (x1, y1) = (
                left + ((ox + 1) as f32 * k) as usize,
                top + ((oy + 1) as f32 * k) as usize,
            );
            let mut acc = [0.0f32; 4];
            let mut n = 0.0;
            for y in y0..y1.max(y0 + 1) {
                for x in x0..x1.max(x0 + 1) {
                    n += 1.0;
                    if x < size && y < size {
                        let p = &rgba[(y * size + x) * 4..(y * size + x) * 4 + 4];
                        let a = p[3] as f32;
                        for c in 0..3 {
                            acc[c] += p[c] as f32 * a;
                        }
                        acc[3] += a;
                    }
                }
            }
            let color = |c: usize| {
                if acc[3] > 0.0 {
                    (acc[c] / acc[3]) as u8
                } else {
                    0
                }
            };
            out.extend_from_slice(&[color(0), color(1), color(2), (acc[3] / n) as u8]);
        }
    }
    let file = std::fs::File::create(&args[3]).unwrap();
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(file),
        OUT_EDGE as u32,
        OUT_EDGE as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&out)
        .unwrap();
}
