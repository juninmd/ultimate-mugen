use ume_model3d::model::Posed;
use ume_model3d::state_map::clip_time;
use ume_model3d::{Camera, Facing, Model, StateMap};

const FOX: &[u8] = include_bytes!("data/Fox.glb");
const STATES: &str = include_str!("data/fox_states.json");

fn fox() -> Model {
    Model::from_glb(FOX).expect("fox loads")
}

#[test]
fn loads_skeleton_and_three_clips() {
    let m = fox();
    let names: Vec<_> = m.clips.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Survey", "Walk", "Run"]);
    assert!(m.clips.iter().all(|c| c.duration > 0.0));
}

#[test]
fn rejects_garbage_and_truncated_input() {
    assert!(Model::from_glb(b"not a glb").is_err());
    assert!(Model::from_glb(&FOX[..FOX.len() / 2]).is_err());
    assert!(Model::from_glb(&[]).is_err());
}

#[test]
fn animation_moves_vertices_and_loops() {
    let m = fox();
    let run = m.clip_index("Run").unwrap();
    let (mut a, mut b, mut c) = (Posed::default(), Posed::default(), Posed::default());
    m.pose(Some(run), 0.0, &mut a);
    m.pose(Some(run), m.clips[run].duration * 0.37, &mut b);
    m.pose(Some(run), m.clips[run].duration, &mut c);
    assert_ne!(a.positions, b.positions);
    let drift = a
        .positions
        .iter()
        .flatten()
        .zip(c.positions.iter().flatten())
        .map(|(x, y)| (x[0] - y[0]).abs() + (x[1] - y[1]).abs() + (x[2] - y[2]).abs())
        .fold(0.0, f32::max);
    assert!(drift < 1.0, "loop seam drift {drift}");
}

#[test]
fn state_map_picks_clip_by_mugen_stateno_with_fallback() {
    let map = StateMap::from_json(STATES).unwrap();
    assert_eq!(map.resolve(20).clip, "Walk");
    assert_eq!(map.resolve(0).clip, "Survey");
    assert_eq!(map.resolve(430).clip, "Run");
    assert!(!map.resolve(430).looping);
    assert!(StateMap::from_json("{}").is_err());
}

#[test]
fn clip_time_wraps_when_looping_and_clamps_otherwise() {
    let map = StateMap::from_json(STATES).unwrap();
    assert!((clip_time(map.resolve(20), 600, 0.7) - (10.0 % 0.7)).abs() < 1e-4);
    assert_eq!(clip_time(map.resolve(430), 6000, 0.7), 0.7);
    assert_eq!(clip_time(map.resolve(430), -5, 0.7), 0.0);
}

#[test]
fn projection_fits_requested_height_and_flips_with_facing() {
    let m = fox();
    let cam = Camera::fit_height(&m, 120.0);
    let mut posed = Posed::default();
    m.pose_for_state(&StateMap::from_json(STATES).unwrap(), 0, 0, &mut posed);
    let (mut right, mut left) = (Vec::new(), Vec::new());
    m.project(&posed, &cam, Facing::Right, [0.0, 0.0], &mut right);
    m.project(&posed, &cam, Facing::Left, [0.0, 0.0], &mut left);
    let ys: Vec<f32> = right
        .iter()
        .flat_map(|t| t.points.iter().map(|p| p[1]))
        .collect();
    let span =
        ys.iter().cloned().fold(f32::MIN, f32::max) - ys.iter().cloned().fold(f32::MAX, f32::min);
    assert!((span - 120.0).abs() < 6.0, "span {span}");
    assert!(
        ys.iter().cloned().fold(f32::MIN, f32::max) < 6.0,
        "feet on the floor"
    );
    let mean = |v: &Vec<ume_model3d::Triangle>| {
        v.iter().map(|t| t.points[0][0]).sum::<f32>() / v.len() as f32
    };
    assert!(mean(&right) * mean(&left) < 0.0 || (mean(&right) - mean(&left)).abs() > 1.0);
    assert!(right.windows(2).all(|w| w[0].depth <= w[1].depth));
}
