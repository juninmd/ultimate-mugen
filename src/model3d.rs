use crate::render::Gfx;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

#[cfg(feature = "model3d")]
mod active;
#[cfg(feature = "model3d")]
mod bindings;
#[cfg(feature = "model3d")]
mod canvas;
#[cfg(feature = "model3d")]
mod fit;
#[cfg(feature = "model3d")]
mod portrait;
#[cfg(feature = "model3d")]
mod pose_pick;
#[cfg(feature = "model3d")]
mod specs;

#[cfg(feature = "model3d")]
pub(crate) fn draw(g: &mut Gfx, m: &Match, f: &Fighter, shx: f32, shy: f32) -> bool {
    active::draw(g, m, f, shx, shy)
}

#[cfg(not(feature = "model3d"))]
pub(crate) fn draw(_: &mut Gfx, _: &Match, _: &Fighter, _: f32, _: f32) -> bool {
    false
}

#[cfg(feature = "model3d")]
pub(crate) fn is_3d(id: &str) -> bool {
    specs::spec(id.trim_start_matches("umf:"), false).is_some()
}

#[cfg(not(feature = "model3d"))]
pub(crate) fn is_3d(_: &str) -> bool {
    false
}

#[cfg(feature = "model3d")]
pub(crate) fn draw_fit(
    id: &str,
    anim: i32,
    ticks: i32,
    center_x: f32,
    bottom: f32,
    height: f32,
    mirror: bool,
) -> bool {
    is_3d(id) && fit::draw(id, anim, ticks, center_x, bottom, height, mirror)
}

#[cfg(not(feature = "model3d"))]
pub(crate) fn draw_fit(_: &str, _: i32, _: i32, _: f32, _: f32, _: f32, _: bool) -> bool {
    false
}

#[cfg(feature = "model3d")]
pub(crate) fn portrait(id: &str) -> Option<ume_engine::chardef::Sprite> {
    portrait::portrait(id.trim_start_matches("umf:"))
}

#[cfg(not(feature = "model3d"))]
pub(crate) fn portrait(_: &str) -> Option<ume_engine::chardef::Sprite> {
    None
}

#[cfg(feature = "model3d")]
pub(crate) fn hud_portrait(
    def: &ume_engine::chardef::CharDef,
) -> Option<(
    ume_engine::chardef::Sprite,
    std::sync::Arc<ume_engine::chardef::ImageData>,
    bool,
)> {
    let sprite = portrait(&def.id)?;
    let image = sprite.image.clone()?;
    Some((sprite, image, true))
}

#[cfg(not(feature = "model3d"))]
pub(crate) fn hud_portrait(
    _: &ume_engine::chardef::CharDef,
) -> Option<(
    ume_engine::chardef::Sprite,
    std::sync::Arc<ume_engine::chardef::ImageData>,
    bool,
)> {
    None
}
