pub mod epilogue;
pub mod epilogue_music;
pub mod objects;
pub mod objects_art;
pub mod objects_textures;
mod run;
mod stages;
pub mod text;

pub use run::{Next, Phase, StoryRun};
pub use stages::{
    fight_center, fights_in, strip_span, villain_life_ratio, StageDef, FIGHT_HALF_RANGE, STAGES,
};
