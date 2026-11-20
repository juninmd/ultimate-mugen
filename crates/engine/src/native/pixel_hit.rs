use super::effects::FrameArt;
use super::types::{ActiveSpec, Box4, NativeChar, NativeFrame};

pub const STANDING_UNITS: f32 = 96.0;
pub const BODY_START: f32 = 0.10;
pub const MIN_REACH: f32 = 0.60;
pub const MIN_HEIGHT: f32 = 0.36;
const BACK_STRIKE_CENTER: f32 = -0.30;
pub const HIGHEST_TOP: f32 = -0.95;

pub fn standing_height(nc: &NativeChar) -> f32 {
    let declared = nc.stats.height.filter(|height| height.is_finite() && *height > 0.0).unwrap_or(100.0) as f32;
    STANDING_UNITS * declared / 100.0
}

pub(super) fn fit_to_character(nc: &NativeChar, fr: &NativeFrame, render: &mut FrameArt, height_ratio: f32) {
    if let Some(boxes) = declared_strike_boxes(nc, fr) {
        render.hit = boxes;
    }
    for bounds in &mut render.hurt {
        for edge in bounds.iter_mut() {
            *edge *= height_ratio;
        }
    }
}

fn declared_strike_boxes(nc: &NativeChar, fr: &NativeFrame) -> Option<Vec<Box4>> {
    let active = fr.hit.as_ref()?;
    if let Some(explicit) = active.bx {
        return Some(vec![explicit]);
    }
    let height = standing_height(nc);
    Some(vec![playable_strike_box(seed_strike_box(active, height), height)])
}

fn seed_strike_box(active: &ActiveSpec, height: f32) -> Box4 {
    let part = active.part.to_ascii_lowercase();
    let (top, bottom) = if ["foot", "knee", "shin", "leg", "ankle"].iter().any(|low| part.contains(low)) {
        (-0.42, -0.02)
    } else if part.contains("head") {
        (-0.95, -0.70)
    } else {
        (-0.78, -0.42)
    };
    let reach = 0.55 * height + active.radius.unwrap_or(10.0);
    [BODY_START * height, top * height, reach, bottom * height]
}

pub fn playable_strike_box(mut bounds: Box4, height: f32) -> Box4 {
    let faces_forward = (bounds[0] + bounds[2]) * 0.5 > BACK_STRIKE_CENTER * height;
    bounds[1] = bounds[1].max(HIGHEST_TOP * height);
    bounds[3] = bounds[3].min(0.0);
    if bounds[3] - bounds[1] < MIN_HEIGHT * height {
        bounds[3] = (bounds[1] + MIN_HEIGHT * height).min(0.0);
        bounds[1] = bounds[3] - MIN_HEIGHT * height;
    }
    if faces_forward {
        bounds[0] = bounds[0].min(BODY_START * height);
        bounds[2] = bounds[2].max(MIN_REACH * height);
    } else {
        let missing = MIN_HEIGHT * height - (bounds[2] - bounds[0]);
        if missing > 0.0 {
            bounds[0] -= missing * 0.5;
            bounds[2] += missing * 0.5;
        }
    }
    bounds
}
