use super::designs::all;
use super::*;

fn inside(x: f32, y: f32) -> bool {
    (-0.001..=1.001).contains(&x) && (-0.001..=1.001).contains(&y)
}

#[test]
fn codes_are_unique_and_lookup_ignores_case() {
    let codes: Vec<&str> = all().map(|(code, _)| code).collect();
    for (n, code) in codes.iter().enumerate() {
        assert!(!codes[..n].contains(code), "{code} repeats");
    }
    assert!(design("JP").is_some());
    assert!(design("zz").is_none());
}

#[test]
fn every_shape_stays_inside_its_flag() {
    for (code, shapes) in all() {
        assert!(!shapes.is_empty(), "{code} is empty");
        for shape in shapes {
            let ok = match *shape {
                Shape::Rect { x, y, w, h, .. } => inside(x, y) && inside(x + w, y + h),
                Shape::Disc { x, y, r, .. }
                | Shape::Half { x, y, r, .. }
                | Shape::Star { x, y, r, .. } => {
                    inside(x - r / ASPECT, y - r) && inside(x + r / ASPECT, y + r)
                }
                Shape::Poly { points, .. } => {
                    points.len() >= 3
                        && points.len() <= 8
                        && points.iter().all(|p| inside(p[0], p[1]))
                }
            };
            assert!(ok, "{code}: {shape:?} leaves the flag");
        }
    }
}

#[test]
fn every_bundled_profile_names_a_drawable_flag() {
    for (id, profile) in crate::profile::bundled_all() {
        assert!(
            design(&profile.country).is_some(),
            "{id} has no flag for '{}'",
            profile.country
        );
    }
}
