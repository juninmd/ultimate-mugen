use crate::render::Gfx;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

#[cfg(feature = "model3d")]
mod active;
#[cfg(feature = "model3d")]
mod bindings;

#[cfg(feature = "model3d")]
pub(crate) fn draw(g: &mut Gfx, m: &Match, f: &Fighter, shx: f32, shy: f32) -> bool {
    active::draw(g, m, f, shx, shy)
}

#[cfg(not(feature = "model3d"))]
pub(crate) fn draw(_: &mut Gfx, _: &Match, _: &Fighter, _: f32, _: f32) -> bool {
    false
}
