use super::*;
use ume_formats::stage::{load_stage, StageImage};
use ume_formats::vfs::Vfs;

fn preview_of(layer_extra: &str) -> Vec<u8> {
    let def = format!("[BGdef]\n\n[BG square]\ntype = normal\nspriteno = 0, 0\nstart = 0, 120\n{layer_extra}\n");
    let mut vfs = Vfs::new();
    vfs.insert("stages/t.def".into(), def.into_bytes());
    let mut st = load_stage(&vfs, "stages/t.def").unwrap();
    let rgba = [255u8, 0, 0, 255].repeat(20 * 20);
    st.sprites.insert(
        (0, 0),
        StageImage {
            w: 20,
            h: 20,
            axis_x: 10,
            axis_y: 10,
            rgba,
        },
    );
    mugen_preview(&st).unwrap().rgba.to_vec()
}

fn red_pixels(rgba: &[u8]) -> Vec<(usize, usize)> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, p)| p[0] == 255 && p[1] == 0)
        .map(|(i, _)| (i % 320, i / 320))
        .collect()
}

fn extent(points: &[(usize, usize)]) -> (usize, usize) {
    let xs = points.iter().map(|p| p.0);
    let ys = points.iter().map(|p| p.1);
    (
        xs.clone().max().unwrap() - xs.min().unwrap() + 1,
        ys.clone().max().unwrap() - ys.min().unwrap() + 1,
    )
}

#[test]
fn preview_applies_layer_scale() {
    let base = red_pixels(&preview_of(""));
    let scaled = red_pixels(&preview_of("scalestart = 2, 2"));
    assert_eq!(extent(&base).0, 20);
    assert_eq!(extent(&scaled).0, 40);
    assert!(extent(&scaled).1.abs_diff(extent(&base).1 * 2) <= 2);
    assert!(scaled.len() > base.len() * 3);
}

#[test]
fn preview_applies_angle_and_shear() {
    let base = red_pixels(&preview_of(""));
    let quarter_turn = red_pixels(&preview_of("angle = 90"));
    let rotated = red_pixels(&preview_of("angle = 45"));
    let sheared = red_pixels(&preview_of("xshear = 1"));
    assert!(extent(&quarter_turn).0.abs_diff(extent(&base).1) <= 2);
    assert!(extent(&quarter_turn).1.abs_diff(extent(&base).0) <= 2);
    assert!(extent(&rotated).0 > extent(&base).0 + 2);
    assert!((rotated.len() as f32 - base.len() as f32).abs() < base.len() as f32 * 0.1);
    assert!(extent(&sheared).0 > extent(&base).0 + 4);
    assert!(extent(&sheared).1.abs_diff(extent(&base).1) <= 1);
}

#[test]
fn preview_clips_to_window_intersection() {
    let clipped = red_pixels(&preview_of(
        "window = -10, 100, 10, 140\nmaskwindow = 0, 100, 100, 140",
    ));
    assert_eq!(extent(&clipped).0, 10);
    assert!(clipped.iter().all(|p| p.0 >= 160));
}
