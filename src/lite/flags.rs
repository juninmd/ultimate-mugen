use crate::settings::Settings;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

static AFTERIMAGE: AtomicBool = AtomicBool::new(true);
static GLASS: AtomicBool = AtomicBool::new(true);
static SHADOWS: AtomicBool = AtomicBool::new(true);
static BACKDROP: AtomicBool = AtomicBool::new(true);
static CRT: AtomicBool = AtomicBool::new(true);
static SCALE: AtomicU32 = AtomicU32::new(0x3f80_0000);

pub fn afterimage() -> bool { AFTERIMAGE.load(Relaxed) }
pub fn glass() -> bool { GLASS.load(Relaxed) }
pub fn shadows() -> bool { SHADOWS.load(Relaxed) }
pub fn backdrop() -> bool { BACKDROP.load(Relaxed) }
pub fn crt() -> bool { CRT.load(Relaxed) }
pub fn scale() -> f32 { f32::from_bits(SCALE.load(Relaxed)) }

pub fn apply(s: &Settings) -> u8 {
    let l = &s.lite;
    AFTERIMAGE.store(l.afterimage, Relaxed);
    GLASS.store(l.glass, Relaxed);
    SHADOWS.store(l.shadows, Relaxed);
    BACKDROP.store(l.backdrop, Relaxed);
    CRT.store(l.crt, Relaxed);
    SCALE.store(l.render_scale().to_bits(), Relaxed);
    ume_shell::quad::set_render_scale(l.render_scale());
    let fx = s.fx.min(2);
    crate::vfx::quality::set(fx.min(l.particles as usize));
    fx as u8
}
