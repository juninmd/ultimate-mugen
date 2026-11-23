use std::time::Instant;
use ume_model3d::model::Posed;
use ume_model3d::{Camera, Facing, Model, Raster, StateMap};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = Model::from_glb(&std::fs::read(&args[1]).unwrap()).unwrap();
    let map = StateMap::from_json(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let samples: usize = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(2);
    let frames = 300;
    let mut camera = Camera::fit_height(&model, 106.0);
    camera.pixels_per_unit *= 2.1;
    let (mut posed, mut triangles) = (Posed::default(), Vec::new());
    let mut raster = Raster::new(448, 448, samples);
    let mut rgba = vec![0u8; 448 * 448 * 4];
    let mut spent = [0.0f64; 3];
    for tick in 0..frames {
        let start = Instant::now();
        model.pose_for_state(&map, 0, tick, &mut posed);
        model.project(
            &posed,
            &camera,
            Facing::Right,
            [224.0, 376.0],
            &mut triangles,
        );
        let posed_at = Instant::now();
        raster.draw(&triangles, &model.images);
        let drawn_at = Instant::now();
        raster.resolve(&mut rgba);
        spent[0] += (posed_at - start).as_secs_f64();
        spent[1] += (drawn_at - posed_at).as_secs_f64();
        spent[2] += drawn_at.elapsed().as_secs_f64();
    }
    let ms = spent.map(|s| s * 1000.0 / frames as f64);
    let area: f32 = triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.points;
            ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() / 2.0
        })
        .sum();
    let rows: f32 = triangles
        .iter()
        .map(|t| {
            let ys = t.points.map(|q| q[1]);
            ys[0].max(ys[1]).max(ys[2]).ceil() - ys[0].min(ys[1]).min(ys[2]).floor()
        })
        .sum();
    let spread: f32 = triangles
        .iter()
        .map(|t| {
            t.points
                .iter()
                .map(|q| q[0].abs() + q[1].abs())
                .fold(0.0, f32::max)
        })
        .fold(0.0, f32::max);
    println!("fragment area {area:.0} px at 1x, rows {rows:.0}, max coord {spread:.0}");
    println!(
        "triangles {} samples {samples}: pose+project {:.2} ms, raster {:.2} ms, resolve {:.2} ms per frame",
        triangles.len(),
        ms[0],
        ms[1],
        ms[2]
    );
}
