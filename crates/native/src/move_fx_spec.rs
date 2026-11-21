use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExplodSpec {
    pub at: i32,
    pub anim: String,
    #[serde(default)]
    pub pos: Option<[f32; 2]>,
    #[serde(default)]
    pub postype: Option<String>,
    #[serde(default)]
    pub bind: Option<i32>,
    #[serde(default, rename = "removeTime")]
    pub remove_time: Option<i32>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub scale: Option<[f32; 2]>,
    #[serde(default, rename = "superMove")]
    pub super_move: Option<bool>,
    #[serde(default)]
    pub vel: Option<[f32; 2]>,
    #[serde(default)]
    pub accel: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AfterImageSpec {
    #[serde(default)]
    pub at: Option<i32>,
    #[serde(default)]
    pub time: Option<i32>,
    #[serde(default)]
    pub length: Option<i32>,
    #[serde(default)]
    pub framegap: Option<i32>,
    #[serde(default)]
    pub trans: Option<String>,
    #[serde(default, rename = "palBright")]
    pub pal_bright: Option<[i32; 3]>,
    #[serde(default, rename = "palContrast")]
    pub pal_contrast: Option<[i32; 3]>,
    #[serde(default, rename = "palAdd")]
    pub pal_add: Option<[i32; 3]>,
    #[serde(default, rename = "palMul")]
    pub pal_mul: Option<[f32; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShakeSpec {
    #[serde(default)]
    pub at: Option<i32>,
    pub time: i32,
    #[serde(default)]
    pub ampl: Option<f32>,
    #[serde(default)]
    pub freq: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlashSpec {
    #[serde(default)]
    pub at: Option<i32>,
    pub time: i32,
    #[serde(default)]
    pub color: Option<[i32; 3]>,
    #[serde(default)]
    pub under: Option<bool>,
    #[serde(default, rename = "superMove")]
    pub super_move: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SuperPauseSpec {
    #[serde(default)]
    pub at: Option<i32>,
    #[serde(default)]
    pub time: Option<i32>,
    #[serde(default)]
    pub pos: Option<[f32; 2]>,
    #[serde(default)]
    pub anim: Option<i32>,
    #[serde(default)]
    pub darken: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeleportMoveSpec {
    pub at: i32,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub offset: Option<[f32; 2]>,
    #[serde(default)]
    pub turn: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TeleportStep {
    At(i32),
    Spec(TeleportMoveSpec),
}

impl TeleportStep {
    pub fn at(&self) -> i32 {
        match self {
            TeleportStep::At(frame) => *frame,
            TeleportStep::Spec(spec) => spec.at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FollowUpSpec {
    pub to: String,
    #[serde(default)]
    pub on: Option<String>,
    #[serde(default)]
    pub at: Option<i32>,
}
