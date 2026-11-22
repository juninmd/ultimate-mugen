use crate::postfx::Look;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    Off,
    Cinema,
    Noir,
    Vivid,
    Retro,
}

impl Filter {
    pub const ALL: [Filter; 5] = [Filter::Off, Filter::Cinema, Filter::Noir, Filter::Vivid, Filter::Retro];

    pub fn next(self) -> Filter {
        let at = Filter::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Filter::ALL[(at + 1) % Filter::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        use crate::i18n::tr3;
        match self {
            Filter::Off => tr3("Natural", "Natural", "Natural"),
            Filter::Cinema => tr3("Cinema", "Cinema", "Cine"),
            Filter::Noir => tr3("Noir", "Noir", "Noir"),
            Filter::Vivid => tr3("Vívido", "Vivid", "Vívido"),
            Filter::Retro => tr3("Retrô", "Retro", "Retro"),
        }
    }

    pub fn apply(self, base: &Look) -> Look {
        let mut l = base.clone();
        match self {
            Filter::Off => {}
            Filter::Cinema => {
                l.vignette = 1.0;
                l.grain = 0.06;
                l.tonemap = 0.8;
                l.split = 0.9;
                l.exposure = 0.98;
                l.saturation = 1.1;
                l.shadows = [0.8, 0.95, 1.15];
                l.highlights = [1.15, 1.02, 0.85];
            }
            Filter::Noir => {
                l.saturation = 0.0;
                l.vignette = 1.2;
                l.grain = 0.09;
                l.tonemap = 0.7;
                l.exposure = 1.1;
                l.split = 0.0;
                l.shadows = [1.0; 3];
                l.highlights = [1.0; 3];
            }
            Filter::Vivid => {
                l.saturation = 1.45;
                l.bloom = 1.6;
                l.sharpen = 0.5;
                l.vignette = 0.6;
                l.exposure = 1.05;
            }
            Filter::Retro => {
                l.saturation = 0.55;
                l.grain = 0.14;
                l.vignette = 1.1;
                l.chroma = 2.0;
                l.split = 0.8;
                l.shadows = [1.0, 0.9, 0.75];
                l.highlights = [1.12, 1.0, 0.8];
            }
        }
        l
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycling_visits_every_filter_and_wraps() {
        let mut f = Filter::Off;
        let mut seen = vec![f];
        for _ in 0..Filter::ALL.len() - 1 {
            f = f.next();
            seen.push(f);
        }
        assert_eq!(seen, Filter::ALL);
        assert_eq!(f.next(), Filter::Off);
    }

    #[test]
    fn natural_leaves_the_look_untouched_and_noir_removes_color() {
        let base = Look::parse("");
        assert_eq!(Filter::Off.apply(&base), base);
        assert_eq!(Filter::Noir.apply(&base).saturation, 0.0);
        assert!(Filter::Vivid.apply(&base).saturation > base.saturation);
    }

    #[test]
    fn every_filter_keeps_values_in_the_ranges_the_loader_allows() {
        let base = Look::parse("");
        for f in Filter::ALL {
            let l = f.apply(&base);
            assert!((0.0..=2.0).contains(&l.saturation));
            assert!((0.0..=0.2).contains(&l.grain) && (0.0..=3.0).contains(&l.vignette));
            assert!((0.4..=2.0).contains(&l.exposure));
        }
    }
}
