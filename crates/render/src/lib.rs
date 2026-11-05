pub mod atlas;
pub mod batch;
pub mod canvas;
pub mod gpu;
pub mod palette;
pub mod post;

pub use atlas::{Atlas, Rect};
pub use canvas::{Canvas, Color};
pub use batch::{Batch, Blend, DrawCall, PalFx, Quad, Vertex};
pub use gpu::{Renderer, Target};
pub use palette::{PaletteColors, PaletteRows};
pub use post::{Glass, PostFx, PostParams, MAX_GLASS, POST_SHADER};

pub const SHADER: &str = include_str!("sprite.wgsl");
pub const PALETTE_ROWS: u32 = 1024;
pub const RGBA_SPRITE: u32 = u32::MAX;

/// Ceiling for the draw calls a normal 2D fight frame may issue.
///
/// Quads are merged by blend state and scissor, so a frame costs one call per state change, not one
/// per sprite. The browser port this engine is benchmarked against issues one call per sprite
/// (measured: 82-105 per frame at 640x480); a state-sorted scene must stay well under that.
pub const DRAW_CALL_BUDGET: usize = 24;
