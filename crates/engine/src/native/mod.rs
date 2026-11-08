//! UMF (Ultimate Mugen Format): simple declarative characters compiled to the MUGEN-compatible VM.
mod abilities;
pub mod ability_spec;
mod air_variant;
mod armor;
mod chain;
mod charge;
pub mod compile;
pub mod forms;
pub mod lookup;
mod multihit;
mod pixel;
pub mod rig;
mod throw;
pub mod types;
pub(crate) use charge::{CHARGE_COMMAND, CHARGE_STATE};
mod pack;
mod pixel_hit;
mod pixel_hurt;
mod portrait;
mod recolor;
pub use ability_spec::*;
pub use compile::{compile_native, compile_native_preview, compile_native_with, STANDARD_ANIMS};
pub use lookup::register_native;
pub use types::*;

pub use pack::{
    contains as has_sprite_pack, insert as insert_sprite_pack, revision as sprite_pack_revision,
};
pub use pixel::{audit_native_sprite_frames, native_frame_sprite_key};
