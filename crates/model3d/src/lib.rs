pub mod camera;
pub mod clip;
pub mod error;
pub mod glb;
pub mod math;
pub mod model;
pub mod pose;
pub mod scene;
pub mod skin;
pub mod state_map;

mod accessor;
mod mesh_read;
mod skeleton_read;

pub use camera::Camera;
pub use error::ModelError;
pub use model::Model;
pub use scene::{Facing, Triangle};
pub use state_map::{ClipBinding, StateMap};
