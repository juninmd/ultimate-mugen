use super::rig::render_pose;
use super::types::{ActiveSpec, Box4, NativeChar, NativeFrame};
use serde_json::Value;

pub const STANDING_UNITS: f32 = 96.0;
pub const BODY_START: f32 = 0.10;
pub const MIN_REACH: f32 = 0.60;
pub const MIN_HEIGHT: f32 = 0.36;
pub const HIGHEST_TOP: f32 = -0.95;

pub fn standing_height(nc: &NativeChar) -> f32 {
    let declared = nc.stats.height.filter(|height| height.is_finite() && *height > 0.0).unwrap_or(100.0) as f32;
    STANDING_UNITS * declared / 100.0
}

pub(super) fn declared_strike_boxes(nc: &NativeChar, fr: &NativeFrame) -> Option<Vec<Box4>> {
    let active = fr.hit.as_ref()?;
    if let Some(explicit) = active.bx {
        return Some(vec![explicit]);
    }
    if nc.art.kind != "rig" {
        return None;
    }
    let pose = match fr.pose.as_ref()? {
        Value::String(name) => nc.art.poses.get(name)?.clone(),
        other => other.clone(),
    };
    let mut geometry = render_pose(&nc.art.style, &pose, Some(active), false, None);
    if geometry.hit.is_empty() {
        let leading_limb = ActiveSpec { part: "handF".into(), radius: active.radius, bx: None };
        geometry = render_pose(&nc.art.style, &pose, Some(&leading_limb), false, None);
    }
    let height = standing_height(nc);
    (!geometry.hit.is_empty()).then(|| geometry.hit.into_iter().map(|tip| playable_strike_box(tip, height)).collect())
}

pub fn playable_strike_box(mut bounds: Box4, height: f32) -> Box4 {
    let faces_forward = (bounds[0] + bounds[2]) * 0.5 > 0.0;
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
