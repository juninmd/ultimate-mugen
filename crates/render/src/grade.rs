pub const MAX_WAVES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grade {
    pub tonemap: f32,
    pub exposure: f32,
    pub split: f32,
    pub heat: f32,
    pub threshold: f32,
    pub shadows: [f32; 3],
    pub highlights: [f32; 3],
}

impl Default for Grade {
    fn default() -> Self {
        Grade { tonemap: 0.0, exposure: 1.0, split: 0.0, heat: 0.0, threshold: 0.58, shadows: [1.0; 3], highlights: [1.0; 3] }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Waves {
    pub n: usize,
    pub items: [[f32; 8]; MAX_WAVES],
}

impl Waves {
    pub fn push(&mut self, x: f32, y: f32, radius: f32, width: f32, strength: f32) -> bool {
        if self.n >= MAX_WAVES || radius <= 0.0 || width <= 0.0 || strength <= 0.0 {
            return false;
        }
        self.items[self.n] = [x, y, radius, width, strength, 0.0, 0.0, 0.0];
        self.n += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waves_are_bounded_and_reject_empty() {
        let mut w = Waves::default();
        assert!(!w.push(0.0, 0.0, 0.0, 4.0, 1.0));
        assert!((0..MAX_WAVES).all(|_| w.push(1.0, 1.0, 10.0, 4.0, 1.0)));
        assert!(!w.push(1.0, 1.0, 10.0, 4.0, 1.0));
    }
}
