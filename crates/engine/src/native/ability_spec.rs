use super::types::NativeMove;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AbilitySpec {
    #[serde(default)] pub fly: Option<FlySpec>,
    #[serde(default)] pub teleport: Option<TeleportSpec>,
    #[serde(default, rename = "superSpeed")] pub super_speed: Option<SuperSpeedSpec>,
    #[serde(default)] pub forms: Vec<FormSpec>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlySpec {
    #[serde(default)] pub speed: Option<f32>,
    #[serde(default)] pub cost: Option<f32>,
    #[serde(default)] pub input: Option<String>,
    #[serde(default)] pub anim: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeleportSpec {
    #[serde(default)] pub cost: Option<f32>,
    #[serde(default)] pub input: Option<String>,
    #[serde(default)] pub anim: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuperSpeedSpec {
    #[serde(default)] pub mul: Option<f32>,
    #[serde(default)] pub cost: Option<f32>,
    #[serde(default)] pub input: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormSpec {
    pub id: String,
    pub name: String,
    #[serde(default)] pub cost: Option<f32>,
    #[serde(default)] pub attack: Option<f32>,
    #[serde(default)] pub defense: Option<f32>,
    #[serde(default)] pub palette: Option<Value>,
    #[serde(default)] pub parts: Option<Vec<String>>,
    #[serde(default)] pub character: Option<String>,
    #[serde(default)] pub moves: Vec<NativeMove>,
    #[serde(default, rename = "replaceMoves", alias = "replace_moves")] pub replace_moves: bool,
}
