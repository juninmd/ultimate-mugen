//! UMF (Ultimate Mugen Format): simple declarative characters compiled to the MUGEN-compatible VM.
pub mod types;
pub mod compile;
pub mod rig;
pub use compile::{compile_native, STANDARD_ANIMS};
pub use types::*;
