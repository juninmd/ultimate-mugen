use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub type Box4 = [f32; 4];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveSpec { pub part: String, #[serde(default)] pub radius: Option<f32>, #[serde(default, rename = "box")] pub bx: Option<Box4> }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeFrame {
    #[serde(default)] pub pose: Option<Value>,
    #[serde(default)] pub effect: Option<Value>,
    #[serde(default)] pub sprite: Option<String>,
    pub ticks: i32,
    #[serde(default)] pub hit: Option<ActiveSpec>,
    #[serde(default)] pub hurt: Option<Value>,
    #[serde(default)] pub offset: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeAnim { pub frames: Vec<NativeFrame>, #[serde(default, rename = "loop")] pub loop_start: Option<usize> }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HitSpec {
    pub at: i32, pub damage: f32,
    #[serde(default, rename = "guardDamage")] pub guard_damage: Option<f32>,
    #[serde(default)] pub level: Option<String>,
    #[serde(default)] pub guard: Option<String>,
    #[serde(default)] pub knockback: Option<[f32; 2]>,
    #[serde(default, rename = "airKnockback")] pub air_knockback: Option<[f32; 2]>,
    #[serde(default)] pub launch: Option<bool>,
    #[serde(default)] pub hitstop: Option<i32>,
    #[serde(default)] pub hitstun: Option<i32>,
    #[serde(default)] pub spark: Option<i32>,
    #[serde(default)] pub shake: Option<i32>,
    #[serde(default)] pub priority: Option<i32>,
    #[serde(default)] pub hitflag: Option<String>,
    #[serde(default)] pub pausetime: Option<[i32; 2]>,
    #[serde(default)] pub cornerpush: Option<f32>,
    #[serde(default, rename = "hitSound")] pub hit_sound: Option<[i32; 2]>,
    #[serde(default, rename = "guardSound")] pub guard_sound: Option<[i32; 2]>,
    #[serde(default, rename = "sparkPos")] pub spark_pos: Option<[f32; 2]>,
    #[serde(default, rename = "groundBounce")] pub ground_bounce: Option<bool>,
    #[serde(default, rename = "wallBounce")] pub wall_bounce: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectileSpec {
    pub at: i32, pub anim: String,
    #[serde(default, rename = "hitAnim")] pub hit_anim: Option<String>,
    pub speed: [f32; 2],
    #[serde(default)] pub accel: Option<[f32; 2]>,
    pub offset: [f32; 2],
    #[serde(default)] pub hits: Option<i32>,
    pub damage: f32,
    #[serde(default)] pub level: Option<String>,
    #[serde(default)] pub launch: Option<bool>,
    #[serde(default)] pub knockback: Option<[f32; 2]>,
    #[serde(default, rename = "removeTime")] pub remove_time: Option<i32>,
    #[serde(default)] pub priority: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> { One(T), Many(Vec<T>) }
impl<T: Clone> OneOrMany<T> { pub fn to_vec(&self) -> Vec<T> { match self { OneOrMany::One(t) => vec![t.clone()], OneOrMany::Many(v) => v.clone() } } }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VelChange { pub at: i32, pub vel: [Option<f32>; 2] }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArmorSpec { pub hits: u32, #[serde(default)] pub threshold: Option<f32> }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParrySpec { pub window: [i32; 2], #[serde(default)] pub spark: Option<i32> }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThrowSpec {
    pub damage: f32,
    #[serde(default)] pub bind: Option<[f32; 2]>,
    #[serde(default, rename = "p2State")] pub p2_state: Option<i32>,
    #[serde(default)] pub at: Option<i32>,
    #[serde(default)] pub launch: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeMove {
    pub id: String, pub name: String, pub input: String,
    #[serde(default)] pub simple: Option<String>,
    pub kind: String,
    #[serde(default)] pub when: Option<String>,
    pub anim: String,
    #[serde(default)] pub hits: Vec<HitSpec>,
    #[serde(default)] pub moves: Vec<VelChange>,
    #[serde(default)] pub projectile: Option<OneOrMany<ProjectileSpec>>,
    #[serde(default)] pub cost: Option<f32>,
    #[serde(default)] pub invuln: Option<[i32; 2]>,
    #[serde(default, rename = "airPhysics")] pub air_physics: Option<bool>,
    #[serde(default)] pub note: Option<String>,
    #[serde(default)] pub teleport: Option<i32>,
    #[serde(default)] pub fx: Option<String>,
    #[serde(default)] pub voice: Option<String>,
    #[serde(default)] pub armor: Option<ArmorSpec>,
    #[serde(default)] pub parry: Option<ParrySpec>,
    #[serde(default)] pub burst: Option<bool>,
    #[serde(default)] pub flight: Option<bool>,
    #[serde(default)] pub timestop: Option<i32>,
    #[serde(default)] pub throw: Option<ThrowSpec>,
    #[serde(default)] pub cancels: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NativeStats {
    #[serde(default)] pub life: Option<f64>, #[serde(default)] pub power: Option<f64>, #[serde(default)] pub attack: Option<f64>, #[serde(default)] pub defense: Option<f64>,
    #[serde(default)] pub walk: Option<[f64; 2]>, #[serde(default)] pub run: Option<[f64; 2]>, #[serde(default)] pub jump: Option<[f64; 4]>,
    #[serde(default)] pub gravity: Option<f64>, #[serde(default, rename = "airJumps")] pub air_jumps: Option<f64>,
    #[serde(default)] pub height: Option<f64>, #[serde(default)] pub width: Option<[f64; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeArt {
    #[serde(rename = "type")] pub kind: String,
    #[serde(default)] pub style: Value,
    #[serde(default)] pub poses: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeChar {
    pub format: String, pub id: String, pub name: String, pub series: String, pub author: String,
    #[serde(default)] pub description: Option<String>,
    pub color: String,
    #[serde(default)] pub stats: NativeStats,
    #[serde(default)] pub passives: Option<super::passive_spec::PassiveSpec>,
    #[serde(default)] pub abilities: Option<super::ability_spec::AbilitySpec>,
    pub art: NativeArt,
    pub anims: indexmap_like::OrderedMap<NativeAnim>,
    pub moves: Vec<NativeMove>,
    #[serde(default)] pub palettes: Vec<Value>,
    #[serde(default, rename = "awakeningPalette")] pub awakening_palette: Option<Value>,
    #[serde(default, rename = "winQuote")] pub win_quote: Option<String>,
    /// round-start lines: `intro` is said first, `taunt` is the reply / provocation
    #[serde(default)] pub intro: Option<String>, #[serde(default)] pub taunt: Option<String>,
    /// name of the transformed form (shown when the fighter awakens)
    #[serde(default)] pub form: Option<String>,
    /// final boss: hidden from the select screen, closes the arcade ladder
    #[serde(default)] pub boss: bool,
    #[serde(default)] pub npc: bool,
    #[serde(default, rename = "spriteSet")] pub sprite_set: Option<String>,
    /// animation speed multiplier (> 1 = every timed frame is shorter: faster moves, startup and recovery)
    #[serde(default)] pub speed: Option<f32>,
}

/// Minimal insertion-ordered map for deterministic animation numbering.
pub mod indexmap_like {
    use serde::de::{MapAccess, Visitor};
    use serde::ser::SerializeMap;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    #[derive(Clone, Debug, Default)]
    pub struct OrderedMap<V>(pub Vec<(String, V)>);
    impl<V> OrderedMap<V> {
        pub fn get(&self, k: &str) -> Option<&V> { self.0.iter().find(|(n, _)| n == k).map(|(_, v)| v) }
        pub fn iter(&self) -> impl Iterator<Item = (&String, &V)> { self.0.iter().map(|(k, v)| (k, v)) }
    }
    impl<V: Serialize> Serialize for OrderedMap<V> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> { let mut m = s.serialize_map(Some(self.0.len()))?; for (k, v) in &self.0 { m.serialize_entry(k, v)?; } m.end() }
    }
    impl<'de, V: Deserialize<'de>> Deserialize<'de> for OrderedMap<V> {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct Vis<V>(std::marker::PhantomData<V>);
            impl<'de, V: Deserialize<'de>> Visitor<'de> for Vis<V> {
                type Value = OrderedMap<V>;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("map") }
                fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> { let mut v = vec![]; while let Some((k, x)) = a.next_entry()? { v.push((k, x)); } Ok(OrderedMap(v)) }
            }
            d.deserialize_map(Vis(std::marker::PhantomData))
        }
    }
}
