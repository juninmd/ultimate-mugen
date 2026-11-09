use crate::vfx::FxParams;
use ume_shell::quad::{set_post, PostParams};
use ume_shell::Grade;

mod look;
pub use look::Look;

pub struct PostFx {
    pub level: u8,
    pub look: Look,
}

impl PostFx {
    pub fn new() -> Self {
        PostFx { level: 2, look: Look::load() }
    }

    pub fn begin(&mut self) {}

    pub fn shockwaves_on(&self) -> bool {
        self.level >= self.look.shockwave_level && self.look.shockwave > 0.0
    }

    fn grade(&self, p: &FxParams) -> Grade {
        let l = &self.look;
        let mut g = Grade::default();
        if self.level >= l.grade_level {
            g.tonemap = l.tonemap;
            g.exposure = l.exposure;
            g.split = l.split;
            g.shadows = l.shadows;
            g.highlights = l.highlights;
        }
        if self.level >= l.heat_level {
            g.heat = ((p.radial * 0.7 + p.flash * 1.5) * l.heat).min(1.5);
        }
        g.threshold = l.threshold;
        g
    }

    pub fn end(&mut self, p: &FxParams, t: f32) {
        if self.level == 0 {
            ume_shell::waves::take();
            return;
        }
        let l = &self.look;
        set_post(Some(PostParams {
            chroma: p.chroma * l.chroma,
            vignette: p.vignette * l.vignette,
            bloom: if self.level >= l.bloom_level { p.bloom * l.bloom } else { 0.0 },
            grain: l.grain,
            saturation: p.saturation,
            contrast: 1.07,
            flash: p.flash,
            time: (t * 60.0).floor() % 97.0,
            radial: p.radial,
            center: p.center,
            tint: p.tint,
            invert: p.invert,
            glass: Default::default(),
            grade: self.grade(p),
            waves: Default::default(),
        }));
    }
}
