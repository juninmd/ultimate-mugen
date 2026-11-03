pub mod atlas;
pub mod batch;
pub mod canvas;
pub mod gpu;

pub use atlas::{Atlas, Rect};
pub use canvas::{Canvas, Color};
pub use batch::{Batch, Blend, DrawCall, Quad, Vertex};
pub use gpu::{Renderer, Target};

pub const SHADER: &str = include_str!("sprite.wgsl");
pub const PALETTE_ROWS: u32 = 1024;
pub const RGBA_SPRITE: u32 = u32::MAX;
