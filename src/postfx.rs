use crate::vfx::FxParams;
use ume_shell::quad::{set_post, PostParams};

pub struct PostFx { pub level: u8 }

impl PostFx {
    pub fn new() -> Self { PostFx { level: 2 } }

    pub fn begin(&mut self) {}

    pub fn end(&mut self, p: &FxParams, t: f32) {
        if self.level == 0 { return; }
        set_post(Some(PostParams {
            chroma: p.chroma,
            vignette: p.vignette,
            bloom: if self.level >= 2 { p.bloom } else { 0.0 },
            grain: 0.035,
            saturation: p.saturation,
            contrast: 1.07,
            flash: p.flash,
            time: (t * 60.0).floor() % 97.0,
            radial: p.radial,
            center: p.center,
            tint: p.tint,
            invert: p.invert,
            glass: Default::default(),
        }));
    }
}
