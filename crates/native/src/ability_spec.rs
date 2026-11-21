use super::types::NativeMove;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AbilitySpec {
    #[serde(default)] pub fly: Option<FlySpec>,
    #[serde(default)] pub teleport: Option<TeleportSpec>,
    #[serde(default, rename = "superSpeed")] pub super_speed: Option<SuperSpeedSpec>,
    #[serde(default)] pub charge: Option<ChargeSpec>,
    #[serde(default)] pub forms: Vec<FormSpec>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FlySpec {
    #[serde(default)] pub speed: Option<f32>,
    #[serde(default)] pub cost: Option<f32>,
    #[serde(default)] pub input: Option<String>,
    #[serde(default)] pub anim: Option<String>,
    #[serde(default, rename = "upAnim")] pub up_anim: Option<String>,
    #[serde(default, rename = "downAnim")] pub down_anim: Option<String>,
    #[serde(default, rename = "fwdAnim")] pub fwd_anim: Option<String>,
    #[serde(default, rename = "backAnim")] pub back_anim: Option<String>,
    #[serde(default)] pub sound: Option<[u16; 2]>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TeleportSpec {
    #[serde(default)] pub cost: Option<f32>,
    #[serde(default)] pub input: Option<String>,
    #[serde(default)] pub anim: Option<String>,
    #[serde(default)] pub at: Option<i32>,
    #[serde(default)] pub offset: Option<f32>,
    #[serde(default)] pub invuln: Option<i32>,
    #[serde(default)] pub turn: Option<bool>,
    #[serde(default)] pub sound: Option<[u16; 2]>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SuperSpeedSpec {
    #[serde(default)] pub mul: Option<f32>,
    #[serde(default)] pub cost: Option<f32>,
    #[serde(default)] pub input: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChargeSpec {
    #[serde(default)] pub input: Option<String>,
    #[serde(default)] pub anim: Option<String>,
    #[serde(default, rename = "auraColor")] pub aura_color: Option<[i32; 3]>,
    #[serde(default)] pub shake: Option<bool>,
    #[serde(default)] pub sound: Option<[u16; 2]>,
    #[serde(default)] pub explod: Option<String>,
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
