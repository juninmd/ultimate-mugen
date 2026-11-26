use crate::model::{Model, Posed};

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub pixels_per_unit: f32,
    pub floor: f32,
    pub yaw_offset: f32,
    pub pitch: f32,
    pub lens: f32,
    pub pivot: f32,
    pub span: f32,
}

impl Camera {
    pub fn fit_height(model: &Model, pixels: f32) -> Camera {
        let mut posed = Posed::default();
        model.pose((!model.clips.is_empty()).then_some(0), 0.0, &mut posed);
        let ys = posed.positions.iter().flatten().map(|p| p[1]);
        let (lo, hi) = ys.fold((f32::MAX, f32::MIN), |(lo, hi), y| (lo.min(y), hi.max(y)));
        Camera {
            pixels_per_unit: pixels / (hi - lo).max(f32::EPSILON),
            floor: lo,
            yaw_offset: 0.0,
            pitch: 0.0,
            lens: 0.0,
            pivot: (lo + hi) / 2.0,
            span: (hi - lo).max(f32::EPSILON),
        }
    }
}
