//! UMF (Ultimate Mugen Format): simple declarative characters compiled to the MUGEN-compatible VM.
pub mod types;
pub mod compile;
pub mod rig;
mod pixel;
pub use compile::{compile_native, compile_native_preview, compile_native_with, STANDARD_ANIMS};
pub use types::*;
