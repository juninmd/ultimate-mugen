pub const GRAVITY_MIN: f32 = 0.25;
pub const GRAVITY_MAX: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mutators {
    pub gravity: f32,
}

impl Default for Mutators {
    fn default() -> Self {
        Self { gravity: 1.0 }
    }
}

impl Mutators {
    pub fn with_gravity(gravity: f32) -> Option<Self> {
        (gravity.is_finite() && (GRAVITY_MIN..=GRAVITY_MAX).contains(&gravity)).then_some(Self { gravity })
    }

    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}
