use super::Vfx;

const ROW: f32 = 44.0;
const BOX_ABOVE_BASELINE: f32 = 24.0;

fn presence(t: f32) -> f32 {
    ((1.6 - t) / 0.4).clamp(0.0, 1.0)
}

impl Vfx {
    pub fn below_banners(&self, min_top: f32) -> f32 {
        let rows: f32 = self.banners.iter().map(|b| presence(b.t)).sum();
        if rows <= 0.0 {
            return min_top;
        }
        min_top.max(self.banner_top - BOX_ABOVE_BASELINE + ROW * rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ribbons_clear_every_banner_and_slide_back_as_they_fade() {
        let mut v = Vfx::new();
        assert_eq!(v.below_banners(116.0), 116.0);
        v.banner("A", "", [1.0; 3]);
        v.banner("B", "", [1.0; 3]);
        let stacked = v.below_banners(116.0);
        assert!(stacked >= v.banner_top + ROW + 11.5);
        v.banners[1].t = 1.5;
        assert!(v.below_banners(116.0) < stacked);
    }
}
