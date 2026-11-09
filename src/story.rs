mod run;
mod stages;
pub mod text;

pub use run::{Next, Phase, StoryRun};
pub use stages::{
    fight_center, fights_in, strip_span, villain_life_ratio, StageDef, FIGHT_HALF_RANGE, STAGES,
};
