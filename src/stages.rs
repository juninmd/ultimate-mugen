//! Original procedural stages + MUGEN stage adapter.
//!
//! Every built-in stage is a stack of parallax layers rasterized once with tiny-skia plus procedural noise passes (`texgen`):
//! sky/cloud noise, brick, wood grain, stone, sand, water... Layers flagged `dynamic` are empty placeholders that only carry a
//! parallax transform: the animated parts (flags, foliage, crowd, lava, neon flicker...) are drawn per frame by `stagefx`.
use crate::audio::MusicLoop;
use crate::texgen::{blobs, grain, hash, haze, n1, shade_v};
use std::sync::Arc;
use ume_engine::chardef::ImageData;
use ume_engine::matchs::StageInfo;
use ume_engine::raster::*;
use ume_formats::stage::MugenStage;


pub mod ambience;
pub mod interactive;
mod story_strip;
#[cfg(target_arch = "wasm32")]
mod web_decode;
pub use story_strip::*;
mod wonders; pub(crate) mod looping; pub(crate) mod story_loop; pub(crate) mod scenery;
mod types;
mod layer;
mod paint;
mod props;
mod dojo;
mod neon;
mod bamboo;
mod plains;
mod spirit;
mod volcano;
#[cfg(feature = "model3d")]
mod airport;
mod illustrations;
mod catalog;
mod mugen;
mod preview;
pub use types::*;
use layer::*;
use paint::*;
pub use props::*;
use dojo::dojo;
use neon::neon;
use bamboo::bamboo;
use plains::plains;
use spirit::spirit;
use volcano::volcano;
pub use illustrations::*;
pub use catalog::*;
pub use mugen::*;
pub use preview::*;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod illustration_tests;
