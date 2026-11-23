use ume_model3d::model::Posed;
use ume_model3d::{Lens, Look, Model, Raster};

const AIRPORT: &[u8] = include_bytes!("../../../assets/model3d/airport.glb");

fn lens(eye: [f32; 3]) -> Lens {
    Lens {
        eye,
        target: [0.0, 15.0, -60.0],
        fov_y: 0.66,
        width: 160.0,
        height: 90.0,
    }
}

const LOOK: Look = Look {
    toward_light: [-0.5, 0.7, 0.5],
    light: [0.6, 0.5, 0.4],
    ambient: [0.5, 0.5, 0.7],
    fog: [1.0, 0.7, 0.6],
    fog_density: 0.0016,
    sky: [[0.2, 0.2, 0.5], [1.0, 0.7, 0.5]],
};

fn render(model: &Model, time: f32, eye: [f32; 3]) -> Vec<u8> {
    let (mut posed, mut tris) = (Posed::default(), Vec::new());
    model.pose(model.clip_index("ambient"), time, &mut posed);
    model.project_scene(&posed, &lens(eye), &LOOK, &mut tris);
    let mut raster = Raster::new(160, 90, 1);
    raster.draw(&tris, &model.images);
    let mut rgba = vec![0u8; 160 * 90 * 4];
    raster.resolve(&mut rgba);
    rgba
}

#[test]
fn airport_scene_fills_every_pixel() {
    let model = Model::from_glb(AIRPORT).unwrap();
    let rgba = render(&model, 7.5, [0.0, 9.0, 70.0]);
    assert!(rgba.chunks_exact(4).all(|p| p[3] == 255));
}

#[test]
fn ambient_clip_moves_the_planes() {
    let model = Model::from_glb(AIRPORT).unwrap();
    let a = render(&model, 3.0, [0.0, 9.0, 70.0]);
    let b = render(&model, 9.0, [0.0, 9.0, 70.0]);
    assert_ne!(a, b);
}

#[test]
fn same_time_renders_identically() {
    let model = Model::from_glb(AIRPORT).unwrap();
    assert_eq!(
        render(&model, 5.0, [0.0, 9.0, 70.0]),
        render(&model, 5.0, [0.0, 9.0, 70.0])
    );
}

#[test]
fn camera_inside_geometry_does_not_panic_or_leave_holes() {
    let model = Model::from_glb(AIRPORT).unwrap();
    let rgba = render(&model, 1.0, [-40.0, 4.0, -36.0]);
    assert!(rgba.chunks_exact(4).all(|p| p[3] == 255));
}

#[test]
fn nearer_geometry_hides_farther_geometry() {
    let model = Model::from_glb(AIRPORT).unwrap();
    let ground = render(&model, 1.0, [0.0, 9.0, 70.0]);
    let bottom_row = &ground[(89 * 160) * 4..(89 * 160 + 160) * 4];
    let top_row = &ground[..160 * 4];
    assert_ne!(bottom_row[..3], top_row[..3]);
}
