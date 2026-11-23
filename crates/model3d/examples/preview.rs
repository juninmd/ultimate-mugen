use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};
use ume_model3d::model::Posed;
use ume_model3d::{Camera, Facing, Model, StateMap};

fn draw(pix: &mut Pixmap, tris: &[ume_model3d::Triangle]) {
    for t in tris {
        let mut pb = PathBuilder::new();
        pb.move_to(t.points[0][0], t.points[0][1]);
        pb.line_to(t.points[1][0], t.points[1][1]);
        pb.line_to(t.points[2][0], t.points[2][1]);
        pb.close();
        let Some(path) = pb.finish() else { continue };
        let mut paint = Paint::default();
        paint.set_color_rgba8(t.color[0], t.color[1], t.color[2], 255);
        pix.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        let mut edge = Paint::default();
        edge.set_color_rgba8(t.color[0], t.color[1], t.color[2], 255);
        pix.stroke_path(
            &path,
            &edge,
            &tiny_skia::Stroke {
                width: 0.8,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = Model::from_glb(&std::fs::read(&args[1]).unwrap()).unwrap();
    let map = StateMap::from_json(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let cam = Camera::fit_height(&model, 150.0);
    let states = [
        (0, "stand 0"),
        (20, "walk 20"),
        (100, "run 100"),
        (430, "attack 430"),
    ];
    let ticks = [0, 12, 24, 36, 48];
    let (w, h) = (ticks.len() as u32 * 280, states.len() as u32 * 210);
    let mut pix = Pixmap::new(w, h).unwrap();
    pix.fill(tiny_skia::Color::from_rgba8(28, 30, 44, 255));
    let (mut posed, mut tris) = (Posed::default(), Vec::new());
    for (row, (stateno, _)) in states.iter().enumerate() {
        for (col, tick) in ticks.iter().enumerate() {
            model.pose_for_state(&map, *stateno, *tick, &mut posed);
            let facing = if row % 2 == 0 {
                Facing::Right
            } else {
                Facing::Left
            };
            model.project(
                &posed,
                &cam,
                facing,
                [140.0 + col as f32 * 280.0, 190.0 + row as f32 * 210.0],
                &mut tris,
            );
            draw(&mut pix, &tris);
        }
    }
    pix.save_png(&args[3]).unwrap();
}
