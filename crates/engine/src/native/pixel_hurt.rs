const BODY_STEMS: [&str; 3] = ["12_fall1", "13_fall2", "14_lie"];
const OPAQUE: u8 = 40;
const SIDE_INSET: f32 = 0.12;
const TOP_INSET: f32 = 0.10;
const MIN_SIDE: f32 = 18.0;

pub(super) fn fit_body(
    pose_box: [f32; 4],
    stem: &str,
    rgba: &[u8],
    width: u32,
    height: u32,
    axis: [f32; 2],
    units_per_pixel: f32,
) -> [f32; 4] {
    if !BODY_STEMS.contains(&stem) {
        return pose_box;
    }
    opaque_bounds(rgba, width, height)
        .map_or(pose_box, |bounds| inset_box(bounds, axis, units_per_pixel))
}

fn opaque_bounds(rgba: &[u8], width: u32, height: u32) -> Option<[u32; 4]> {
    let (mut left, mut top, mut right, mut bottom) = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..height {
        for x in 0..width {
            if rgba
                .get(((y * width + x) * 4 + 3) as usize)
                .copied()
                .unwrap_or(0)
                < OPAQUE
            {
                continue;
            }
            left = left.min(x);
            top = top.min(y);
            right = right.max(x + 1);
            bottom = bottom.max(y + 1);
        }
    }
    (left < right && top < bottom).then_some([left, top, right, bottom])
}

fn inset_box(bounds: [u32; 4], axis: [f32; 2], units_per_pixel: f32) -> [f32; 4] {
    let to_units = |pixel: u32, origin: f32| (pixel as f32 - origin) * units_per_pixel;
    let (left, right) = (to_units(bounds[0], axis[0]), to_units(bounds[2], axis[0]));
    let (top, bottom) = (
        to_units(bounds[1], axis[1]),
        to_units(bounds[3], axis[1]).min(0.0),
    );
    let (width, tall) = (right - left, bottom - top);
    let (side, lift) = (
        (width * SIDE_INSET).min((width - MIN_SIDE).max(0.0) / 2.0),
        (tall * TOP_INSET).min((tall - MIN_SIDE).max(0.0) / 2.0),
    );
    [left + side, top + lift, right - side, bottom]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite(width: u32, height: u32, body: [u32; 4]) -> Vec<u8> {
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        for y in body[1]..body[3] {
            for x in body[0]..body[2] {
                rgba[((y * width + x) * 4 + 3) as usize] = 255;
            }
        }
        rgba
    }

    #[test]
    fn a_body_lying_across_the_ground_gets_a_flat_wide_box() {
        let rgba = sprite(200, 100, [10, 60, 190, 100]);
        let lying = fit_body(
            [-22.0, -96.0, 22.0, 0.0],
            "14_lie",
            &rgba,
            200,
            100,
            [100.0, 100.0],
            1.0,
        );
        assert!(lying[2] - lying[0] > 120.0, "wide: {lying:?}");
        assert!(lying[3] - lying[1] < 40.0, "flat: {lying:?}");
        assert!(lying[3] <= 0.0, "never below the floor: {lying:?}");
    }

    #[test]
    fn a_falling_body_follows_its_opaque_pixels() {
        let rgba = sprite(100, 100, [30, 20, 70, 80]);
        let falling = fit_body(
            [-22.0, -96.0, 22.0, 0.0],
            "12_fall1",
            &rgba,
            100,
            100,
            [50.0, 100.0],
            1.0,
        );
        assert!(
            falling[1] > -82.0 && falling[3] <= -20.0 + 1.0,
            "box hugs the sprite, not the standing box: {falling:?}"
        );
    }

    #[test]
    fn standing_and_empty_poses_keep_the_pose_box() {
        let standing = [-22.0, -96.0, 22.0, 0.0];
        assert_eq!(
            fit_body(
                standing,
                "01_idle",
                &sprite(10, 10, [0, 0, 10, 10]),
                10,
                10,
                [5.0, 10.0],
                1.0
            ),
            standing
        );
        assert_eq!(
            fit_body(
                standing,
                "14_lie",
                &sprite(10, 10, [0, 0, 0, 0]),
                10,
                10,
                [5.0, 10.0],
                1.0
            ),
            standing
        );
    }
}
