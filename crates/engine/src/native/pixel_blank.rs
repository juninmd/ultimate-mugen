use super::effects::FrameArt;
use super::pixel::{hurtbox, pose_stem, STAND_UNITS, STRIKE_BOX};
use super::types::{NativeChar, NativeFrame};
use super::native_frame_sprite_key;

pub(super) fn boxes_only(nc: &NativeChar, fr: &NativeFrame) -> FrameArt {
    let lower = native_frame_sprite_key(fr).to_ascii_lowercase();
    let height = nc.stats.height.filter(|height| height.is_finite() && *height > 0.0).unwrap_or(STAND_UNITS as f64) as f32;
    let mut art = FrameArt {
        image: None,
        width: 0.0,
        height: 0.0,
        axis_x: 0.0,
        axis_y: 0.0,
        scale: 1.0,
        hurt: vec![hurtbox(&lower, pose_stem(&lower))],
        hit: if fr.hit.is_some() { vec![STRIKE_BOX] } else { vec![] },
    };
    super::pixel_hit::fit_to_character(nc, fr, &mut art, height / STAND_UNITS);
    art
}
