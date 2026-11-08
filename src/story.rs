mod run;
mod stages;
pub mod text;

pub use run::{Next, Phase, StoryRun};
pub use stages::{StageDef, fight_center, fights_in, strip_span, FIGHT_HALF_RANGE, SEGMENT_WORLD, STAGES};
