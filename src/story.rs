pub mod beats;
pub mod choice;
pub mod epilogue;
pub mod epilogue_edit;
pub mod epilogue_store;
pub mod helper;
pub mod prefight;
pub mod rank;
pub mod tips;
pub mod epilogue_music;
pub mod epilogue_pose;
pub mod objects;
pub mod objects_art;
pub mod objects_textures;
mod run;
mod script_source;
pub mod studio;
pub mod script;
mod stages;
mod tally;
pub mod text;

pub use helper::HelperPick;
pub use run::{Next, Phase, StoryRun};
pub use script::active as active_script;
pub use tally::StageTally;
pub use stages::{
    fight_center, fights_in, strip_span, villain_life_ratio, StageDef, FIGHT_HALF_RANGE, STAGES,
};
