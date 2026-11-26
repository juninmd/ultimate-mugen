use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ease {
    Linear,
    In,
    Out,
    #[default]
    InOut,
    Back,
}

impl Ease {
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Ease::Linear => t,
            Ease::In => t * t * t,
            Ease::Out => 1.0 - (1.0 - t).powi(3),
            Ease::InOut => t * t * (3.0 - 2.0 * t),
            Ease::Back => {
                let c1 = 1.701_58;
                let u = t - 1.0;
                1.0 + (c1 + 1.0) * u * u * u + c1 * u * u
            }
        }
    }
}
